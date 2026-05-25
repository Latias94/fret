# Fret Node Paint Root Frame Grid Diagnostics Adapter v1

Status: Closed
Last updated: 2026-05-25

## Why This Lane Exists

`fret-node-paint-root-frame-grid-adapter-v1` closed grid tile cache warmup scene-sink access and
left grid tile diagnostics registry writes as the smallest remaining grid follow-on.

## Relevant Authority

- `docs/workstreams/fret-node-paint-root-frame-grid-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `docs/workstreams/fret-node-paint-root-frame-diagnostics-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `docs/adr/0330-retained-runtime-internal-and-compat-surface.md`
- `docs/adr/0128-canvas-widgets-and-interactive-surfaces.md`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid_stats.rs`

## Problem

`paint_grid_stats.rs` still mixes two concerns:

- collecting grid tile cache diagnostics snapshot data from the node graph canvas and warmup stats,
- reading retained `PaintCx` fields (`window`, `node`, `app`) and writing
  `CanvasCacheStatsRegistry`.

That retained diagnostics write is separate from grid tile cache warmup, grid plan policy, tile op
generation, tail cleanup, and cached/immediate passes.

## Target State

- Grid tile cache diagnostics recording is behind a named adapter seam.
- `paint_grid_stats.rs` no longer reads retained `PaintCx` fields directly.
- The retained `PaintCx` binding for `window`, `node`, `app.frame_id`, and registry writes lives in
  a retained grid diagnostics binding module.
- Source-policy coverage locks the grid diagnostics adapter boundary.

## In Scope

- `paint_grid_stats.rs`
- new grid diagnostics adapter modules under `ui/canvas/widget/`
- source-policy coverage in `ecosystem/fret-node/src/lib.rs`
- workstream evidence and gates

## Out Of Scope

- grid tile cache warmup scene sink access,
- grid plan/chrome hint policy,
- grid tile op generation,
- cache key schema changes,
- path-cache diagnostics,
- tail cleanup,
- cached/immediate passes,
- public diagnostics schema changes.

## Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Grid diagnostics is a separate operation family from grid warmup. | Confident | Grid adapter closeout names registry writes as a split follow-on. | Reopen only as a narrower follow-on, not by broadening grid cache warmup. |
| The seam should accept a stats snapshot, not own cache warmup. | Confident | `paint_grid_cache/warm.rs` already owns budget and warmup after the grid adapter lane. | Split cache lifecycle only if a future cache lane needs it. |
| The retained binding should mirror the path-cache diagnostics pattern. | Confident | `frame_diagnostics_retained_cx.rs` already isolates window/node/frame-id registry writes. | Adjust only if registry APIs change. |

## Architecture Direction

Prefer a narrow diagnostics adapter:
`record_grid_tile_cache_stats(cx, snapshot)`. `paint_grid_stats.rs` should own snapshot collection;
the retained binding should own window checks, key construction, frame id, and registry writes.

## Closeout Condition

This lane can close when grid tile cache diagnostics recording is isolated behind the adapter,
source-policy coverage locks the seam, and validation gates pass.

## Closeout State

Closed on 2026-05-25 with `CLOSEOUT_AUDIT_2026-05-25.md`. Grid tile cache diagnostics recording now
uses the grid diagnostics adapter seam. Grid plan/chrome hint routing, grid tile op generation,
tail cleanup, and cached/immediate pass clip emission remain separate follow-on candidates.
