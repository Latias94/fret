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
- Facade posture after `FR-RENDER-TEXT-011`:
  - `src/lib.rs` now exposes an explicit root facade (`pub use`) while internal modules stay
    crate-private, which removes direct downstream dependence on module layout.
- Remaining surface risk to review:
  - root `pub use` coverage is still broad, so future changes should audit which items truly belong
    in the stable text-engine facade versus renderer-internal helper space.
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

- Font DB, catalog extraction, rescan seed/apply, injected-font retention, baseline-metrics cache
  - Files: `crates/fret-render-text/src/parley_font_db.rs`
- Parley shaping entrypoints, style translation, line-metrics computation, layout hand-off
  - Files: `crates/fret-render-text/src/parley_shaper.rs`
- Fallback policy composition and diagnostics snapshot
  - Files: `crates/fret-render-text/src/fallback_policy.rs`
- Wrapper helper seams now split by responsibility
  - Width-balancing helper: `crates/fret-render-text/src/wrapper_balance.rs`
  - Boundary / grapheme cut / hit-testing helpers: `crates/fret-render-text/src/wrapper_boundaries.rs`
  - Paragraph + newline assembly and ellipsis handling: `crates/fret-render-text/src/wrapper_paragraphs.rs`
  - Range slicing + single-line shaping helpers: `crates/fret-render-text/src/wrapper_slices.rs`
  - Paragraph range wrapping helpers: `crates/fret-render-text/src/wrapper_ranges.rs`
- Remaining top-level layout adaptation dispatcher
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

- Facade drift at the crate root
  - Failure mode: downstream crates start depending on incidental root re-exports, making future
    ownership cleanup harder even though module paths are now private.
  - Existing gates: `cargo check -p fret-render-text -p fret-render-wgpu -p fret-typography-real-shaping-gates`.
  - Missing gate to add: a surface review or `public_api` snapshot before trimming root re-exports.
- Residual ownership concentration around `ParleyShaper`
  - Failure mode: a shaping refactor still regresses baseline-metrics caching or font-context
    orchestration even after the font DB/catalog/rescan state moved into `parley_font_db.rs`.
  - Existing gates: `registered_font_blobs_dedup_and_lru_eviction_by_count`,
    `registered_font_blobs_eviction_by_bytes_budget`,
    `rescan_apply_returns_false_when_environment_is_unchanged`.
  - Missing gate to add: crate-local tests for baseline-metrics cache invalidation boundaries after
    catalog refresh or injected-font changes.
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
  - Existing gates: cache presence diagnostics via `font_db_diagnostics_snapshot` plus the
    crate-local build-count harness
    `font_catalog_cached_reads_do_not_rebuild_entries_until_invalidated`.
  - Missing gate to add: optional higher-level perf evidence if catalog enumeration ever becomes a
    user-visible startup bottleneck in real apps.
- `wrapper.rs` as a second "god module"
  - Failure mode: line layout, wrapping, metrics, and hit testing regress together because there is
    no smaller ownership boundary.
  - Existing gates: crate tests exist, and helper seams now isolate balancing, boundary logic,
    ellipsis/paragraph assembly, range slicing, and paragraph range wrapping from the top-level
    dispatcher.
  - Missing gate to add: split module-level tests aligned to the new subdomains so helper ownership
    is reflected in the test layout as well.

## 6) Code quality findings (Rust best practices)

- Positive: the crate keeps backend dependencies out and now has a clearer internal seam:
  `ParleyFontDbState` isolates catalog caches, blob retention, and rescan replay from the shaping
  entrypoints.
- Positive: `wrapper.rs` is no longer the sole owner of all wrapping helpers; boundary math, slice
  shaping, balancing, paragraph assembly, and range wrapping have moved into dedicated internal
  modules.
- The main remaining maintainability risk is responsibility concentration:
  - `ParleyShaper` still owns shaping, locale/fallback inputs, and baseline-metrics orchestration.
  - `wrapper.rs` still carries the top-level public entrypoints and its large in-file test module.
- No obvious `unsafe` usage was observed in the audited entry points.
- The fallback-policy contract is strong, but much of its regression coverage currently lives in
  `fret-render-wgpu`, which makes renderer-independent refactors slower to validate.

Evidence anchors:

- `crates/fret-render-text/src/parley_font_db.rs` (`ParleyFontDbState`,
  `all_font_catalog_entries`, `catalog_entries_build_count`, `apply_system_font_rescan_result`,
  `run_system_font_rescan`)
- `crates/fret-render-text/src/parley_shaper.rs` (`font_db_diagnostics_snapshot`,
  `base_ascent_descent_px_for_style`,
  `font_catalog_cached_reads_do_not_rebuild_entries_until_invalidated`)
- `crates/fret-render-text/src/wrapper_boundaries.rs` (`hit_test_x`,
  `clamp_to_grapheme_boundary_down`)
- `crates/fret-render-text/src/wrapper_balance.rs` (`balanced_word_wrap_width_px`)
- `crates/fret-render-text/src/wrapper_paragraphs.rs` (`wrap_with_newlines`,
  `wrap_none_ellipsis`)
- `crates/fret-render-text/src/wrapper_slices.rs` (`shape_slice`,
  `shape_slice_measure_only`, `slice_spans`)
- `crates/fret-render-text/src/wrapper_ranges.rs` (`wrap_grapheme_range`,
  `wrap_word_range_measure_only`)
- `crates/fret-render-text/src/fallback_policy.rs` (`TextFallbackPolicyV1`,
  `diagnostics_snapshot`)
- `crates/fret-render-wgpu/src/text/tests.rs`

## 7) Recommended refactor steps (small, gated)

1. Separate fallback-policy tests from renderer-backend tests by adding crate-local key/snapshot
   coverage — outcome: portable refactors do not need `fret-render-wgpu` to validate policy logic —
   gate: `cargo nextest run -p fret-render-text`.
2. Consider moving the large `wrapper.rs` test module into helper-aligned test files once the team
   wants review diffs to mirror the new internal seams — outcome: implementation ownership and test
   ownership match — gate: `cargo nextest run -p fret-render-text`.

## 8) Open questions / decisions needed

- Should `FontCatalogEntryMetadata` remain renderer-owned, or should a stable subset move into
  `fret-runtime` so launch/runtime do not depend on renderer-specific structure choices?
- Do we want `wrapper.rs` to remain in `fret-render-text`, or is there a future split between pure
  shaping/fallback and higher-level layout adaptation?
