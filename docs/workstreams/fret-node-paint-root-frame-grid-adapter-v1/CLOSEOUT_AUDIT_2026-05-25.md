# Fret Node Paint Root Frame Grid Adapter v1 - Closeout Audit

Status: Closed
Last updated: 2026-05-25

## Verdict

This lane is closed.

It proved the grid tile cache warmup scene-sink adapter seam without widening into grid diagnostics,
grid plan policy, tile op generation, tail cleanup, cached/immediate passes, or public scene schema
changes.

## Shipped State

- `paint_grid_cache_adapter.rs` defines `PaintGridTileCacheCx`, `paint_grid_scene`, and
  `request_grid_paint_redraw`.
- `paint_grid_cache_retained_cx.rs` is the retained `PaintCx` binding for grid tile cache scene
  sink access.
- `paint_grid_cache/warm.rs` now owns tile budget selection, cache key use, replay delta, and tile
  op generation while delegating retained scene sink and redraw access through the adapter.
- Source-policy coverage in `ecosystem/fret-node/src/lib.rs` keeps the grid cache adapter free of
  retained lifecycle context names, verifies warmup no longer reads `PaintCx` or `cx.scene`
  directly, and confirms grid diagnostics remain in `paint_grid_stats.rs`.

## Split State

The following grid/frame operation families remain intentionally outside this lane:

- grid tile diagnostics registry writes,
- grid plan and chrome hint routing,
- grid tile operation generation,
- tail cleanup / `SceneOp::PopClip`,
- cached/immediate pass clip emission.

The next follow-on should choose one operation family. The smallest likely candidate is grid tile
diagnostics because `paint_grid_stats.rs` still mirrors the retained window/node/frame-id registry
write pattern that the path-cache diagnostics lane already isolated.

## Closeout Evidence

- `docs/workstreams/fret-node-paint-root-frame-background-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid_cache/warm.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid_cache_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid_cache_retained_cx.rs`
- `ecosystem/fret-node/src/lib.rs`

## Fresh Gates

- `cargo fmt --package fret-node` - passed.
- `cargo test -p fret-node --features compat-retained-canvas paint_grid_cache_adapter` - passed.
- `cargo check -p fret-node` - passed.
- `cargo check -p fret-node --features compat-retained-canvas` - passed.
- `python3 -m json.tool docs/workstreams/fret-node-paint-root-frame-grid-adapter-v1/WORKSTREAM.json` -
  passed.
- `python3 tools/check_workstream_catalog.py` - passed.
- `python3 tools/check_layering.py` - passed.
- `git diff --check` - passed.

## Residual Risks

- `paint_grid_stats.rs` still takes retained `PaintCx` for registry writes and should be split in a
  narrow grid diagnostics follow-on.
- Grid plan/chrome hint routing still uses retained app graph reads through `resolve_canvas_chrome_hint`.
