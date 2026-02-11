# ADR 0257: Font Selection, Fallback, and Variable Font Instance Identity (Fontique-backed)

- Status: Proposed
- Date: 2026-02-11

## Context

Fret’s text boundary (ADR 0006) intentionally keeps UI portable and backend-agnostic. The renderer owns:

- font enumeration for settings pickers,
- font selection and fallback behavior,
- shaping and rasterization (via Parley/fontique + Swash),
- caching keys and invalidation (`TextFontStackKey`, `TextBlobKey`, glyph atlas keys).

Recent refactors converged family resolution to a single source of truth (Parley/fontique), but several *font-system*
gaps remain that can cause correctness drift and make refactors risky:

1) **Variable font instance identity must be explicit and cache-safe**.
   - Parley shaping can apply variable font instance coordinates and (optionally) synthesis.
   - The renderer must ensure rasterization matches shaping, and glyph cache keys must include instance identity.
   - As of 2026-02-11, normalized coords are carried end-to-end; synthesis is tracked but not yet applied to raster output.
   - As of 2026-02-11, normalized coords are carried end-to-end; synthesis is now applied to raster output and
     participates in glyph cache identity.

2) **Fallback chain semantics are underspecified**.
   - Fret currently injects a curated/common fallback list into generic families.
   - Fontique supports script + locale fallback keys and backend-driven fallback selection, but the renderer does not
     treat this as a first-class policy surface.

3) **Font enumeration is best-effort and metadata-poor**.
   - A picker-friendly list of family names exists, but we cannot expose “variable axes available” or similar signals.

This ADR locks the direction for a v1 “font system contract” inside the renderer/runner boundary.

Current Fret evidence anchors:

- Generic stack injection and cache invalidation: `crates/fret-render-wgpu/src/text/mod.rs` (`TextSystem::set_font_families`,
  `font_stack_key`, `reset_caches_for_font_change`).
- Locale plumbing for script/locale fallback selection: `crates/fret-render-wgpu/src/text/mod.rs`
  (`TextSystem::set_text_locale`), seeded by runners:
  - `crates/fret-launch/src/runner/desktop/app_handler.rs`
  - `crates/fret-launch/src/runner/web/gfx_init.rs`
- Parley/fontique family resolution: `crates/fret-render-wgpu/src/text/parley_shaper.rs` (`resolve_family_id`,
  `set_generic_family_ids`, `add_fonts`).
- Picker metadata global (axes + monospace hint): `crates/fret-runtime/src/font_catalog.rs`
  (`FontCatalogMetadata`, `FontCatalogEntry`) populated by the runner when refreshing the catalog.
- Variable font instance identity in glyph keys and rasterization:
  - `crates/fret-render-wgpu/src/text/parley_shaper.rs` (`ParleyGlyph::normalized_coords`)
  - `crates/fret-render-wgpu/src/text/mod.rs` (`variation_key_from_normalized_coords`, Swash `normalized_coords(...)`)
  - `crates/fret-render-wgpu/src/text/mod.rs` (`variable_font_weight_changes_face_key_and_raster_output`)
  - `crates/fret-render-wgpu/src/text/mod.rs` (synthesis: `synthesis_skew_participates_in_face_key_and_raster_output`)
- Missing-glyph + selected-family diagnostics trace (bundle-scoped, renderer-owned):
  - `crates/fret-core/src/render_text.rs` (`RendererTextFontTraceSnapshot`)
  - `crates/fret-render-wgpu/src/text/mod.rs` (`TextSystem::font_trace_snapshot`)
  - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (`UiResourceCachesV1.render_text_font_trace`)

## Goals

1) Make variable fonts correct and cache-safe.
2) Make fallback behavior explicit, testable, and part of invalidation keys.
3) Keep `fret-core::FontId` semantic and portable (no backend indexes leak into contracts).
4) Keep Web/WASM deterministic (bundles + curated defaults) while allowing desktop to use system font DBs.

## Non-goals

- Mandate specific font family names (platforms differ).
- Require a stable, cross-platform full font DB API in `fret-core`.
- Define a full end-user UI for arbitrary variation axes (debug/advanced settings can come later).

## Decision

### 0) Add a renderer-owned “font selection trace” to diagnostics bundles

When missing/tofu glyphs happen, engineers need a portable artifact that answers:

- which font was requested (named family vs generic),
- which families the shaper actually used for the blob,
- which families produced missing glyphs.

Decision:

- The renderer records a **bounded, per-frame trace** of prepared text blobs, primarily when missing glyphs are
  observed.
- The runner serializes this trace into the diagnostics bundle (`bundle.json`) so regressions are shareable and
  scriptable.

Implementation anchors:

- Core types: `crates/fret-core/src/render_text.rs` (`RendererTextFontTraceSnapshot`, entries + family usage).
- Renderer capture: `crates/fret-render-wgpu/src/text/mod.rs` (trace ring + snapshot export).
- Bundle serialization: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (`UiRendererTextFontTraceSnapshotV1`).

### 1) Define a renderer-owned “font instance identity” and include it in glyph keys

The renderer defines a stable key that identifies a *font face instance* used for rasterization:

- base font identity (bytes/source + face index),
- variable font instance coordinates (normalized coords),
- synthesis flags if they affect raster output (embolden/skew).

