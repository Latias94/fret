# Fret Node Paint Root Cached Static Scene Adapter v1

Status: Closed
Last updated: 2026-05-25

## Why This Lane Exists

`fret-node-paint-root-pass-clip-adapter-v1` closed immediate pass static scene routing and explicitly
split cached static layer scene replay into a follow-on.

## Relevant Authority

- `docs/workstreams/fret-node-paint-root-pass-clip-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `docs/workstreams/fret-node-paint-root-tail-cleanup-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `docs/adr/0330-retained-runtime-internal-and-compat-surface.md`
- `docs/adr/0128-canvas-widgets-and-interactive-surfaces.md`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/static_layer.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/static_cache.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_groups.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_nodes.rs`

## Problem

Cached group/node static scene replay still knows retained paint context shape:

- `static_layer.rs` replays cached group/node ops through `cx.scene`.
- `static_cache.rs` stores and replays through `PaintCx` and `cx.scene`.
- `cached_groups.rs` / `cached_nodes.rs` build cache-local static ops by reading `cx.app`,
  `cx.services`, and `cx.scale_factor` directly.

These are separate from cached edge replay and overlay layer routing. Keeping this lane on static
group/node cached scene replay prevents a broad cached-paint rewrite.

## Target State

- Cached static group/node replay uses a named adapter seam for retained scene access.
- `static_layer.rs` and `static_cache.rs` no longer read `cx.scene` or depend on `PaintCx`.
- `cached_groups.rs` and `cached_nodes.rs` no longer read `cx.app`, `cx.services`, or
  `cx.scale_factor` directly for cached static op construction.
- The retained `PaintCx` binding owns cached static host/services/scale/scene field access.
- Source-policy coverage locks the boundary.

## In Scope

- `paint_root/static_layer.rs`
- `paint_root/static_cache.rs`
- `paint_root/cached_groups.rs`
- `paint_root/cached_nodes.rs`
- new cached static scene adapter modules under `paint_root/`
- source-policy coverage in `ecosystem/fret-node/src/lib.rs`
- workstream evidence and gates

## Out Of Scope

- cached edge and edge-label replay,
- immediate edge paint routing,
- immediate or cached overlay layer routing,
- cache key semantics,
- tile cache eviction policy,
- public scene schema changes,
- visual behavior changes.

## Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Static group/node cached replay is a separate operation family from cached edge replay. | Confident | Pass scene closeout lists cached static layer and cached edge replay separately. | Split this lane further if edge replay proves coupled. |
| The adapter should expose host/services/scale/scene route inputs, not move cache key policy. | Likely | Existing cache-plan and pass-scene adapters keep policy in the routing owner and retained fields in bindings. | If this keeps too much retained shape in cached modules, promote a narrower action adapter in a follow-on. |
| Cache-local clip ops can remain in cached group/node builders for this slice. | Likely | This lane owns retained scene sink and context field access, not every `SceneOp` emitted into cache-local temp scenes. | Split a cached local clip-op adapter if source-policy later requires scene-op agnosticism there too. |

## Architecture Direction

Prefer a narrow cached static scene adapter:

- `paint_root_cached_static_host(cx)`
- `paint_root_cached_static_services(cx)`
- `paint_root_cached_static_scale_factor(cx)`
- `paint_root_cached_static_scene(cx)`

`static_cache.rs` should use the adapter for replay scene access. `cached_groups.rs` and
`cached_nodes.rs` should use it for route inputs while keeping render collection and cache-local op
construction in their existing modules.

## Closeout Condition

This lane can close when cached static replay no longer reads retained `PaintCx` fields directly,
source-policy coverage locks the seam, and validation gates pass.

## Closeout State

Closed on 2026-05-25 with `CLOSEOUT_AUDIT_2026-05-25.md`. Cached static group/node replay now uses
the cached static scene adapter seam. Cached edge replay, edge-label replay, cache-local clip-op
emission, and overlay routing remain separate follow-on candidates.
