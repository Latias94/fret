# Fret Node Low-Level Adapter v1 - Evidence and Gates

Status: Active
Last updated: 2026-05-25

## Canonical Gates

- `cargo check -p fret-node`
- `cargo check -p fret-node --features compat-retained-canvas`
- `cargo test -p fret-node --features compat-retained-canvas retained_compatibility_surface_stays_declarative_only`
- `python3 tools/check_layering.py`

## Evidence Anchors

- `ecosystem/fret-node/Cargo.toml`
- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/low_level_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_low_level_adapter.rs`
- `docs/workstreams/retained-public-surface-exit-v1/EVIDENCE_AND_GATES.md`

## 2026-05-25 - NLA-010/NLA-020 first low-level adapter seam

Claim to verify:

- Retained context usage inside `ecosystem/fret-node/src/ui/canvas/widget/**` is audited enough to
  select the first adapter seam.
- The common redraw / paint invalidation / handled / pointer-capture release operations live behind
  `low_level_adapter.rs`.
- Retained `EventCx`, `CommandCx`, `LayoutCx`, and `PaintCx` bindings for this seam are isolated in
  `retained_low_level_adapter.rs`.
- Default `fret-node` stays off retained authoring while `compat-retained-canvas` still compiles.

Fresh validation:

- Passed on 2026-05-25:
  - `python3 -m json.tool docs/workstreams/fret-node-low-level-adapter-v1/WORKSTREAM.json`
  - `cargo fmt --package fret-node --check`
  - `cargo check -p fret-node`
  - `cargo check -p fret-node --features compat-retained-canvas`
  - `cargo test -p fret-node --features compat-retained-canvas retained_compatibility_surface_stays_declarative_only`
  - `cargo test -p fret-node --features compat-retained-canvas retained_canvas_low_level_adapter_policy_helpers_stay_off_retained_bridge`
  - `cargo test -p fret-node --features compat-retained-canvas low_level_adapter`
  - `python3 tools/check_layering.py`
  - `git diff --check`

Notes:

- `cargo check` / `cargo test` still emit existing `fret-ui` warnings for unexpected cfg
  `unstable-retained-bridge` in `crates/fret-ui/src/tree/layout/clean_geometry.rs` and dead code
  `current_effective_opacity`.
