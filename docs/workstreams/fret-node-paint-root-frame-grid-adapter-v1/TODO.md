# Fret Node Paint Root Frame Grid Adapter v1 - TODO

Status: Closed
Last updated: 2026-05-25

## FGA-M0 - Scope Freeze

- [x] FGA-010 [owner=codex] [deps=none] [scope=docs/workstreams/fret-node-paint-root-frame-grid-adapter-v1]
  Goal: Open a narrow follow-on for grid tile cache warmup scene sink access only.
  Validation: `python3 -m json.tool docs/workstreams/fret-node-paint-root-frame-grid-adapter-v1/WORKSTREAM.json`
  Evidence: `DESIGN.md`, `EVIDENCE_AND_GATES.md`
  Handoff: Keep grid diagnostics, grid plan policy, tile op generation, tail cleanup, and
  cached/immediate passes out of scope.

## FGA-M1 - Grid Tile Cache Adapter Seam

- [x] FGA-020 [owner=codex] [deps=FGA-010] [scope=ecosystem/fret-node/src/ui/canvas/widget]
  Goal: Move grid tile cache warmup scene sink access behind a grid cache adapter plus retained
  `PaintCx` binding.
  Validation: `cargo test -p fret-node --features compat-retained-canvas paint_grid_cache_adapter`
  Evidence: `paint_grid_cache/warm.rs`, `paint_grid_cache_adapter.rs`,
  `paint_grid_cache_retained_cx.rs`, `ecosystem/fret-node/src/lib.rs`
  Handoff: Do not move `paint_grid_stats.rs` diagnostics or grid plan/chrome hint policy in this
  slice. Complete; retained scene sink access lives in `paint_grid_cache_retained_cx.rs`.

## FGA-M2 - Closeout

- [x] FGA-030 [owner=codex] [deps=FGA-020] [scope=docs/workstreams/fret-node-paint-root-frame-grid-adapter-v1]
  Goal: Close the lane and split remaining grid/frame operation families into follow-on candidates.
  Validation: `python3 tools/check_workstream_catalog.py`; `git diff --check`
  Evidence: `CLOSEOUT_AUDIT_2026-05-25.md`
  Handoff: Start a separate follow-on for grid diagnostics, tail cleanup, cached/immediate pass clip
  emission, or grid plan/chrome hint routing.
