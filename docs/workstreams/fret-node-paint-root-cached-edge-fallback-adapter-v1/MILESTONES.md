# Fret Node Paint Root Cached Edge Fallback Adapter v1 - Milestones

Status: Closed
Last updated: 2026-05-25

## M0 - Scope Freeze

- The lane owns cached edge fallback route ownership only.
- Cache keys, replay, overlay, anchor target, build-state helpers, and deeper edge-paint internals
  are explicitly out of scope.
- `WORKSTREAM.json` validates as JSON.

## M1 - Fallback Adapter

- A named cached edge fallback adapter exists under `cached_edges/`.
- The retained `PaintCx` binding owns retained host access and direct edge paint dispatch.
- `cached_edges/fallback.rs` and `cached_edges/edges/fallback.rs` call the adapter.
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
