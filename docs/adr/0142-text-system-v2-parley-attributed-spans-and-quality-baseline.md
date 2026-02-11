# ADR 0142: Text System v2 (Parley Shaping, Attributed Spans, and a Quality Baseline)

- Status: Proposed
- Date: 2026-01-13

## Context

Fret targets editor-grade UI and needs a text stack that can scale from “UI labels” to:

- Markdown and rich inline styling (bold/italic/links/code/strikethrough),
- command palettes and lists (fast truncation/ellipsis and deterministic measurement),
- selection/caret/hit-testing with stable semantics (ADR 0045 / ADR 0046),
- eventual code editor surfaces (virtualized, high-volume, high-DPI correctness).

The repository already locks the **UI/renderer boundary**:

- `TextBlobId` + `TextMetrics` + geometry queries (ADR 0006 / ADR 0045 / ADR 0046),
- atlas-backed glyph rendering (ADR 0029),
- truncation semantics via `TextOverflow::Ellipsis` (ADR 0059).

What is not locked yet is the **future-proof implementation shape** that avoids late rewrites:

- the attributed text input model (shaping vs paint separation),
- the line/chunk shaping backend choice (Parley),
- the multi-line wrapper strategy (wrap/truncate as a separate layer),
- the atlas/quality baseline (subpixel strategy + gamma/contrast correction),
- cache keys and invalidation rules (theme changes must not cause reshaping).

Zed/GPUI provides a proven reference decomposition:

- `PlatformTextSystem` (font resolution + glyph raster + metrics),
- `LineWrapper` (wrap/truncate driven by measured advances),
- a text system that caches shaped layouts and atlas tiles separately,
- shader-side gamma/contrast correction and subpixel positioning variants.

This ADR updates the implementation direction described in ADR 0029 and supersedes ADR 0107’s v2 sketch.

## Goals

1) Make rich inline styling a first-class input, without “many Text nodes” hacks.
2) Keep caching correct: theme color changes do not invalidate shaping/layout caches.
3) Keep geometry queries deterministic and derived from the same layout used for rendering.
4) Lock a text quality baseline that avoids blurry output under non-integer scaling.
5) Adopt Parley for low-level line/chunk shaping and keep multi-line layout controlled by Fret.
6) Do not rely on backend feature gates for the mainline implementation.

## Decision

### 1) Introduce attributed text spans with explicit shaping vs paint separation

Replace the current “rich runs” surface with an attributed span model that separates:

- **Shaping-affecting attributes**: participate in the layout/shaping cache key.
- **Paint-only attributes**: applied at draw time and must not participate in the shaping key.

Proposed core types (exact naming may evolve, but the split is locked):

```rust
pub enum TextInput {
    Plain {
        text: Arc<str>,
        style: TextStyle,
    },
    Attributed {
        text: Arc<str>,
        base: TextStyle,
        spans: Arc<[TextSpan]>,
    },
}

pub struct TextSpan {
    /// Span length in UTF-8 bytes.
    pub len: usize,
    pub shaping: TextShapingStyle,
    pub paint: TextPaintStyle,
}

pub struct TextShapingStyle {
    pub font: Option<FontId>,
    pub weight: Option<FontWeight>,
    pub slant: Option<TextSlant>,
    pub letter_spacing_em: Option<f32>,
    // Future: font features, fallbacks, language/script hints.
}

pub struct TextPaintStyle {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub underline: Option<UnderlineStyle>,
    pub strikethrough: Option<StrikethroughStyle>,
}
```

Constraints:

- Spans must cover the entire string (`sum(len) == text.len()`).
- Span boundaries must be valid UTF-8 char boundaries.
- For v2, `size` and `line_height` remain uniform per layout (defined by `base`/`style`).

### 2) Keep the stable UI boundary and evolve `TextService` with a single entry point

`fret-ui` continues to consume:

- `TextMetrics` for layout,
- `TextBlobId` for paint via `SceneOp::Text`,
- geometry queries on prepared blobs (ADR 0045 / ADR 0046).

`TextService` is evolved to accept `TextInput`:

- `prepare(input: &TextInput, constraints: TextConstraints) -> (TextBlobId, TextMetrics)`
- `measure(input: &TextInput, constraints: TextConstraints) -> TextMetrics`

