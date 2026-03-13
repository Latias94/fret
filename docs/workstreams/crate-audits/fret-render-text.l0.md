# Crate audit (L0) — `fret-render-text`

## Crate

- Name: `fret-render-text`
- Path: `crates/fret-render-text`
- Owners / adjacent crates: `fret-core`, `fret-fonts`, `fret-render-wgpu`, `fret-launch`
- Current layer: renderer-owned text shaping, fallback policy, catalog extraction, wrapping

## 1) Purpose (what this crate *is*)

- The renderer-owned text engine built on Parley: shaping, wrapping, metrics, fallback policy, font
  catalog extraction, and system-font rescan support.
- It should own font selection/fallback mechanics and cache-key inputs, while staying portable and
  free of backend glue (`wgpu`, `winit`, browser APIs).
- This crate currently serves both as a core text engine and as a utility surface for higher-level
  renderer crates, which is why public-surface discipline matters here.

Evidence anchors:

- `crates/fret-render-text/Cargo.toml`
- `crates/fret-render-text/src/lib.rs`

## 2) Public contract surface

- Key exports / stable types:
  - `FontCatalogEntryMetadata`
  - `FontVariableAxisMetadata`
  - `SystemFontRescanSeed`
  - `SystemFontRescanResult`
  - `effective_text_scale_factor`
- "Accidental" exports to consider removing:
  - `src/lib.rs` currently exposes 15 `pub mod` entries, which makes most internal modules
    reachable by downstream crates even when they are implementation details.
- Feature flags and intent:
  - no crate-level feature matrix beyond dev-time font bundle expansion in tests, which is good for
    portability but means public modules do most of the contract work.

Evidence anchors:

- `crates/fret-render-text/src/lib.rs`
- `python tools/audit_crate.py --crate fret-render-text`

## 3) Dependency posture

- Backend coupling risks:
  - no `wgpu`, `winit`, `web-sys`, or runner dependencies; the crate stays on the portable side of
    the text/render boundary.
- Layering policy compliance:
  - good; it depends only on `fret-core`, `fret-fonts`, and text-related third-party crates.
- Compile-time / maintenance hotspots:
  - `wrapper.rs` (~2675 LOC), `parley_shaper.rs` (~2209 LOC), and `geometry.rs` (~2105 LOC) are
    the dominant refactor risk zones.

Evidence anchors:

- `crates/fret-render-text/Cargo.toml`
- `crates/fret-render-text/src/lib.rs`
- `python tools/audit_crate.py --crate fret-render-text`

## 4) Module ownership map (internal seams)

- Font DB, catalog, rescan, injected-font retention, shaping entrypoints
  - Files: `crates/fret-render-text/src/parley_shaper.rs`
- Fallback policy composition and diagnostics snapshot
  - Files: `crates/fret-render-text/src/fallback_policy.rs`
- Layout adaptation / wrapping / hit testing
  - Files: `crates/fret-render-text/src/wrapper.rs`, `crates/fret-render-text/src/measure.rs`,
    `crates/fret-render-text/src/prepare_layout.rs`
- Geometry and decoration translation
  - Files: `crates/fret-render-text/src/geometry.rs`,
    `crates/fret-render-text/src/decorations.rs`
- Cache-key inputs and trace output
  - Files: `crates/fret-render-text/src/cache_keys.rs`,
    `crates/fret-render-text/src/font_instance_key.rs`,
    `crates/fret-render-text/src/font_trace.rs`

## 5) Refactor hazards (what can regress easily)

- Public-surface drift via `pub mod`
  - Failure mode: downstream crates start depending on internals from `wrapper`, `fallback_policy`,
    or `parley_shaper`, making future module moves painful.
  - Existing gates: none focused on API surface.
  - Missing gate to add: a surface review or `public_api` snapshot before shrinking exports.
