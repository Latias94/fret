# Fret Node Paint Root Frame Diagnostics Adapter v1

Status: Closed
Last updated: 2026-05-25

## Why This Lane Exists

`fret-node-paint-root-frame-clip-adapter-v1` closed the root frame clip scene-emission seam and
left path-cache diagnostics as a separate frame operation-family candidate.

## Relevant Authority

- `docs/workstreams/fret-node-paint-root-frame-clip-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `docs/workstreams/fret-node-paint-root-frame-setup-adapter-v1/FRAME_SETUP_SCOPE_AUDIT_2026-05-25.md`
- `docs/adr/0330-retained-runtime-internal-and-compat-surface.md`
- `docs/adr/0128-canvas-widgets-and-interactive-surfaces.md`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame/cache.rs`

## Problem

`record_path_cache_stats` still mixes two concerns:

- collecting path-cache diagnostics snapshot data from the node graph canvas cache,
- reading retained `PaintCx` fields (`window`, `node`, `app`) and writing
  `CanvasCacheStatsRegistry`.

That retained diagnostics write is separate from cache frame begin, viewport preparation, clip
emission, background paint, grid paint, and tail cleanup.

## Target State

- Path-cache diagnostics recording is behind a named adapter seam.
- `frame/cache.rs` no longer reads retained `PaintCx` fields directly for path-cache diagnostics.
- The retained `PaintCx` binding for `window`, `node`, `app.frame_id`, and registry writes lives in
  a retained diagnostics binding module.
- Source-policy coverage locks the diagnostics adapter boundary.

## In Scope

- `paint_root/frame/cache.rs`
- new frame diagnostics adapter modules under `paint_root/`
- source-policy coverage in `ecosystem/fret-node/src/lib.rs`
- workstream evidence and gates

## Out Of Scope

- cache frame begin,
- viewport/bounds/render-cull,
- root frame clip emission,
- background paint,
- grid paint and grid tile diagnostics,
- edge label budget diagnostics,
- tail cleanup,
- cached/immediate passes,
- public diagnostics schema changes.

## Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Path-cache diagnostics is a separate operation family. | Confident | `FRAME_SETUP_SCOPE_AUDIT_2026-05-25.md` lists diagnostics separately from cache begin and scene emission. | Re-audit before implementing a broader diagnostics adapter. |
| The seam should accept a path-cache snapshot, not own cache begin. | Confident | `begin_paint_root_caches` has no retained context use. | Split cache begin only if future cache lifecycle work needs it. |
| Grid and edge diagnostics should remain out of scope. | Likely | They live in separate files and record different cache/budget families. | Open separate diagnostics lanes if repeated patterns justify it. |

## Architecture Direction

Prefer a narrow action adapter:
`record_paint_root_path_cache_stats(cx, entries, stats)`. `cache.rs` should own snapshot collection;
the retained binding should own window checks, key construction, frame id, and registry writes.

## Closeout Condition

This lane can close when path-cache diagnostics recording is isolated behind the adapter,
source-policy coverage locks the seam, and the validation gates pass.

## Closeout State

Closed on 2026-05-25 with `CLOSEOUT_AUDIT_2026-05-25.md`. Path-cache diagnostics recording now uses
the frame diagnostics adapter seam. Background paint, grid paint, tail cleanup, cached/immediate
pass clip emission, grid tile diagnostics, and edge label budget diagnostics remain separate
follow-on candidates.
