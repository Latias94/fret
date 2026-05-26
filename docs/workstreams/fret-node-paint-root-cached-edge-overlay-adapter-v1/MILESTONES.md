# Fret Node Paint Root Cached Edge Overlay Adapter v1 - Milestones

Status: Closed
Last updated: 2026-05-25

## M0 - Scope Freeze

- The lane owns selected/hovered edge overlay route ownership in cached edge paths only.
- Anchor target, fallback uncached rendering, replay, cache keys, and build-state helpers are
  explicitly out of scope.
- `WORKSTREAM.json` validates as JSON.

## M1 - Overlay Adapter

- A named cached edge overlay adapter exists under `cached_edges/`.
- The retained `PaintCx` binding owns the direct `paint_edge_overlays_selected_hovered` call.
- `single_rect.rs` and `tile_path.rs` call the adapter.
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

Result (2026-05-25): complete.
