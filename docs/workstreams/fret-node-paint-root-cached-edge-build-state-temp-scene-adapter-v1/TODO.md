# Fret Node Paint Root Cached Edge Build State Temp Scene Adapter v1 - TODO

Status: Closed
Last updated: 2026-05-25

## CEBTS-M0 - Scope Freeze

- [x] CEBTS-010 [owner=codex] [deps=none] [scope=docs/workstreams/fret-node-paint-root-cached-edge-build-state-temp-scene-adapter-v1]
  Goal: Open a narrow follow-on for cached edge/edge-label build-state temporary scene construction.
  Validation: `python3 -m json.tool docs/workstreams/fret-node-paint-root-cached-edge-build-state-temp-scene-adapter-v1/WORKSTREAM.json`
  Evidence: `DESIGN.md`, `EVIDENCE_AND_GATES.md`
  Handoff: Keep clip-op construction, replay sinks, cache keys, route-input adapters, and overlays
  out of scope.

## CEBTS-M1 - Temp Scene Helper Seam

- [x] CEBTS-020 [owner=codex] [deps=CEBTS-010] [scope=ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges]
  Goal: Move cached edge and edge-label build-state temporary scene construction behind a named
  helper owned by build-state stepping.
  Validation: `cargo test -p fret-node --features compat-retained-canvas paint_root_cached_edge_build_state_temp_scene_adapter`
  Evidence: `build_state/temp_scene.rs`, `build_state/step.rs`, `edges/single.rs`,
  `edges/tiled.rs`, `labels/single.rs`, `labels/tiled.rs`, `ecosystem/fret-node/src/lib.rs`
  Handoff: Route helpers no longer allocate temporary scenes. Clip-op merge policy remains
  untouched. Complete.

## CEBTS-M2 - Closeout

- [x] CEBTS-030 [owner=codex] [deps=CEBTS-020] [scope=docs/workstreams/fret-node-paint-root-cached-edge-build-state-temp-scene-adapter-v1]
  Goal: Close the lane and keep cache-local clip-op construction/merge cleanup as a separate
  follow-on.
  Validation: `python3 tools/check_workstream_catalog.py`; `git diff --check`
  Evidence: `CLOSEOUT_AUDIT_2026-05-25.md`
  Handoff: Start a separate follow-on for cache-local clip-op construction or another
  operation-family slice. Complete; see `CLOSEOUT_AUDIT_2026-05-25.md`.
