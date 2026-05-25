# Fret Node Paint Root Cached Edge Build State Adapter v1 - TODO

Status: Closed
Last updated: 2026-05-25

## CEBS-M0 - Scope Freeze

- [x] CEBS-010 [owner=codex] [deps=none] [scope=docs/workstreams/fret-node-paint-root-cached-edge-build-state-adapter-v1]
  Goal: Open a narrow follow-on for cached edge build-state host/services/scale route inputs only.
  Validation: `python3 -m json.tool docs/workstreams/fret-node-paint-root-cached-edge-build-state-adapter-v1/WORKSTREAM.json`
  Evidence: `DESIGN.md`, `EVIDENCE_AND_GATES.md`
  Handoff: Keep edge-label build-state, temporary scenes, cache keys, and overlays out of scope.

## CEBS-M1 - Cached Edge Build-State Adapter Seam

- [x] CEBS-020 [owner=codex] [deps=CEBS-010] [scope=ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges]
  Goal: Move cached edge build-state host/services/scale route inputs behind an adapter plus
  retained `PaintCx` binding.
  Validation: `cargo test -p fret-node --features compat-retained-canvas paint_root_cached_edge_build_state_adapter`
  Evidence: `edges/single.rs`, `edges/tiled.rs`, `build_state_adapter.rs`,
  `build_state_retained_cx.rs`, `ecosystem/fret-node/src/lib.rs`
  Handoff: Cached edge build-state route inputs use the adapter seam. Edge-label build-state remains
  untouched. Complete.

## CEBS-M2 - Closeout

- [x] CEBS-030 [owner=codex] [deps=CEBS-020] [scope=docs/workstreams/fret-node-paint-root-cached-edge-build-state-adapter-v1]
  Goal: Close the lane and keep residual cached edge-label build-state route inputs as a separate
  follow-on.
  Validation: `python3 tools/check_workstream_catalog.py`; `git diff --check`
  Evidence: `CLOSEOUT_AUDIT_2026-05-25.md`
  Handoff: Start a separate follow-on for cached edge-label build-state host/services/scale route
  inputs. Complete; see `CLOSEOUT_AUDIT_2026-05-25.md`.
