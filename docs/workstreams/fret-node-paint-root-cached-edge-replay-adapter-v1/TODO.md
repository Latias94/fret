# Fret Node Paint Root Cached Edge Replay Adapter v1 - TODO

Status: Closed
Last updated: 2026-05-25

## CEA-M0 - Scope Freeze

- [x] CEA-010 [owner=codex] [deps=none] [scope=docs/workstreams/fret-node-paint-root-cached-edge-replay-adapter-v1]
  Goal: Open a narrow follow-on for cached edge and edge-label replay scene sinks only.
  Validation: `python3 -m json.tool docs/workstreams/fret-node-paint-root-cached-edge-replay-adapter-v1/WORKSTREAM.json`
  Evidence: `DESIGN.md`, `EVIDENCE_AND_GATES.md`
  Handoff: Keep build-state route inputs, temporary scenes, cache keys, and overlays out of scope.

## CEA-M1 - Cached Edge Replay Adapter Seam

- [x] CEA-020 [owner=codex] [deps=CEA-010] [scope=ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges]
  Goal: Move cached edge and edge-label replay scene sink access behind an adapter plus retained
  `PaintCx` binding.
  Validation: `cargo test -p fret-node --features compat-retained-canvas paint_root_cached_edge_replay_adapter`
  Evidence: `edges/replay.rs`, `labels/replay.rs`, `replay_adapter.rs`, `replay_retained_cx.rs`,
  `ecosystem/fret-node/src/lib.rs`
  Handoff: Cached edge and edge-label replay use the adapter seam. Edge build-state route inputs
  remain untouched. Complete.

## CEA-M2 - Closeout

- [x] CEA-030 [owner=codex] [deps=CEA-020] [scope=docs/workstreams/fret-node-paint-root-cached-edge-replay-adapter-v1]
  Goal: Close the lane and split residual cached edge build-state route inputs into follow-ons.
  Validation: `python3 tools/check_workstream_catalog.py`; `git diff --check`
  Evidence: `CLOSEOUT_AUDIT_2026-05-25.md`
  Handoff: Start a separate follow-on for cached edge build-state host/services/scale route inputs.
  Complete; see `CLOSEOUT_AUDIT_2026-05-25.md`.
