# Fret Node Paint Root Cached Edge Anchor Target Adapter v1 - Milestones

Status: Closed
Last updated: 2026-05-25

## M0 - Scope Freeze

- The lane owns cached edge anchor target route ownership only.
- Fallback, overlay, replay, cache keys, build-state helpers, and deeper edge-anchor internals are
  explicitly out of scope.
- `WORKSTREAM.json` validates as JSON.

## M1 - Anchor Target Adapter

- A named cached edge anchor target adapter exists under `cached_edges/`.
- The retained `PaintCx` binding owns the direct shared edge-anchor helper calls.
- `anchor_target.rs` calls the adapter.
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
