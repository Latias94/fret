# Workstream: Font System v1 (Fontique Audit + Roadmap)

Status: In progress

This workstream focuses on the *font system* (not the entire text pipeline):

- font enumeration (what a settings picker sees),
- font selection + fallback chain semantics (what gets picked for a glyph),
- caching and invalidation boundaries (what keys must change),
- variable font instances (axes/coords/synthesis) correctness across shaping + rasterization.

The main text pipeline tracker remains: `docs/workstreams/text-system-v2-parley.md`.

## Goals

1) Make font selection *predictable and auditable*:
   - “Which family did we pick for this glyph, and why?”
   - “Which fallback chain was in effect (generic + script + locale)?”
2) Make variable fonts correct end-to-end:
   - shaping and rasterization must agree on instance coordinates,
   - glyph cache keys must include variable font instance identity.
3) Keep contracts portable:
   - `fret-core::FontId` remains semantic (not backend indexes),
   - wasm/bootstrap story stays deterministic (bundles + curated defaults).

## Non-goals (v1)

- Per-script fallback customization in the public settings schema (we can stage this).
- Full font feature UI (OpenType feature toggles per span) beyond an internal “plumbing is correct”.
- Multi-backend support (e.g. DirectWrite/CoreText shaping backends) — this workstream prepares the seams.

## Current implementation snapshot (Fret)

### Entry points

- Renderer text system + fallback injection + keys:
  - `crates/fret-render-wgpu/src/text/mod.rs`
  - `crates/fret-render-wgpu/src/text/parley_shaper.rs`
- Runtime globals for pickers + invalidation:
  - `crates/fret-runtime/src/font_catalog.rs`
  - `crates/fret-runtime/src/font_bootstrap.rs`
- Runner wiring (refresh catalog + apply config + bump stack key):
  - `crates/fret-launch/src/runner/desktop/mod.rs`
  - `crates/fret-launch/src/runner/web/*`

### What is already good

- Semantic `FontId` in `fret-core` (portable across runs + wasm): `crates/fret-core/src/ids.rs`.
- Single source of truth for family resolution: Parley/fontique collection (no legacy bridge).
- Cache invalidation: `TextBlobKey` includes `font_stack_key`; font config mutations bump key and reset caches.
- Variable font instance coordinates are carried end-to-end (shaping → glyph keys → Swash rasterization) with a
  deterministic fixture test:
  - `crates/fret-render-wgpu/src/text/parley_shaper.rs` (`ParleyGlyph::normalized_coords`)
  - `crates/fret-render-wgpu/src/text/mod.rs` (`variation_key_from_normalized_coords`, Swash `normalized_coords(...)`)
  - `crates/fret-render-wgpu/src/text/mod.rs` (`variable_font_weight_changes_face_key_and_raster_output`)
- Locale is plumbed into shaping to unlock fontique script+locale fallbacks on system-font builds:
  - `crates/fret-render-wgpu/src/text/mod.rs` (`TextSystem::set_text_locale`)
  - `crates/fret-launch/src/runner/desktop/app_handler.rs` (seed renderer locale from `I18nService`)
  - `crates/fret-launch/src/runner/web/gfx_init.rs` (seed renderer locale on adopt)
- Font catalog metadata is available as a runner-populated global for pickers/diagnostics:
  - `crates/fret-runtime/src/font_catalog.rs` (`FontCatalogMetadata`, `FontCatalogEntry`)
  - `crates/fret-render-wgpu/src/text/parley_shaper.rs` (`ParleyShaper::all_font_catalog_entries`)

### Known gaps (must-fix)

1) Variable fonts: synthesis + diagnostics are incomplete.
   - Normalized coords are now applied consistently, but synthesis (embolden/skew) is not yet applied to rasterization
     and is not part of `variation_key`.
   - We also lack a human-auditable representation for “axis tag + value” settings (useful for debugging and future UI).

