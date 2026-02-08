# ADR 0029: Text Pipeline, Atlas Strategy, and Quality Targets (GPUI-Inspired)

Status: Accepted

## Context

Editor-grade UI requires high-quality text for:

- property/inspector panels,
- logs/diagnostics,
- eventually code-editor-grade text (large documents, selection, IME composition).

Text is also one of the most common sources of “late rewrites” due to:

- unclear ownership between layout/shaping/rasterization/rendering,
- missing caching keys and invalidation strategy,
- platform differences (fonts, fallbacks, emoji/color glyphs),
- insufficient quality controls (gamma/contrast/subpixel).

Fret already defines a stable core boundary (`TextBlobId` + metrics) in ADR 0006. This ADR specifies how we
should *implement* that boundary in a way that remains portable (desktop now, wasm later) and scales to large editors.

GPUI provides a proven reference architecture:

- an app-level `TextSystem` with caching and fallback handling,
- a `PlatformTextSystem` trait for platform-specific shaping/raster bounds/rasterization,
- atlas-backed glyph/image rendering with shader-based quality adjustments.

References:

- GPUI text system:
  - `repo-ref/zed/crates/gpui/src/text_system.rs`
  - `repo-ref/zed/crates/gpui/src/platform.rs` (`PlatformTextSystem`)
  - `repo-ref/zed/crates/gpui/src/platform/linux/text_system.rs` (cosmic-text integration)
- GPUI shader-side text quality work (gamma/contrast helpers):
  - `repo-ref/zed/crates/gpui/src/platform/blade/shaders.wgsl`
- Fret text boundary:
  - `docs/adr/0006-text-system.md`

## Decision

### 1) Keep the core contract stable (ADR 0006)

`fret-ui` only consumes:

- `TextMetrics` for layout decisions,
- `TextBlobId` for painting (`SceneOp::Text`),

and never touches shaping, font loading, atlas allocation, or GPU uploads.

### 2) Provide a platform-facing shaping/rasterization interface

Adopt a GPUI-like abstraction (names TBD):

- `PlatformTextSystem`: resolves fonts, shapes runs, computes metrics and raster bounds, rasterizes glyph bitmaps.

Initial implementation strategy:

- Historical note: this ADR originally proposed `cosmic-text` as the default cross-platform shaper.
- Current direction: shaping/layout converges on Parley (ADR 0157) with wrapper-owned wrap/ellipsis. Backend/library choices that remain (font DB bridging, fallback heuristics) must not change the stable UI boundary (ADR 0006).
- Keep the interface compatible with future native backends (e.g. DirectWrite, CoreText) without changing `fret-core`.

### 3) Atlas strategy: separate monochrome vs polychrome

Text and icon rendering must anticipate two classes of glyphs:

- **Monochrome glyphs**: typical text glyphs, can be stored in a single-channel atlas.
- **Polychrome glyphs**: emoji / color glyphs, require RGBA atlas tiles.

We should keep these as separate atlas resources (different formats/sampling paths), to avoid performance and
quality regressions.

### 4) Text glyphs are coverage bitmaps (not SDF) in the mainline path

Even though Fret uses analytic SDF for shape primitives (ADR 0030), text should follow GPUI’s proven approach:

- rasterize glyphs to **coverage bitmaps** (alpha masks / subpixel-positioned variants if needed),
- store them in atlases and apply shader-side quality adjustments (gamma/contrast),
- do not rely on SDF/MSDF for general text rendering.

Rationale:

- small-size text quality is dominated by hinting/rasterization decisions that SDF tends to lose,
- emoji/color glyph support naturally wants RGBA tiles anyway,
- atlas + quality shader knobs scale better for “code editor grade” text than SDF does.

### 5) Define quality targets early (gamma/contrast/subpixel)

Text quality is not just shaping—it is also sampling and compositing. We should define targets:

- blending: premultiplied alpha (consistent with ADR 0002),
- gamma/contrast correction strategy for glyph alpha (platform-dependent tuning allowed),
- optional subpixel positioning variants (especially for code editor readability).

These targets define shader and cache key requirements, so they must be decided before building a full text stack.

Locked P0 text quality baseline:

- Raster: grayscale coverage masks (alpha), premultiplied blending.
- Positioning: enable subpixel positioning by rasterizing X-offset variants (`x4`) for small text; Y variants are deferred.
- Emoji/color glyphs: reserve the RGBA atlas path; implementation can start partial, but the pipeline contract stays stable.

### 6) Caching and invalidation are explicit

Define stable caching keys for `TextBlobId` creation, at minimum:

