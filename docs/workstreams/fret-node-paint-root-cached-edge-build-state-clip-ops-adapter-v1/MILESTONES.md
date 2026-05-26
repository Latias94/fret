# Fret Node Paint Root Cached Edge Build State Clip Ops Adapter v1 - Milestones

Status: Closed
Last updated: 2026-05-25

## M0 - Scope Freeze

- The lane owns cache-local clip stack construction and temp-op merge policy only.
- Temporary scene construction, replay sinks, cache keys, route-input adapters, and overlays are
  explicitly out of scope.
- `WORKSTREAM.json` validates as JSON.

## M1 - Clip Ops Helper

- A named clip ops helper exists under `cached_edges/build_state/`.
- `build_state/ops.rs` delegates initial clip construction and temp-op merging.
- `build_state/ops.rs` no longer mentions `SceneOp::PushClipRect` or `SceneOp::PopClip`.
- Focused source-policy coverage locks the seam.

## M2 - Verification And Closeout

- `cargo fmt --package fret-node` passes.
- Focused source-policy test passes under `compat-retained-canvas`.
- `cargo check -p fret-node` passes.
- `cargo check -p fret-node --features compat-retained-canvas` passes.
- `python3 tools/check_workstream_catalog.py` passes.
- `python3 tools/check_layering.py` passes.
- `git diff --check` passes.
- A closeout audit records shipped state and residual follow-ons.