- Mixed ownership inside `parley_shaper.rs`
  - Failure mode: a refactor in shaping accidentally regresses catalog caches, rescan replay, or
    injected-font retention because all concerns share one module.
  - Existing gates: `rescan_apply_returns_false_when_environment_is_unchanged`.
  - Missing gate to add: crate-local tests for cache invalidation boundaries after catalog refresh
    and locale/fallback updates.
- Fallback policy / cache-key invalidation ordering
  - Failure mode: locale or injection-mode changes do not propagate into the effective fallback
    policy key, causing stale layout or glyph reuse.
  - Existing gates: renderer-side tests
    `bundled_only_defaults_use_profile_ui_family_when_config_is_empty`,
    `fallback_policy_snapshot_reports_profile_contract_and_defaults`,
    `ui_generic_resolution_prefers_first_available_configured_candidate`.
  - Missing gate to add: direct `fret-render-text` tests that assert policy-key changes without
    needing the full renderer harness.
- Catalog extraction cost on the foreground path
  - Failure mode: `all_font_catalog_entries()` and related metadata probes become unexpectedly
    expensive on startup or after rescans.
  - Existing gates: cache presence diagnostics via `font_db_diagnostics_snapshot`.
  - Missing gate to add: a bounded perf/regression harness for repeated catalog enumeration.
- `wrapper.rs` as a second "god module"
  - Failure mode: line layout, wrapping, metrics, and hit testing regress together because there is
    no smaller ownership boundary.
  - Existing gates: crate tests exist, but the ownership seam is still broad.
  - Missing gate to add: split module-level tests aligned to subdomains after extraction.

## 6) Code quality findings (Rust best practices)

- Positive: the crate keeps backend dependencies out and already exposes useful diagnostics for font
  DB cache state.
- The main maintainability risk is responsibility concentration:
  - `ParleyShaper` owns shaping, font DB state, catalog caches, blob retention, locale, and rescan
    replay.
  - `wrapper.rs` owns too much post-shaping behavior.
- No obvious `unsafe` usage was observed in the audited entry points.
- The fallback-policy contract is strong, but much of its regression coverage currently lives in
  `fret-render-wgpu`, which makes renderer-independent refactors slower to validate.

Evidence anchors:

- `crates/fret-render-text/src/parley_shaper.rs` (`font_db_diagnostics_snapshot`,
  `record_registered_font_blob`, `all_font_catalog_entries`, `apply_system_font_rescan_result`)
- `crates/fret-render-text/src/fallback_policy.rs` (`TextFallbackPolicyV1`,
  `diagnostics_snapshot`)
- `crates/fret-render-wgpu/src/text/tests.rs`

## 7) Recommended refactor steps (small, gated)

1. Extract a `font_db/` ownership seam from `parley_shaper.rs` (catalog, caches, rescan seed/apply,
   injected-font retention) — outcome: shaping code stops sharing a god module with font DB state —
   gate: `cargo nextest run -p fret-render-text`.
2. Reduce `src/lib.rs` to an explicit facade and make internal modules crate-private where possible
   — outcome: a smaller accidental API surface — gate: `cargo check -p fret-render-text` plus any
   caller fixes.
3. Separate fallback-policy tests from renderer-backend tests by adding crate-local key/snapshot
   coverage — outcome: portable refactors do not need `fret-render-wgpu` to validate policy logic —
   gate: `cargo nextest run -p fret-render-text`.
4. Split `wrapper.rs` into smaller submodules (`line_breaking`, `hit_test`, `metrics`, `selection`)
   — outcome: smaller diffs and clearer ownership — gate: `cargo nextest run -p fret-render-text`.

## 8) Open questions / decisions needed

- Should `FontCatalogEntryMetadata` remain renderer-owned, or should a stable subset move into
  `fret-runtime` so launch/runtime do not depend on renderer-specific structure choices?
- Do we want `wrapper.rs` to remain in `fret-render-text`, or is there a future split between pure
  shaping/fallback and higher-level layout adaptation?
