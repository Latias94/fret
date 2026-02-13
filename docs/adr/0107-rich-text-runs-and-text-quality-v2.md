# ADR 0107: Rich Text Runs and Text Quality v2 (Runs, Decorations, Subpixel Strategy)

Status: Proposed

## Context

Fret’s current text contract (ADR 0006 / ADR 0029) is intentionally simple:

- `TextService::prepare(text: &str, style: &TextStyle, constraints: TextConstraints) -> (TextBlobId, TextMetrics)`
- `SceneOp::Text { text: TextBlobId, color, origin }`

This works well for “uniform style” UI text. However, editor-grade surfaces (and the new Markdown
surface in ADR 0099) require **inline style variation** that must still participate in wrapping,
selection geometry, and high-performance caching.

### Problems observed in the current codebase

1) **No rich text runs in the core contract**
- `fret-core::TextStyle` applies to a full string; there is no notion of per-range style.
- `TextProps` carries a single `TextStyle` and an optional single `color`.

2) **Markdown has to “fake” inline features**
- `ecosystem/fret-markdown` currently approximates strikethrough by overlaying a 1px rectangle on
  top of a `Text` element, and does not express italic/underline as true text intent.
- This blocks correct wrapping across inline segments and increases element count and layout cost.

3) **Code highlighting is structurally incompatible with soft wrap**
- `ecosystem/fret-code-view` represents a highlighted line as multiple `Text` elements (one per
  highlight segment). This makes “soft wrap + highlight” impractical because wrapping must be
  decided by a single shaper/line-breaker across the entire line/paragraph.

4) **Text can appear blurry under DPI scaling**
- Current renderer uses a **filtering sampler** for the glyph atlas and draws glyph quads at
  fractional device-pixel positions.
- Current text backend also converts `SwashContent::SubpixelMask` into a single-channel alpha mask
  (`max(r,g,b)`), which removes LCD/subpixel information.
- Hinting is currently disabled in `layout_to_buffer(..., Hinting::Disabled)`.
- Legacy note: earlier backends used 4-way subpixel binning but did not always apply the same snap rule
  at quad placement time, which can make placement disagree with the rasterized variant (visible as “soft” text).

Zed/GPUI provides a proven reference for addressing these issues:

- run-based shaping (`TextRun`) and a `StyledText` element,
- decorations as first-class runs (underline/strikethrough),
- explicit subpixel positioning strategy (`SUBPIXEL_VARIANTS_X = 4`) and shader-side gamma/contrast
  correction.

References:
- Zed GPUI text system: `repo-ref/zed/crates/gpui/src/text_system.rs`
- Zed `StyledText`: `repo-ref/zed/crates/gpui/src/elements/text.rs`
- Zed gamma/contrast shader: `repo-ref/zed/crates/gpui/src/platform/blade/shaders.wgsl`
- Rich content selection contract (Markdown-first): `docs/adr/0108-rich-content-selection-and-clipboard.md`

## Goals

1) Enable **rich inline styling** (Markdown, chat transcripts, code highlights) with correct
   wrapping and geometry semantics.
2) Keep the “editor-grade” text boundary: UI never performs shaping or atlas management.
3) Make caching correct and future-proof:
   - layout/shaping cache should not be invalidated by theme color changes,
   - text quality knobs must participate in cache keys.
4) Establish a concrete **text quality baseline** to avoid blurry output under DPI scaling.

## Decision

### 1) Introduce a first-class “rich text runs” input surface

We introduce a new backend-agnostic input type that can represent per-range styling:

```rust
pub struct TextRun {
    /// Run length in UTF-8 bytes.
    pub len: usize,
    /// Shaping-affecting overrides (font/weight/slant/tracking). Size is expected to be uniform.
    pub shaping: TextShapingStyle,
    /// Paint-only attributes (colors, backgrounds, decorations).
    pub paint: TextPaintStyle,
}

pub struct AttributedText {
    pub text: Arc<str>,
    pub runs: Arc<[TextRun]>,
}
```

Constraints:
- Runs must cover the entire `text` (sum(len) == text.len()).
- Run boundaries must be valid UTF-8 char boundaries.
- For v2, `font_size` and `line_height` are **uniform per layout** (defined by a base style); runs
  may override family/weight/slant/tracking.

Rationale:
- This matches the “practical minimum” needed by Markdown and syntax highlighting.
- Keeping size/line-height uniform avoids hard-to-change multiline metrics complexity early.

### 2) Split shaping/layout from paint-only attributes (cache correctness)

We introduce a two-layer concept:

- **Layout key / shaped layout**: depends on text content, constraints, font stack, and shaping
  attributes (font/weight/slant/tracking), plus scale factor and quality knobs.
