# Fearless Architecture Convergence v1 - Evidence and Gates

Status: Active
Last updated: 2026-06-30

## Canonical Gates

- `cargo test -p fret-ui retained_widget_authoring_exports_are_compat_feature_gated`
  - Proves retained widget authoring exports stay behind `compat-retained-widgets`.
- `cargo check -p fret-ui`
  - Proves the runtime crate compiles with default features after the public surface shrink.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Proves the explicit node compatibility island still compiles.
- `python3 tools/check_layering.py`
  - Proves crate boundary policy still passes.
- `python3 tools/check_workstream_catalog.py`
  - Proves new workstream directories are indexed.
- `git diff --check`
  - Proves touched files have no whitespace errors.

## Planned Gates

- `python3 tools/check_surface_policy.py`
  - Will prove source responsibility policy after FAC-100 lands.

## Evidence Anchors

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

- Pending for the implementation slice:
  - `python3 tools/check_layering.py`
  - `python3 tools/check_workstream_catalog.py`
  - `git diff --check`

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
