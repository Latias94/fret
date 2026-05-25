# Fret Node Paint Root Cached Edge Overlay Adapter v1

Status: Closed
Last updated: 2026-05-25

Status note (2026-05-25): this lane shipped the cached edge selected/hovered overlay adapter seam
and is closed. Future retained cleanup for anchor target resolution, fallback route inputs, replay,
or cache keys should start as a separate narrow follow-on.

## Why This Lane Exists

`fret-node-paint-root-cached-edge-build-state-clip-ops-adapter-v1` closed cached build-state clip
ops. The next narrow retained paint-root surface is cached edge selected/hovered overlay routing in
the single-rect and tiled cached paths.

## Relevant Authority

- `docs/workstreams/fret-node-paint-root-cached-edge-build-state-clip-ops-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `docs/adr/0330-retained-runtime-internal-and-compat-surface.md`
- `docs/adr/0128-canvas-widgets-and-interactive-surfaces.md`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/single_rect.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/tile_path.rs`

## Problem

Cached edge route helpers still call retained overlay paint directly:

- `single_rect.rs` calls `paint_edge_overlays_selected_hovered(cx, snapshot, geom, zoom)`.
- `tile_path.rs` calls the same retained paint helper in the tiled cached path.

This is separate from anchor target resolution, fallback uncached edge paint, replay, and cache key
construction.

## Target State

- Cached edge selected/hovered overlay routing uses a named adapter seam.
- `single_rect.rs` and `tile_path.rs` no longer call `paint_edge_overlays_selected_hovered` directly.
- The retained `PaintCx` binding owns the direct overlay paint helper call.
- Source-policy coverage locks the boundary.

## In Scope

- `cached_edges/single_rect.rs`
- `cached_edges/tile_path.rs`
- new cached edge overlay adapter modules under `cached_edges/`
- focused source-policy coverage in `ecosystem/fret-node/src/lib.rs`
- workstream evidence and gates

## Out Of Scope

- edge anchor target resolution,
- fallback uncached edge rendering,
- replay scene sinks,
- cache key semantics,
- route-input host/services/scale adapters,
- build-state temporary scene or clip-op helpers,
- public overlay semantics.

## Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Selected/hovered cached edge overlays can share one adapter. | Confident | Single-rect and tiled paths call the same overlay paint helper with the same arguments. | Split only if ordering semantics diverge. |
| Anchor target resolution should stay separate. | Confident | It returns data consumed by paint-root tail and has a different input/output shape. | Open an anchor-target follow-on. |
| The adapter should not change overlay behavior. | Confident | This is a route ownership change only. | Preserve call ordering and arguments exactly. |

## Architecture Direction

Prefer a narrow overlay adapter:

- `paint_root_cached_edge_overlays(cx, canvas, snapshot, geom, zoom)`

The retained binding should delegate to `paint_edge_overlays_selected_hovered`; cached route helpers
should only call the adapter.

## Closeout Condition

This lane can close when cached single-rect and tiled edge routes no longer call the retained overlay
paint helper directly, source-policy coverage proves the seam, validation gates pass, and anchor
target/fallback/replay/cache-key cleanup remains separate.

Closeout result (2026-05-25): complete. Cached single-rect and tiled edge routes call
`overlay_adapter::paint_root_cached_edge_overlays_selected_hovered`; the retained binding owns the
direct `paint_edge_overlays_selected_hovered` call.