- **Paint**: depends on theme colors and decorations, and is applied at draw time without forcing a
  new shaped layout.

This avoids cache explosion where “same text, different theme color” creates multiple blobs.

### 3) Evolve the core contract to accept attributed text

We extend the `TextService` contract to accept either plain or attributed inputs.

Options (exact naming TBD):

1. Replace `prepare(&str, &TextStyle, ...)` with:
   - `prepare_text(&TextInput, constraints) -> (TextBlobId, TextMetrics)`
2. Keep the existing API as a convenience wrapper around a single-run attributed input.

Given the repository allows fearless refactors, we prefer the “single entry point” contract:

```rust
pub enum TextInput {
    Plain { text: Arc<str>, style: TextShapingStyle },
    Attributed { text: Arc<str>, base: TextShapingStyle, runs: Arc<[TextRun]> },
}
```

### 4) Add glyph-to-text mapping to enable run-aware paint without reshaping

To apply per-run paint in the renderer without rebuilding layouts, the shaped layout must retain a
mapping from glyph quads back to text indices.

Implementation direction:
- Store at least one stable index per glyph quad (e.g. `start` byte offset or cluster index).
- Encoding then assigns paint attributes by walking the run ranges in order (linear-time merge),
  similar to how syntax highlight spans are merged today.

### 5) Decorations and inline backgrounds become renderer-supported

We standardize that underline/strikethrough and inline background are not “widget hacks”.

Decorations are derived from layout line metrics:
- drawn per visual line segment,
- respect clipping/scissoring,
- ordered consistently with text:
  - background quads behind glyphs,
  - decorations above glyphs (line-through/underline).

This ADR builds on ADR 0102 (Text Decorations and Markdown Theme Tokens) for the vocabulary, but
locks the additional “runs + renderer contract” needed to implement it correctly.

### 6) Text quality baseline: subpixel positioning + gamma/contrast controls

We adopt a GPUI-aligned baseline:

- Subpixel positioning uses discrete variants:
  - `SUBPIXEL_VARIANTS_X = 4`
  - `SUBPIXEL_VARIANTS_Y = 1` on Windows/Linux; `= 4` on macOS (matching GPUI’s defaults).
- Glyph quads should be aligned to integer device pixels, with the fractional position encoded in
  the chosen subpixel variant (atlas key).
- Provide shader-side correction parameters for grayscale coverage masks:
  - gamma ratios and an “enhanced contrast” factor (as used by Zed).

Notes:
- This baseline targets “crisp code/editor text” under non-integer scale factors.
- LCD (true subpixel RGB text) remains optional future work; this ADR does not require it.

### 7) Caching keys must include quality knobs and shaping-relevant fields

The shaped-layout cache key must include:
- text content (hash + length, and/or `Arc<str>`),
- wrap constraints / overflow / max width,
- scale factor,
- resolved font stack key / font database revision,
- shaping attributes participating in glyph selection,
- subpixel variant strategy (if it changes output),
- hinting/shaping mode if configurable.

Paint-only attributes (colors, decoration colors) must NOT be part of the shaped-layout key.

## Consequences

- This is a cross-cutting change (fret-core / fret-ui / fret-render):
  - new text input type and updated `TextService` contract,
  - new UI element surface for rich runs,
  - renderer-side run-aware paint, backgrounds, decorations,
  - new text quality knobs and shader parameters.
- Markdown becomes dramatically simpler and more correct:
  - remove per-feature overlay hacks (strikethrough lines),
  - enable correct wrapping across inline segments,
  - express italic/underline/line-through as text intent.
- Code blocks gain a path to “highlight + soft wrap” without exploding element counts.

## Alternatives Considered

1) **Keep per-segment `Text` elements** (status quo)
   - Rejected: cannot support wrapping across segments; too many nodes; hacky decorations.
2) **Create separate blobs per run and place them manually**
   - Rejected: breaks line wrapping and geometry semantics; hard to handle Unicode and ligatures.
3) **Bake paint attributes into the blob cache key**
   - Rejected: theme changes would explode caches and increase memory.

## Follow-ups

1) Reconcile ADR 0102 with this ADR:
   - Either mark ADR 0102 as “Superseded by ADR 0107” or merge its vocabulary section as an
     appendix once implementation starts.
2) Define the minimal `TextRun`/`TextInput` types in `fret-core` and update `SceneOp::Text` to carry
   run paint data as needed.
3) Implement subpixel variant selection and gamma/contrast correction in the renderer.
4) Migrate `fret-markdown` and `fret-code-view` to use the new rich text surface.
