# Fret Node Paint Root Cached Edge Build State Temp Scene Adapter v1 - Milestones

Status: Closed
Last updated: 2026-05-25

## M0 - Scope Freeze

- The lane owns temporary scene construction for cached edge and edge-label build-state stepping.
- Clip-op construction, replay sinks, cache keys, route-input adapters, and overlays are explicitly
  out of scope.
- `WORKSTREAM.json` validates as JSON.

## M1 - Temp Scene Helper

- A named temporary scene helper exists under `cached_edges/build_state/`.
- Build-state step helpers own temporary scene construction.
- Edge and edge-label route helpers no longer mention `fret_core::Scene::default()`, `Scene::default`,
  or local `tmp` scene allocation.
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
