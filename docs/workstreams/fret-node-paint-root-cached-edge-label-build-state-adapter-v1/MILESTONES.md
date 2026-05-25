# Fret Node Paint Root Cached Edge Label Build State Adapter v1 - Milestones

Status: Closed
Last updated: 2026-05-25

## M0 - Scope Freeze

- The lane owns cached edge-label build-state route inputs only.
- Edge build-state, replay sinks, temporary scenes, cache keys, clip ops, and overlays are explicitly
  out of scope.
- `WORKSTREAM.json` validates as JSON.

## M1 - Adapter Seam

- A cached edge-label build-state adapter exists under `cached_edges/`.
- The retained `PaintCx` binding owns `app`, `services`, and `scale_factor` field reads.
- `labels/single.rs` and `labels/tiled.rs` route edge-label build-state host/services/scale inputs
  through the adapter.
- Focused source-policy coverage locks the seam and confirms the lane did not reopen edge
  build-state.

## M2 - Verification And Closeout

- `cargo fmt --package fret-node` passes.
- Focused source-policy test passes under `compat-retained-canvas`.
- `cargo check -p fret-node` passes.
- `cargo check -p fret-node --features compat-retained-canvas` passes.
- `python3 tools/check_workstream_catalog.py` passes.
- `python3 tools/check_layering.py` passes.
- `git diff --check` passes.
- A closeout audit records shipped state and residual follow-ons.