2) Fallback chain semantics still need conformance + explicit policy composition.
   - Locale is now passed into Parley shaping, enabling fontique’s script+locale fallback on desktop when system fonts
     are present.
   - Curated/common fallbacks can be injected into both generic and named family stacks, and this is enabled by default
     for wasm/bundled-only environments. On desktop, the platform default is to prefer system fallback unless explicitly
     enabled via settings (`TextFontFamilyConfig.common_fallback_injection`).
   - Remaining work: define and test the exact composition rules (requested stack + script/locale fallback + overrides)
     with a focused mixed-script conformance suite.
   - Initial conformance evidence (UI Gallery script + screenshots/bundles):
     - `tools/diag-scripts/ui-gallery-text-bidi-font-fallback-screenshots.json`
       - Includes `render_text_missing_glyphs_is` assertions (requires diagnostics bundles).

3) Font enumeration is still uncached and metadata is best-effort.
   - `all_font_names()` and `all_font_catalog_entries()` are best-effort snapshots (platform-dependent).
   - `FontCatalogMetadata` now carries coarse metadata (axes + monospace hint), but settings UI still needs to consume it.
   - Remaining work: decide refresh policy + caching boundaries (when does the runner rescan?).

## Reference study (repo-ref)

### Zed / GPUI

Key ideas to borrow:

- Separate platform font DB (`PlatformTextSystem`) from app-level caches:
  - `repo-ref/zed/crates/gpui/src/platform.rs` (`PlatformTextSystem`)
  - `repo-ref/zed/crates/gpui/src/text_system.rs` (`TextSystem` caches for IDs/metrics/raster bounds)
- Explicit fallback stack semantics:
  - global fallback stack inside `TextSystem`,
  - optional per-style fallbacks (`FontFallbacks`) for override lists.

### Parley / Fontique

Key ideas to borrow:

- Script + locale fallback keys:
  - `repo-ref/parley/fontique/src/fallback.rs` (`FallbackKey`)
- Variable font axes and synthesis:
  - `repo-ref/parley/fontique/src/font.rs` (`AxisInfo`, `Synthesis`)
- Normalized variation coordinates per run:
  - `repo-ref/parley/parley/src/layout/run.rs` (`Run::normalized_coords`, `Run::synthesis`)
- Parley shaping uses synthesis + explicit variations to build HarfBuzz instances:
  - `repo-ref/parley/parley/src/shape/mod.rs` (variation iterator includes synthesis + style variations)

### Xilem

Key ideas to borrow:

- Treat “font lifecycles” as a first-class problem (explicit registration at driver start).
- Demonstrate variable-font usage at the UI layer (weight animation):
  - `repo-ref/xilem/xilem/examples/variable_clock.rs`
  - `repo-ref/xilem/masonry/src/widgets/variable_label.rs`

## Work items (milestones)

- M0 (implemented): variable font instance identity (coords → `variation_key`) + Swash `normalized_coords(...)` +
  deterministic fixture test.
- M1 (in progress): script + locale fallbacks + curated overrides + `TextFontStackKey` participation.
  - Done: locale plumbing to Parley shaping + cache key bumps (`TextSystem::set_text_locale`)
  - Done: configurable common-fallback injection policy (platform-default prefers system fallback on desktop, injects on
    wasm/bundled-only); supports both generic and named family stacks.
  - Next: mixed-script conformance test coverage + explicit policy “explainability” hooks (diagnostics)
- M2 (implemented, partial): picker metadata (axes + monospace hint) is available via `FontCatalogMetadata`.
  - Next: settings UI adoption + explicit refresh/invalidation policy and caching.
- M3 (optional): public shaping knobs (OpenType features + variation axes) and serialized settings.

## Options (tradeoffs)

### Variable font instance identity (M0)

Option A — Fingerprint normalized coords only:

- Pros: closest to what HarfBuzz actually used for shaping; minimal surface area.
- Cons: harder to debug (coords are not self-describing); requires knowing axis ordering for display.

Option B — Fingerprint “axis tag + value” settings (synthesis + explicit variations):

- Pros: debuggable; can be reused for settings serialization later.
- Cons: needs a stable ordering and may diverge from HarfBuzz normalized coords if rounding differs.

Recommendation:

