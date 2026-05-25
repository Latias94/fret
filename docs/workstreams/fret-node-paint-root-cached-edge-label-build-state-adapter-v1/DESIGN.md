# Fret Node Paint Root Cached Edge Label Build State Adapter v1

Status: Closed
Last updated: 2026-05-25

## Why This Lane Exists

`fret-node-paint-root-cached-edge-build-state-adapter-v1` closed cached edge build-state
host/services/scale route inputs and left cached edge-label build-state routing as the next separate
operation-family follow-on.

## Relevant Authority

- `docs/workstreams/fret-node-paint-root-cached-edge-build-state-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `docs/workstreams/fret-node-paint-root-cached-edge-replay-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `docs/adr/0330-retained-runtime-internal-and-compat-surface.md`
- `docs/adr/0128-canvas-widgets-and-interactive-surfaces.md`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/labels/single.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/labels/tiled.rs`

## Problem

Cached edge-label build-state routing still reaches through retained paint context fields:

- `cached_edges/labels/single.rs` passes `&*cx.app`, `cx.services`, and `cx.scale_factor` into
  edge-label build-state initialization and budget stepping.
- `cached_edges/labels/tiled.rs` repeats the same retained route-input reads per label tile.

This is distinct from cached edge build-state routing, cached replay scene sinks, temporary
cache-local scenes, cache keys, clip-op emission, and overlay routing.

## Target State

- Cached edge-label build-state host/services/scale inputs use a named adapter seam.
- `labels/single.rs` and `labels/tiled.rs` no longer read `cx.app`, `cx.services`, or
  `cx.scale_factor` directly for edge-label build-state routing.
- The retained `PaintCx` binding owns those retained field reads.
- Source-policy coverage locks the boundary and confirms edge build-state remains on its existing
  adapter.

## In Scope

- `cached_edges/labels/single.rs`
- `cached_edges/labels/tiled.rs`
- new cached edge-label build-state route-input adapter modules under `cached_edges/`
- focused source-policy coverage in `ecosystem/fret-node/src/lib.rs`
- workstream evidence and gates

## Out Of Scope

- cached edge build-state routing,
- cached edge and edge-label replay scene sinks,
- temporary `fret_core::Scene` construction,
- cache key semantics,
- cache-local clip-op emission,
- overlay routing,
- public scene schema changes.

## Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Edge-label build-state route inputs can mirror the edge build-state adapter shape. | Confident | Label single-rect and tiled paths need the same host/services/scale tuple as the edge path. | Split initialization and step input contracts only if borrowing pressure proves different. |
| Edge build-state should not share this lane. | Confident | The previous lane already closed edge build-state routing and locked it with source-policy coverage. | Treat any edge build-state change as regression unless required by compilation. |
| Replay access should stay on the existing replay adapter. | Confident | Label replay already routes through `PaintRootCachedEdgeReplayCx`. | Do not widen this lane into replay unless a compile-time bound forces a type-only wrapper update. |

## Architecture Direction

Prefer a narrow route-input adapter:

- `paint_root_cached_edge_label_build_state_host(cx)`
- `paint_root_cached_edge_label_build_state_step_inputs(cx)`

The adapter should expose only the retained inputs needed to initialize and step cached edge-label
build-state. Cache ownership, budget policy, temporary scene construction, replay, and redraw
requests stay in their existing helpers.

## Closeout Condition

This lane can close when cached edge-label build-state routing no longer reads retained
host/services/scale fields directly, source-policy coverage locks the seam, validation gates pass,
and deeper cache-local scene/clip/overlay cleanup remains recorded as follow-on work.

## Closeout State

Closed on 2026-05-25 with `CLOSEOUT_AUDIT_2026-05-25.md`. Cached edge-label build-state route
inputs now use the cached edge-label build-state adapter seam. Temporary scene construction,
cache-local clip-op construction, and overlay routing remain separate follow-on candidates.
