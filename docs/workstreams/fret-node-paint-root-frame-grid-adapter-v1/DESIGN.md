# Fret Node Paint Root Frame Grid Adapter v1

Status: Closed
Last updated: 2026-05-25

## Why This Lane Exists

`fret-node-paint-root-frame-background-adapter-v1` closed background scene emission and left grid
paint as the broadest remaining paint-root frame operation family.

## Relevant Authority

- `docs/workstreams/fret-node-paint-root-frame-background-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `docs/workstreams/fret-node-paint-root-frame-diagnostics-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `docs/adr/0330-retained-runtime-internal-and-compat-surface.md`
- `docs/adr/0128-canvas-widgets-and-interactive-surfaces.md`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid_cache/warm.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid_stats.rs`

## Problem

Grid paint currently mixes several concerns:

- grid plan policy and canvas chrome hint resolution,
- grid tile cache warmup and replay into the retained scene sink,
- paint redraw requests when the per-frame tile budget skips work,
- grid tile cache diagnostics written through retained `PaintCx` fields.

This lane must not turn into a broad grid rewrite. The smallest honest seam is the grid tile cache
warmup scene sink: `paint_grid_cache/warm.rs` directly reads `cx.scene` and calls retained redraw
helpers even though the tile plan, cache key, and op generation can stay in grid modules.

## Target State

- Grid tile cache warmup scene sink access is behind a named adapter seam.
- `paint_grid_cache/warm.rs` no longer takes retained `PaintCx` directly and no longer reads
  `cx.scene`.
- The retained `PaintCx` binding for the grid tile warmup scene sink lives in a retained grid cache
  binding module.
- Grid tile diagnostics remain in `paint_grid_stats.rs` for a separate follow-on.
- Source-policy coverage locks the warmup adapter boundary.

## In Scope

- `paint_grid_cache.rs`
- `paint_grid_cache/warm.rs`
- new grid cache adapter modules under `ui/canvas/widget/`
- source-policy coverage in `ecosystem/fret-node/src/lib.rs`
- workstream evidence and gates

## Out Of Scope

- grid plan/chrome hint policy,
- grid tile op generation,
- grid tile diagnostics registry writes,
- cache key schema changes,
- background paint,
- tail cleanup,
- cached/immediate passes,
- public scene schema changes.

## Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Grid paint is too broad for a single safe adapter. | Confident | Background closeout identifies grid paint as mixing plan policy, retained scene sink, cache warming, tile diagnostics, and scene emission. | Split additional grid lanes instead of broadening this one. |
| Warmup scene sink access is the smallest valuable seam. | Confident | `paint_grid_cache/warm.rs` directly passes `cx.scene` into `warm_scene_op_tiles_u64_with`. | If diagnostics is smaller, close this lane as audit-only and open diagnostics next. |
| Grid diagnostics should remain separate. | Confident | `paint_grid_stats.rs` writes `CanvasCacheStatsRegistry` with window/node/frame id, matching the prior diagnostics pattern. | Open a grid diagnostics adapter lane after this one. |

## Architecture Direction

Prefer a narrow cache-warmup adapter:

- `PaintGridTileCacheCx` exposes the retained scene sink needed by tile cache replay.
- `PaintGridTileCacheCx` also supports redraw requests through the existing low-level redraw
  adapter contract.
- `paint_grid_cache/warm.rs` keeps tile budget, cache key, replay delta, and tile op generation.
- `paint_grid_cache_retained_cx.rs` owns the retained `PaintCx.scene` binding.

## Closeout Condition

This lane can close when grid tile cache warmup no longer depends on retained `PaintCx` directly,
source-policy coverage locks the seam, and validation gates pass.

## Closeout State

Closed on 2026-05-25 with `CLOSEOUT_AUDIT_2026-05-25.md`. Grid tile cache warmup scene sink access
now uses the grid cache adapter seam. Grid diagnostics, grid plan/chrome hint routing, grid tile op
generation, tail cleanup, and cached/immediate pass clip emission remain separate follow-on
candidates.