The existing `prepare(&str, &TextStyle, ...)` shape becomes a convenience wrapper (or is removed),
but the stable cross-crate boundary remains “UI never shapes”.

### 3) Adopt Parley for line/chunk shaping (single line as the primitive)

We adopt:

- `parley` for low-level shaping and line layout of a single line/chunk,
- Fret-owned multi-line layout orchestration (wrapping/truncation) as a separate layer.

Parley is used as the “shaping engine”; higher-level decisions are not delegated to it:

- wrap boundaries, ellipsis/truncation, and multi-line assembly are Fret-owned.

### 4) Implement multi-line layout via a dedicated wrapper layer (Zed/GPUI-aligned)

Introduce a `LineWrapper`-like layer inside the renderer text system:

- Computes wrap boundaries under `TextConstraints { max_width, wrap }`.
- Implements `TextOverflow::Ellipsis` (ADR 0059) for single-line truncation.
- Drives the shaper repeatedly on slices, producing `TextLine` records.

This keeps shaping backend responsibilities narrow and makes future algorithm work (CJK boundaries,
grapheme/cluster rules, bidi edge cases) testable at the wrapper layer.

### 5) `TextBlobId` must carry glyph-to-text mapping for run-aware paint and geometry queries

Prepared blobs must store enough metadata to support:

- caret/hit-test/selection rects (ADR 0045 / ADR 0046),
- assigning per-span paint attributes without reshaping.

Minimum requirement:

- each glyph instance carries a stable mapping to a text byte index (or cluster index).

### 6) Atlas strategy is multi-page, budgeted, and evictable (no append-only growth)

Text atlas storage must be:

- split by texture kind (monochrome vs polychrome; optional subpixel),
- multi-page (allocators per page),
- budgeted and evictable (LRU/epoch-based eviction),
- deterministic under pressure (explicit rebuild paths are allowed, but “silent growth forever” is not).

Allocation strategy is expected to use a free-rect allocator (e.g. `etagere`) rather than a pure
append-only “pen”.

### 7) Lock a text quality baseline (subpixel variants + shader correction)

Adopt a GPUI-aligned baseline:

- `SUBPIXEL_VARIANTS_X = 4`
- `SUBPIXEL_VARIANTS_Y = 1` on Windows/Linux, `= 4` on macOS (matching GPUI defaults)
- glyph sampling uses shader-side gamma/contrast correction parameters for grayscale coverage masks

These quality knobs participate in the shaping/raster/atlas cache keys when they change.

### 8) Cache keys and invalidation rules are explicit

We maintain two conceptual caches:

- **Layout/shaping cache**: depends on text content, constraints, font stack, shaping attributes, scale factor, and quality knobs.
- **Paint**: depends on theme colors and decorations; does not invalidate the shaped layout.

Font database mutations (user-loaded fonts, platform discovery changes) must invalidate affected caches via a revision key.

## Consequences

- This is a cross-cutting change (core contract + renderer + UI authoring surfaces).
- Markdown/code highlighting become structurally compatible with wrapping and selection.
- Theme color changes no longer cause cache explosions or unnecessary shaping.
- The text system becomes ready for long-lived editor sessions (budgeted atlas + deterministic eviction).

## Alternatives Considered

1) Keep per-segment `Text` elements for rich styling
   - Rejected: cannot wrap across spans; too many nodes; geometry semantics drift.
2) Keep a paragraph-level shaping backend that owns wrapping decisions
   - Rejected: makes wrap/truncate policy hard to test and hard to keep stable across backends.
3) Keep the current backend behind a feature gate
   - Rejected: increases long-term maintenance cost; this ADR locks the mainline path.

## Migration Plan (Non-Normative)

1) Add `TextInput`/`TextSpan` to `fret-core` and migrate `TextService` to a single entry point.
2) Implement the Parley-based text system in `fret-render` behind the stable `TextService` contract.
3) Update `fret-ui` elements (`Text`, `StyledText`, `SelectableText`) to emit `TextInput`.
4) Migrate `ecosystem/fret-markdown` and `ecosystem/fret-code-view` to spans (decorations and backgrounds become real).
5) Add conformance tests:
   - ellipsis truncation mapping (caret/hit-test),
   - wrap + selection rect correctness across span boundaries,
   - theme-only changes do not trigger reshaping (cache behavior).
