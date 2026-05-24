# Retained Public Surface Exit v1 - Evidence and Gates

Status: Active
Last updated: 2026-05-25

## Canonical Gates

- `cargo test -p fret-ui retained_widget_authoring_exports_are_compat_feature_gated`
  - Proves retained authoring root exports are compatibility-gated while shared mechanism data
    remains public.
- `cargo check -p fret-ui`
  - Proves default `fret-ui` compiles after the root surface shrink.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Proves the explicit node compatibility island compiles with `fret-ui/compat-retained-widgets`.
- `python3 tools/check_layering.py`
  - Proves feature/dependency layering still passes.
- `python3 tools/check_workstream_catalog.py`
  - Proves this new workstream is indexed.
- `git diff --check`
  - Proves no whitespace errors in touched files.

## Evidence Anchors

- `docs/adr/0330-retained-runtime-internal-and-compat-surface.md`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
- `crates/fret-ui/src/lib.rs`
- `ecosystem/fret-node/Cargo.toml`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-public-surface-exit-v1/DESIGN.md`

## 2026-05-25 - Root export compatibility gate

Claim to verify:

- `fret-ui` retains the runtime implementation but stops exporting retained authoring types by
  default.
- `fret-node` retained canvas compatibility is now an explicit two-feature edge:
  `compat-retained-canvas -> fret-ui -> fret-ui/compat-retained-widgets`.

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
