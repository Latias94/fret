# Fret Node Paint Root Cached Edge Anchor Target Adapter v1

Status: Closed
Last updated: 2026-05-25

Status note (2026-05-25): this lane shipped the cached edge anchor target adapter seam and is
closed. Future retained cleanup for fallback route inputs, cache keys, or deeper shared
edge-anchor internals should start as a separate narrow follow-on.

## Why This Lane Exists

`fret-node-paint-root-cached-edge-overlay-adapter-v1` closed cached selected/hovered overlay route
ownership. The next narrow retained paint-root surface is cached edge anchor target routing.

## Relevant Authority

- `docs/workstreams/fret-node-paint-root-cached-edge-overlay-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `docs/adr/0330-retained-runtime-internal-and-compat-surface.md`
- `docs/adr/0128-canvas-widgets-and-interactive-surfaces.md`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/anchor_target.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/edge_anchor.rs`

## Problem

Cached edge anchor target routing still receives retained `PaintCx` directly and passes it to the
shared edge-anchor helpers:

- `resolve_edge_anchor_target_id(cx, snapshot)`
- `resolve_edge_anchor_target_from_geometry(cx, geom, edge_anchor_target_id)`

That route ownership is separate from fallback uncached edge rendering, cache keys, replay scene
sinks, build-state helpers, and overlay paint routing.

## Target State

- Cached edge anchor target routing uses a named adapter seam.
- `cached_edges/anchor_target.rs` no longer names retained `PaintCx` or calls the shared
  `resolve_edge_anchor_target_*` helpers directly.
- The retained `PaintCx` binding owns the direct shared edge-anchor helper calls.
- Source-policy coverage locks the boundary.

## In Scope

- `cached_edges/anchor_target.rs`
- new cached edge anchor target adapter modules under `cached_edges/`
- focused source-policy coverage in `ecosystem/fret-node/src/lib.rs`
- workstream evidence and gates

## Out Of Scope

- fallback uncached edge rendering,
- selected/hovered overlay routing,
- cached edge replay scene sinks,
- cache key semantics,
- build-state temporary scene or clip-op helpers,
- the deeper shared `paint_root/edge_anchor/*` implementation.

## Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Cached anchor target routing can be moved without changing behavior. | Confident | The current helper only sequences target id then geometry target resolution. | Preserve call ordering and return shape exactly. |
| Deeper `edge_anchor` helpers should remain shared for now. | Confident | Immediate path also uses the same target-id helper and render-based target resolution. | Split a shared edge-anchor follow-on only if needed. |
| Fallback, overlay, replay, and cache-key cleanup should stay separate. | Confident | Recent closed lanes handled each surface as narrow seams. | Open separate follow-ons instead of widening this lane. |

## Architecture Direction

Prefer a narrow anchor target adapter:

- `resolve_paint_root_cached_edge_anchor_target(cx, canvas, snapshot, geom)`

The retained binding should delegate to `resolve_edge_anchor_target_id` and
`resolve_edge_anchor_target_from_geometry`; cached route helpers should only call the adapter.

## Closeout Condition

This lane can close when cached anchor target routing no longer names retained `PaintCx` or calls the
shared edge-anchor helpers directly, source-policy coverage proves the seam, validation gates pass,
and fallback/overlay/replay/cache-key cleanup remains separate.

Closeout result (2026-05-25): complete. Cached anchor target routing calls
`anchor_target_adapter::resolve_paint_root_cached_edge_anchor_target`; the retained binding owns the
direct `resolve_edge_anchor_target_id` and `resolve_edge_anchor_target_from_geometry` calls.