This identity must participate in:

- `GlyphKey` (atlas residency + glyph raster cache),
- any “font face registry” the renderer maintains,
- any cached raster bounds tables keyed by font/glyph/size.

Rationale:

- A single font file can represent many instances (variable fonts). A “font face” is not sufficient.
- Cache aliasing across instances is correctness-breaking and extremely hard to debug later.

### 2) Plumb instance coordinates from shaping into rasterization

When Parley shapes a run, it produces:

- `Run::synthesis()` (fontique synthesis suggestions),
- `Run::normalized_coords()` (HarfBuzz normalized variation coordinates).

The renderer must carry the effective instance coordinates to the rasterizer (Swash) and produce glyph images using the
same instance as shaping.

Upstream evidence anchors:

- `repo-ref/parley/parley/src/layout/run.rs` (`normalized_coords`, `synthesis`)
- `repo-ref/parley/parley/src/shape/mod.rs` (variations iterator combines synthesis + explicit variations)

Implementation note:

- `variation_key` should be derived as a stable fingerprint of `normalized_coords` and any synthesis flags that affect
  raster output. This is an internal renderer detail; it must not be exposed as a stable user-facing value.

### 3) Make fallback chain semantics explicit: generic stacks + script/locale fallbacks + overrides

Font selection uses three layers:

1) **Requested family** from `FontId`:
   - generic (`Ui`/`Serif`/`Monospace`) or named family (`FontId::Family(name)`).
2) **Script + locale fallback** using fontique’s `FallbackKey(script, locale)`:
   - this is backend-driven (CoreText/DirectWrite/fontconfig) and should be treated as the baseline missing-glyph
     behavior when system fonts are present.
3) **Curated/override fallbacks** (`TextFontFamilyConfig.common_fallback` and bundled tiers):
   - used to keep wasm deterministic and to allow apps to enforce “no tofu” baselines.
   - controlled by `TextFontFamilyConfig.common_fallback_injection` (platform default prefers system fallback on desktop).

The effective fallback policy (including locale) must participate in `TextFontStackKey` so cached text cannot reuse
stale selection behavior.

Implementation note:

- Start with a single “default text locale” provided by the runner (OS locale or app setting). Future work may allow
  per-document or per-window locale overrides, but the cache key rules stay the same.

### 4) Font enumeration remains best-effort but is versioned and cacheable

Runners expose a picker list via `FontCatalog` as a best-effort snapshot.

This ADR does not mandate that font enumeration is stable across machines, but it requires:

- case-insensitive dedup,
- deterministic sorting,
- a monotonic revision that bumps when the underlying font collection changes.

Follow-up (recommended): extend the catalog to a `FontCatalogEntry` list that can carry metadata such as axis presence.

### 5) Expose per-span variable font axis overrides (advanced)

To support editor-grade typography and deterministic debugging, the portable text contract exposes a minimal, advanced
surface for variable font axes:

- `fret-core::TextShapingStyle.axes: Vec<TextFontAxisSetting>`
- Each `TextFontAxisSetting` is `{ tag: String, value: f32 }`, where `tag` is the 4-byte OpenType axis tag (e.g. `wght`,
  `opsz`).

Rationale:

- Parley/fontique and Swash already support variable fonts; the missing piece is a portable way to express axis intent in
  attributed spans without leaking backend indexes.
- This allows experiments like optical sizing (`opsz`) and non-weight axes without requiring a full end-user UI yet.

Semantics (v1):

- Axis settings are best-effort: unsupported tags are ignored by the shaping backend.
- Tags must be exactly 4 bytes (after trimming) and values must be finite; invalid entries are ignored.
- For duplicate tags, last-write-wins (canonicalized for cache keys).
- Axis overrides participate in shaping cache keys (span shaping fingerprints) so glyph caches cannot alias across axis
  changes.

## Consequences

- Variable fonts become safe to adopt broadly (including in code editor surfaces).
- Cache keys become “what you see is what you rasterized” — reduced drift between shaping and rendering.
- Fallback behavior becomes auditable and less likely to regress across platforms.

## Implementation plan (tracked)

Workstream: `docs/workstreams/font-system-v1.md`

Milestones:

- M0: variable font instance identity + rasterization coordinates + tests.
- M1: script/locale fallback integration + `TextFontStackKey` participation + conformance strings.
- M2: auditable traces + catalog metadata (optional) for pickers and diagnostics.
- M3: public shaping knobs (features + variations) (optional).

## Audit (inputs)

See `docs/audits/font-system-parley-zed-xilem-2026-02.md` for a focused comparison against:

- Zed/GPUI’s cosmic-text-based system and caching patterns,
- Parley/fontique’s design goals and fallback model,
- Xilem’s Parley-based variable font usage.

## Implementation anchors

- Public axis surface:
  - `crates/fret-core/src/text/mod.rs` (`TextShapingStyle.axes`, `TextFontAxisSetting`)
- Parley style mapping:
  - `crates/fret-render-wgpu/src/text/parley_shaper.rs` (`StyleProperty::FontVariations`)
- Cache keys and conformance tests:
  - `crates/fret-render-wgpu/src/text/mod.rs` (`spans_shaping_fingerprint`,
    `variable_font_axis_overrides_participate_in_face_key_and_raster_output`)
