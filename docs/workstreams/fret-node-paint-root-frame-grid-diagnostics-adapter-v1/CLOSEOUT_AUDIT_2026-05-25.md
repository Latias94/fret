# Fret Node Paint Root Frame Grid Diagnostics Adapter v1 - Closeout Audit

Status: Closed
Last updated: 2026-05-25

## Verdict

This lane is closed.

It proved the grid tile cache diagnostics adapter seam without widening into grid cache warmup, grid
plan policy, tile op generation, tail cleanup, cached/immediate passes, or public diagnostics schema
changes.

## Shipped State

- `paint_grid_diagnostics_adapter.rs` defines `GridTileCacheStatsSnapshot`,
  `PaintGridDiagnosticsCx`, and `record_grid_tile_cache_stats`.
- `paint_grid_diagnostics_retained_cx.rs` is the retained `PaintCx` binding for grid tile cache
  diagnostics registry writes.
- `paint_grid_stats.rs` now owns only snapshot collection from the grid scene cache and warmup
  stats before delegating recording to the diagnostics adapter.
- Source-policy coverage in `ecosystem/fret-node/src/lib.rs` keeps the grid diagnostics adapter
  free of retained lifecycle context names and registry types, verifies `paint_grid_stats.rs` no
  longer reads retained `PaintCx` fields directly, and verifies the retained binding owns
  window/node/frame-id registry writes.

## Split State

The following grid/frame operation families remain intentionally outside this lane:

- grid plan and chrome hint routing,
- grid tile operation generation,
- tail cleanup / `SceneOp::PopClip`,
- cached/immediate pass clip emission.

The next follow-on should choose one operation family. The smallest likely candidate is tail cleanup
because it still emits the root frame `PopClip` directly after the frame operation sequence.

## Closeout Evidence

- `docs/workstreams/fret-node-paint-root-frame-grid-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid_stats.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid_diagnostics_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid_diagnostics_retained_cx.rs`
- `ecosystem/fret-node/src/lib.rs`

## Fresh Gates

- `cargo fmt --package fret-node` - passed.
- `cargo test -p fret-node --features compat-retained-canvas paint_grid_diagnostics_adapter` -
  passed.
- `cargo check -p fret-node` - passed.
- `cargo check -p fret-node --features compat-retained-canvas` - passed.
- `python3 -m json.tool docs/workstreams/fret-node-paint-root-frame-grid-diagnostics-adapter-v1/WORKSTREAM.json` -
  passed.
- `python3 tools/check_workstream_catalog.py` - passed.
- `python3 tools/check_layering.py` - passed.
- `git diff --check` - passed.

## Residual Risks

- Grid plan/chrome hint routing still uses retained app graph reads through `resolve_canvas_chrome_hint`.
- Tail cleanup and cached/immediate pass clip emission still directly use retained paint scene access.
