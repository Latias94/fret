# Fret Node Paint Root Cached Edge Fallback Adapter v1

Status: Closed
Last updated: 2026-05-25

Status note (2026-05-25): this lane shipped the cached edge fallback adapter seam and is closed.
Future retained cleanup for cache keys or deeper `paint_edges` internals should start as separate
narrow follow-ons.

## Why This Lane Exists

`fret-node-paint-root-cached-edge-anchor-target-adapter-v1` closed cached anchor target route
ownership. The next narrow retained paint-root surface is cached edge fallback rendering: both the
static-cache-disabled fallback and the uncached edge fallback inside cached single/tiled paths still
read retained `PaintCx` route inputs directly.

## Relevant Authority

- `docs/workstreams/fret-node-paint-root-cached-edge-anchor-target-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `docs/adr/0330-retained-runtime-internal-and-compat-surface.md`
- `docs/adr/0128-canvas-widgets-and-interactive-surfaces.md`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/fallback.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/edges/fallback.rs`

## Problem

Cached edge fallback helpers still read retained route inputs directly:

- `cached_edges/fallback.rs` calls `collect_render_data(&*cx.app, ...)` and `canvas.paint_edges(cx, ...)`.
- `cached_edges/edges/fallback.rs` does the same for the uncached edge fallback in cached paths.

This is separate from cache keys, replay scene sinks, overlay routing, anchor target routing, and
the deeper `paint_edges` internals.

## Target State

- Cached edge fallback rendering uses a named adapter seam for retained host access and retained
  edge paint dispatch.
- The two fallback implementation helpers no longer name retained `PaintCx`, read `cx.app`, or call
  `canvas.paint_edges` directly.
- The retained `PaintCx` binding owns retained host extraction and direct edge paint dispatch.
- Source-policy coverage locks the boundary.

## In Scope

- `cached_edges/fallback.rs`
- `cached_edges/edges/fallback.rs`
- new cached edge fallback adapter modules under `cached_edges/`
- focused source-policy coverage in `ecosystem/fret-node/src/lib.rs`
- workstream evidence and gates

## Out Of Scope

- cache key semantics,
- cached edge replay scene sinks,
- selected/hovered overlay routing,
- anchor target routing,
- build-state temporary scene or clip-op helpers,
- internals of `paint_edges` and edge paint preparation.

## Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Both fallback helpers can share one adapter. | Confident | They both need host access for `collect_render_data` and retained edge paint dispatch. | Split only if call ordering or cache-state clearing diverges. |
| `paint_edges` internals should remain out of scope. | Confident | The current slice only moves cached fallback route ownership. | Start a deeper edge-paint follow-on if needed. |
| Cache key cleanup should stay separate. | Confident | Fallback helpers do not build cache keys. | Open a cache-key lane after fallback is closed. |

## Architecture Direction

Prefer a narrow fallback adapter:

- `paint_root_cached_edge_fallback_host(cx)`
- `paint_root_cached_edge_fallback_paint_edges(cx, canvas, snapshot, render, geom, zoom, view_interacting)`

Fallback helpers should collect render data through the adapter-provided host and dispatch edge
paint through the adapter; retained binding should delegate to `cx.app` and `canvas.paint_edges`.

## Closeout Condition

This lane can close when both cached edge fallback helpers no longer name retained `PaintCx`, read
`cx.app`, or call `canvas.paint_edges` directly, source-policy coverage proves the seam, validation
gates pass, and cache-key/replay/overlay/anchor/deeper edge paint cleanup remains separate.

Closeout result (2026-05-25): complete. Cached edge fallback helpers call
`fallback_adapter::paint_root_cached_edge_fallback_host` and
`fallback_adapter::paint_root_cached_edge_fallback_paint_edges`; the retained binding owns `self.app`
and direct `canvas.paint_edges` dispatch.