- Use normalized coords as the primary cache key input (authoritative for correctness),
  but also keep a debug-only “settings list” representation for diagnostics logs/screenshots.

### Fallback chain integration (M1)

Option A — Keep curated generic injection only (status quo):

- Pros: simple; deterministic on wasm; easy to reason about for UI-only apps.
- Cons: ignores script+locale fallbacks that OS/fontconfig/CoreText/DirectWrite already compute; harder to align with
  upstream Parley expectations; more drift risk as we expand scripts.

Option B — Use fontique fallback keys (script + locale) and treat curated lists as overrides:

- Pros: aligns with fontique backend behavior; better coverage; fewer hard-coded family lists.
- Cons: introduces a “locale input” decision; more complex to test deterministically.

Recommendation:

- Adopt Option B and keep curated lists as overrides (especially for wasm/bundles).

### Font enumeration and picker UX (M2)

Option A — Keep `FontCatalog` as `Vec<String>` only:

- Pros: simplest; good enough for a basic family picker.
- Cons: cannot expose useful metadata (variable axes, monospace candidates); cannot build better UX (“show variable fonts”).

Option B — Introduce `FontCatalogEntry` best-effort metadata:

- Pros: enables better settings UX and diagnostics (“why was this font chosen?”).
- Cons: platform-dependent; needs clear “best-effort” wording and revisioning.

Recommendation:

- Implement Option B behind a new global type, keeping `FontCatalog` for backward compatibility.

### M0 — Variable font correctness (renderer-internal)

Exit criteria:

- Rasterization applies the same variable font instance coordinates that shaping used.
- Glyph cache keys include variable instance identity (`variation_key` is no longer hard-coded to `0`).
- At least one deterministic test covers a variable font (bundled or test fixture).

Implementation sketch:

- Extend the shaping output to carry:
  - `normalized_coords: Arc<[i16]>` (from `parley::layout::Run::normalized_coords()`),
  - `synthesis` flags (embolden/skew) if raster needs to match them.
- Derive `variation_key` as a stable hash/fingerprint of coords (+ synthesis if relevant).
- Apply coords to the Swash scaler configuration when rasterizing.
- Ensure `TextShapeKey`/`GlyphKey` incorporate the fingerprint so caches cannot alias.

### M1 — Fallback chain v2 (script + locale + curated overrides)

Exit criteria:

- Missing glyph selection uses fontique fallback families keyed by script + locale *plus* curated/common overrides.
- The effective fallback policy participates in `TextFontStackKey` (and therefore `TextBlobKey`).
- A focused conformance string/test covers mixed-script fallback (Latin + CJK + emoji + RTL).

Implementation sketch:

- Add a “text locale” input to the renderer text system (from runner/app settings or OS locale).
- When shaping selects a fallback font for a cluster, allow fontique’s `FallbackKey(script, locale)` to contribute.
- Keep `TextFontFamilyConfig.common_fallback` as an override layer (app-specific candidates appended/prepended).

### M2 — Font catalog v2 (picker metadata)

Exit criteria:

- Settings UI can show:
  - family name,
  - whether family has variable axes (wght/wdth/slnt/ital/opsz),
  - whether family is a “good monospace candidate” (best-effort),
  - (optional) scripts covered or a coarse “CJK/emoji coverage present” signal.

Implementation sketch:

- Introduce a new runtime global (or extend `FontCatalog`) to store `FontCatalogEntry` records.
- Populate it from fontique family + font metadata when available; keep it best-effort and platform-dependent.

### M3 — Public shaping knobs (features + variations) (optional)

Exit criteria:

- `TextShapingStyle` can optionally carry OpenType features and variation axes in a serialized form.
- The renderer plumbs them into Parley and includes them in shaping keys.

## Risks / decisions

- “Where does locale live?”: runner global vs per-window vs per-text input. Start with a runner/global default.
- “How stable should picker lists be?”: font enumeration is inherently platform-dependent; cache it and version it.
- “How far to go on axis UI?”: exposing arbitrary axes is powerful but can explode UX; keep an advanced/debug-only path first.
