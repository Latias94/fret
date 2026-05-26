# Fret Node Paint Root Frame Grid Diagnostics Adapter v1 - TODO

Status: Closed
Last updated: 2026-05-25

## FGD-M0 - Scope Freeze

- [x] FGD-010 [owner=codex] [deps=none] [scope=docs/workstreams/fret-node-paint-root-frame-grid-diagnostics-adapter-v1]
  Goal: Open a narrow follow-on for grid tile cache diagnostics recording only.
  Validation: `python3 -m json.tool docs/workstreams/fret-node-paint-root-frame-grid-diagnostics-adapter-v1/WORKSTREAM.json`
  Evidence: `DESIGN.md`, `EVIDENCE_AND_GATES.md`
  Handoff: Keep grid cache warmup, grid plan policy, tile op generation, tail cleanup, and
  cached/immediate passes out of scope.

## FGD-M1 - Grid Diagnostics Adapter Seam

- [x] FGD-020 [owner=codex] [deps=FGD-010] [scope=ecosystem/fret-node/src/ui/canvas/widget]
  Goal: Move grid tile cache diagnostics registry writes behind a grid diagnostics adapter plus
  retained `PaintCx` binding.
  Validation: `cargo test -p fret-node --features compat-retained-canvas paint_grid_diagnostics_adapter`
  Evidence: `paint_grid_stats.rs`, `paint_grid_diagnostics_adapter.rs`,
  `paint_grid_diagnostics_retained_cx.rs`, `ecosystem/fret-node/src/lib.rs`
  Handoff: The adapter should receive a grid tile stats snapshot; do not move grid cache warmup,
  grid plan policy, or tile op generation in this slice. Complete; retained registry writes live in
  `paint_grid_diagnostics_retained_cx.rs`.

## FGD-M2 - Closeout

- [x] FGD-030 [owner=codex] [deps=FGD-020] [scope=docs/workstreams/fret-node-paint-root-frame-grid-diagnostics-adapter-v1]
  Goal: Close the lane and split remaining grid/frame operation families into follow-on candidates.
  Validation: `python3 tools/check_workstream_catalog.py`; `git diff --check`
  Evidence: `CLOSEOUT_AUDIT_2026-05-25.md`
  Handoff: Start a separate follow-on for grid plan/chrome hint routing, tail cleanup, or
  cached/immediate pass clip emission.
