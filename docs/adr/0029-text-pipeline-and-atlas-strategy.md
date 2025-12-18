# ADR 0029: Text Pipeline, Atlas Strategy, and Quality Targets (GPUI-Inspired)

Status: Proposed

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

- Use `cosmic-text` as the default cross-platform implementation for shaping/line breaking.
- Keep the interface compatible with future native backends (e.g. DirectWrite, CoreText) without changing `fret-core`.

### 3) Atlas strategy: separate monochrome vs polychrome

Text and icon rendering must anticipate two classes of glyphs:

- **Monochrome glyphs**: typical text glyphs, can be stored in a single-channel atlas.
- **Polychrome glyphs**: emoji / color glyphs, require RGBA atlas tiles.

We should keep these as separate atlas resources (different formats/sampling paths), to avoid performance and
quality regressions.

### 4) Define quality targets early (gamma/contrast/subpixel)

Text quality is not just shaping—it is also sampling and compositing. We should define targets:

- blending: premultiplied alpha (consistent with ADR 0002),
- gamma/contrast correction strategy for glyph alpha (platform-dependent tuning allowed),
- optional subpixel positioning variants (especially for code editor readability).

These targets define shader and cache key requirements, so they must be decided before building a full text stack.

### 5) Caching and invalidation are explicit

Define stable caching keys for `TextBlobId` creation, at minimum:

- font + size + style features,
- text content (or hash),
- wrap constraints / max width,
- shaping options (ligatures, fallback policy),
- DPI scale factor considerations (if rasterization depends on it).

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