- font + size + style features,
- text content (or hash),
- wrap constraints / max width,
- shaping options (ligatures, fallback policy),
- DPI scale factor considerations (rasterization depends on it; see `TextConstraints.scale_factor` in ADR 0006).

Invalidation must be explicit and compatible with the effects flush loop (ADR 0001 / ADR 0004).

## Consequences

- We can start with property-panel text and scale to code-editor text without changing `SceneOp` or `UiTree`.
- The rendering path becomes predictable: shaped runs → glyph raster → atlas tiles → `SceneOp::Text`.
- We avoid locking ourselves into a single platform backend while still shipping a cross-platform text stack early.

## Open Questions (To Decide Before Implementation)

1) **Backend split**:
   - do we want a single `cosmic-text` backend everywhere at first, or per-OS backends from day one?
2) **Subpixel strategy**:
   - grayscale only vs subpixel positioning variants vs LCD rendering (and wasm constraints).
3) **Emoji policy**:
   - which fallback chain and which atlas format is used for emoji?
4) **Font discovery and user font loading**:
    - how do we map OS font discovery to stable `FontId` and caching?
5) **Shader contracts**:
   - what parameters become part of the glyph sampling shader interface (gamma ratios, contrast factors)?

## Implementation guidance: default font stack and fallbacks (P0 lock-in)

Rust GUI libraries frequently fail IME and international text correctness not because IME events
are missing, but because the default font does not cover the intermediate/provisional glyphs used
by IME composition (fullwidth Latin, kana) or because fallback behavior is inconsistent.

Fret locks the following baseline to avoid "tofu during composition" and non-deterministic glyph
selection across machines:

### Default font stack

- `TextStyle.font` (or its higher-level theme token) must resolve to a concrete *family preference*
  rather than an "empty default" (e.g. `FontId::default()` without semantics).
- Each platform provides a "system UI font" alias (implementation-defined) used as the default.
- A configurable ordered list of default family candidates is supported at the settings layer
  (ADR 0014):
  - `./.fret/settings.json` → `fonts.ui_sans` / `fonts.ui_serif` / `fonts.ui_mono`.
  - The renderer selects the first installed family from the list, otherwise falls back to the
    platform defaults (see `default_*_candidates()` in `crates/fret-render`).
  - This is the minimum needed to avoid “default font has no kana/kanji” failures (common root cause
    for IME/tofu issues).
  - Full per-script fallback customization (UI/CJK/emoji stacks) is still tracked as follow-up work.

### Fallback resolution behavior

Baseline behavior (P0):

1) Resolve the requested primary font family.
2) If the primary cannot be loaded, fall back to the framework's default font stack.
3) During shaping/rasterization, if a glyph is missing:
   - attempt per-style configured fallback list first,
   - then attempt the framework default fallback stack,
   - only then emit tofu/missing-glyph.

### Caching invariants

To keep text output deterministic and avoid stale cache bugs:

- The fallback list (style-level and framework default stack) must participate in the text cache key
  used to produce and reuse `TextBlobId`.
- Any change in font discovery, user font loading, or configured fallbacks must invalidate the
  relevant cached text blobs (directly or via a text-system revision key).

Current status:

- The resolved default family names (`SansSerif`/`Serif`/`Monospace`) participate in the text cache key
  via a `font_stack_key` (see `crates/fret-render-wgpu/src/text.rs`).

### Conformance smoke tests (recommended)

Add at least one integration/demo harness that renders and measures:

- ASCII + fullwidth Latin (IME provisional), hiragana/katakana, common kanji, and emoji.
- A preedit sequence that includes fullwidth Latin + kana before commit (Windows Japanese IME).

## Current MVP status (implementation notes)

This ADR defines the contract and lock-in targets; implementation will evolve. Current code status:

- Text shaping/rasterization uses `cosmic-text` via the renderer text system:
  - `crates/fret-render-wgpu/src/text.rs`
- A framework-level fallback policy and platform-generic family defaults are configured at text-system startup, and a
  `font_stack_key` participates in `TextBlobId` caching so fallback/generic-family changes cannot reuse stale blobs.
- `TextStyle.font` selects among the three generic families (`SansSerif`/`Serif`/`Monospace`) via the semantic `FontId`
  aliases (`FontId::ui()`/`FontId::serif()`/`FontId::monospace()`), which are then resolved by the text backend.

Follow-up decisions (still required):

- Expand the stable mapping beyond the three built-in generic families (user font loading, stable IDs, and persistence
  format: store family + features + fallbacks, never numeric `FontId`).

MVP note:

- `FontId::default()` is treated as the system UI sans-serif alias; `FontId::serif()` / `FontId::monospace()` exist as
  additional built-in aliases. These are interim affordances until the settings-layer font stack and persistence format
  are fully defined.
