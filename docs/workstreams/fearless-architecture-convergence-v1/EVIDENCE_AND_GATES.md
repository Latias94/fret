# Fearless Architecture Convergence v1 - Evidence and Gates

Status: Closed
Last updated: 2026-07-02

## Canonical Gates

- `cargo fmt --all --check`
  - Proves touched Rust files remain formatted.
- `cargo check -p fret-ui`
  - Proves the runtime crate compiles with default features after the public surface shrink.
- `python3 tools/check_layering.py`
  - Proves crate boundary policy still passes.
- `python3 tools/check_surface_policy.py`
  - Proves default app surfaces, advanced/manual classifications, and `fret-ui` mechanism root
    vocabulary stay within the convergence policy.
- `python3 tools/check_consumption_profiles.py`
  - Proves contracts-only, UI substrate, manual assembly, default app facade, batteries, bootstrap,
    and launch consumption profiles compile.
- `python3 tools/check_workstream_catalog.py`
  - Proves new workstream directories are indexed.
- `python3 tools/perf/audit_perf_baselines.py --matrix docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md --strict`
  - Proves checked-in perf baselines still match required fields and thresholds.
- `python3 tools/perf/diag_u8_text_budget_gate.py --skip-native --web-export-bundle target/fret-diag-u8-web-export-code-editor-r3/1782959381479-bundle/bundle.json --out-dir target/fret-diag-u8-web-budget-r3 --out-report target/fret-diag-u8-web-budget-r3/summary.json`
  - Proves the U8 web bundle has bounded text/glyph budget fields.
- `git diff --check`
  - Proves touched files have no whitespace errors.

## Evidence Anchors

- `docs/workstreams/fearless-architecture-convergence-v1/CLOSEOUT_AUDIT_2026-07-02.md`
- `docs/workstreams/fearless-architecture-convergence-v1/DESIGN.md`
- `docs/workstreams/fearless-architecture-convergence-v1/TODO.md`
- `docs/workstreams/retained-public-surface-exit-v1/DESIGN.md`
- `docs/adr/0330-retained-runtime-internal-and-compat-surface.md`
- `crates/fret-ui/src/lib.rs`
- `ecosystem/fret-node/Cargo.toml`
- `ecosystem/fret-node/src/lib.rs`
- `docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md`
- `docs/golden-architecture.md`
- `docs/runtime-contract-matrix.md`
- `docs/ui-closure-map.md`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
- `tools/check_surface_policy.py`
- `tools/check_consumption_profiles.py`
- `tools/perf/diag_u8_text_budget_gate.py`
- `docs/knowledge/engineering/current-state.md`

## 2026-07-02 - 2026 UI framework convergence closeout

Claim to verify:

- U1-U9 of the implementation-ready convergence plan are implemented or explicitly deferred with
  owner/reason/gate evidence.
- Runtime, renderer, text, facade, and policy diagnostics now expose the pressure signals needed to
  continue narrow follow-ons without guessing.
- This coordinator can close without hiding retained compatibility paths.

Fresh validation:

- Passed during the final implementation slices:
  - `cargo fmt --all --check`
  - `cargo check -p fret-ui-gallery-web --target wasm32-unknown-unknown --features gallery-dev`
  - `cargo check -p fret-render-wgpu --target wasm32-unknown-unknown --lib`
  - `cargo check -p fret-render-wgpu --lib`
  - `cargo check -p fret-ui --lib`
  - focused `cargo nextest run -p fret-ui` timer/identity/dispatch/view-boundary gates
  - focused `cargo nextest run -p fret-render-wgpu` WGSL, scene-chunk, resident-upload, and text
    budget gates
  - `python3 tools/check_layering.py`
  - `python3 tools/check_surface_policy.py`
  - `python3 tools/check_consumption_profiles.py`
  - `python3 tools/perf/audit_perf_baselines.py --matrix docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md --strict`
  - `git diff --check`

Resolved aggregate blocker:

- The duplicate ADR ID `0324` blocker was resolved after this closeout by preserving the older
  window input hit-testing ADR at `0324` and renumbering the later a11y state-description ADR to
  `0332`. `python3 tools/check_adr_numbers.py` and the skip-heavy pre-release smoke pass.

## 2026-06-30 - UI framework convergence contract freeze

Claim to verify:

- The current convergence plan has one owner map for default app authoring, runtime mechanisms,
  interaction policy, dirty views, frame boundaries, renderer chunks, text/glyph budgets, and
  consumption profiles.
- ADR 0066, ADR 0165, and ADR 0327 name the current breakable/refactor direction without reopening
  closed broad lanes.
- Remaining implementation-heavy work is delegated to narrow plan units and gates, not hidden in
  this coordinator.

Fresh validation:

- Completed by `CLOSEOUT_AUDIT_2026-07-02.md`.

## 2026-05-25 - Retained public surface first slice

Claim to verify:

- The retained runtime remains available internally.
- `Widget` and retained `*Cx` authoring types are no longer default root exports from `fret-ui`.
- `Invalidation` and `CommandAvailability` remain default public mechanism data types.
- `fret-node/compat-retained-canvas` explicitly enables `fret-ui/compat-retained-widgets`.

Fresh validation:

- Passed on 2026-05-25:
  - `python3 -m json.tool` for all seven new `WORKSTREAM.json` files
  - `python3 tools/check_workstream_catalog.py`
  - `cargo fmt --package fret-ui --package fret-node --check`
  - `cargo nextest run -p fret-ui retained_widget_authoring_exports_are_compat_feature_gated`
  - `cargo test -p fret-ui retained_widget_authoring_exports_are_compat_feature_gated`
  - `cargo check -p fret-ui`
  - `cargo check -p fret-node --features compat-retained-canvas`
  - `python3 tools/check_layering.py`
  - `git diff --check`

Notes:

- `cargo check -p fret-ui` emitted existing warnings for unexpected cfg
  `unstable-retained-bridge` in `crates/fret-ui/src/tree/layout/clean_geometry.rs` and dead code
  `current_effective_opacity`.
