# Fret Node Paint Root Cached Edge Build State Adapter v1

Status: Closed
Last updated: 2026-05-25

## Why This Lane Exists

`fret-node-paint-root-cached-edge-replay-adapter-v1` closed cached edge and edge-label replay scene
sinks, then left cached edge build-state host/services/scale route inputs as the next narrow
operation-family follow-on.

## Relevant Authority

- `docs/workstreams/fret-node-paint-root-cached-edge-replay-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `docs/workstreams/fret-node-paint-root-cached-static-scene-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `docs/adr/0330-retained-runtime-internal-and-compat-surface.md`
- `docs/adr/0128-canvas-widgets-and-interactive-surfaces.md`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/edges/single.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/edges/tiled.rs`

## Problem

Cached edge build-state routing still reaches through retained paint context fields:

- `cached_edges/edges/single.rs` passes `&*cx.app`, `cx.services`, and `cx.scale_factor` into edge
  build-state initialization and budget stepping.
- `cached_edges/edges/tiled.rs` repeats the same retained route-input reads per tile.

The edge-label build-state path has similar retained reads, but it is a separate operation family
and remains out of scope for this lane.

## Target State

- Cached edge build-state host/services/scale inputs use a named adapter seam.
- `edges/single.rs` and `edges/tiled.rs` no longer read `cx.app`, `cx.services`, or
  `cx.scale_factor` directly for edge build-state routing.
- The retained `PaintCx` binding owns those retained field reads.
- Source-policy coverage locks the boundary and confirms edge-label build-state remains a separate
  follow-on.

## In Scope

- `cached_edges/edges/single.rs`
- `cached_edges/edges/tiled.rs`
- new cached edge build-state route-input adapter modules under `cached_edges/`
- focused source-policy coverage in `ecosystem/fret-node/src/lib.rs`
- workstream evidence and gates

## Out Of Scope

- edge-label build-state initialization or budget stepping,
- cached edge and edge-label replay scene sinks,
- temporary `fret_core::Scene` construction,
- cache key semantics,
- cache-local clip-op emission,
- overlay routing,
- public scene schema changes.

## Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Edge build-state route inputs can use one adapter for host, services, and scale factor. | Confident | Single-rect and tiled edge paths both need the same retained fields for edge render collection and budgeted edge paint. | Split initialization and step input contracts if borrowing pressure proves too high. |
| Edge labels should not share this lane. | Confident | The previous closeout explicitly listed cached edge-label build-state route inputs as a separate follow-on. | Open a dedicated label build-state adapter lane after this one closes. |
| Replay access should stay on the existing replay adapter. | Confident | Cached edge replay already has `PaintRootCachedEdgeReplayCx`; this lane only owns build-state route inputs. | Widen only if generic routing requires a shared composition trait, without adding label behavior. |

## Architecture Direction

Prefer a narrow route-input adapter:

- `paint_root_cached_edge_build_state_host(cx)`
- `paint_root_cached_edge_build_state_step_inputs(cx)`

The adapter should expose only the retained inputs needed to initialize and step cached edge
build-state. Cache ownership, budget policy, temporary scene construction, replay, and redraw
requests stay in their existing helpers.

## Closeout Condition

This lane can close when cached edge build-state routing no longer reads retained host/services/scale
fields directly, source-policy coverage locks the seam, validation gates pass, and edge-label
build-state route inputs are recorded as a follow-on rather than included here.

## Closeout State

Closed on 2026-05-25 with `CLOSEOUT_AUDIT_2026-05-25.md`. Cached edge build-state route inputs now
use the cached edge build-state adapter seam. Cached edge-label build-state route inputs remain the
next separate follow-on candidate.
