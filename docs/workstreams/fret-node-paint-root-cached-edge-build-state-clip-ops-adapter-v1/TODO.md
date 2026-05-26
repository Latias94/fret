# Fret Node Paint Root Cached Edge Build State Clip Ops Adapter v1 - TODO

Status: Closed
Last updated: 2026-05-25

## CEBCO-M0 - Scope Freeze

- [x] CEBCO-010 [owner=codex] [deps=none] [scope=docs/workstreams/fret-node-paint-root-cached-edge-build-state-clip-ops-adapter-v1]
  Goal: Open a narrow follow-on for cached edge build-state cache-local clip-op construction and
  temp-op merge policy.
  Validation: `python3 -m json.tool docs/workstreams/fret-node-paint-root-cached-edge-build-state-clip-ops-adapter-v1/WORKSTREAM.json`
  Evidence: `DESIGN.md`, `EVIDENCE_AND_GATES.md`
  Handoff: Keep temporary scene construction, replay sinks, cache keys, route-input adapters, and
  overlays out of scope.

## CEBCO-M1 - Clip Ops Helper Seam

- [x] CEBCO-020 [owner=codex] [deps=CEBCO-010] [scope=ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/build_state]
  Goal: Move cache-local clip stack construction and temp-op merge policy behind a named helper
  owned by `build_state/clip_ops.rs`.
  Validation: `cargo test -p fret-node --features compat-retained-canvas paint_root_cached_edge_build_state_clip_ops_adapter`
  Evidence: `build_state/clip_ops.rs`, `build_state/ops.rs`, `ecosystem/fret-node/src/lib.rs`
  Handoff: `ops.rs` keeps completion bookkeeping and delegates clip policy. Complete once source
  policy passes. Complete.

## CEBCO-M2 - Closeout

- [x] CEBCO-030 [owner=codex] [deps=CEBCO-020] [scope=docs/workstreams/fret-node-paint-root-cached-edge-build-state-clip-ops-adapter-v1]
  Goal: Close the lane and keep replay/cache-key/overlay cleanup as separate follow-ons.
  Validation: `python3 tools/check_workstream_catalog.py`; `git diff --check`
  Evidence: `CLOSEOUT_AUDIT_2026-05-25.md`
  Handoff: Start a separate follow-on for overlay routing or another operation-family slice.
  Complete; see `CLOSEOUT_AUDIT_2026-05-25.md`.
