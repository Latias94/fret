# Fret Node Paint Root Cached Edge Label Build State Adapter v1 - TODO

Status: Closed
Last updated: 2026-05-25

## CELBS-M0 - Scope Freeze

- [x] CELBS-010 [owner=codex] [deps=none] [scope=docs/workstreams/fret-node-paint-root-cached-edge-label-build-state-adapter-v1]
  Goal: Open a narrow follow-on for cached edge-label build-state host/services/scale route inputs
  only.
  Validation: `python3 -m json.tool docs/workstreams/fret-node-paint-root-cached-edge-label-build-state-adapter-v1/WORKSTREAM.json`
  Evidence: `DESIGN.md`, `EVIDENCE_AND_GATES.md`
  Handoff: Keep edge build-state, replay sinks, temporary scenes, cache keys, and overlays out of
  scope.

## CELBS-M1 - Cached Edge-Label Build-State Adapter Seam

- [x] CELBS-020 [owner=codex] [deps=CELBS-010] [scope=ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges]
  Goal: Move cached edge-label build-state host/services/scale route inputs behind an adapter plus
  retained `PaintCx` binding.
  Validation: `cargo test -p fret-node --features compat-retained-canvas paint_root_cached_edge_label_build_state_adapter`
  Evidence: `labels/single.rs`, `labels/tiled.rs`, `label_build_state_adapter.rs`,
  `label_build_state_retained_cx.rs`, `ecosystem/fret-node/src/lib.rs`
  Handoff: Cached edge-label build-state route inputs use the adapter seam. Edge build-state remains
  untouched. Complete.

## CELBS-M2 - Closeout

- [x] CELBS-030 [owner=codex] [deps=CELBS-020] [scope=docs/workstreams/fret-node-paint-root-cached-edge-label-build-state-adapter-v1]
  Goal: Close the lane and keep residual cache-local temporary scene, clip-op, and overlay cleanup as
  separate follow-ons.
  Validation: `python3 tools/check_workstream_catalog.py`; `git diff --check`
  Evidence: `CLOSEOUT_AUDIT_2026-05-25.md`
  Handoff: Start a separate follow-on for cache-local temporary scene construction or another
  operation-family slice. Complete; see `CLOSEOUT_AUDIT_2026-05-25.md`.
