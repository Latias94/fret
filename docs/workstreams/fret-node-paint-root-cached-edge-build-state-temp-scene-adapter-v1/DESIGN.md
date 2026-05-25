# Fret Node Paint Root Cached Edge Build State Temp Scene Adapter v1

Status: Closed
Last updated: 2026-05-25

## Why This Lane Exists

`fret-node-paint-root-cached-edge-label-build-state-adapter-v1` closed cached edge-label
host/services/scale route inputs and left cache-local temporary scene construction as the next
internal cleanup surface across cached edge and edge-label build-state stepping.

## Relevant Authority

- `docs/workstreams/fret-node-paint-root-cached-edge-label-build-state-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `docs/workstreams/fret-node-paint-root-cached-edge-build-state-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `docs/adr/0330-retained-runtime-internal-and-compat-surface.md`
- `docs/adr/0128-canvas-widgets-and-interactive-surfaces.md`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/edges/single.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/edges/tiled.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/labels/single.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/labels/tiled.rs`

## Problem

Cached edge and edge-label build-state route helpers still construct cache-local temporary scenes
inline:

- `cached_edges/edges/single.rs`
- `cached_edges/edges/tiled.rs`
- `cached_edges/labels/single.rs`
- `cached_edges/labels/tiled.rs`

The temporary scene is an internal build-state detail used to collect budgeted paint ops before
`build_state/ops.rs` merges those ops back into cached state. Route helpers should not own scene
allocation shape.

## Target State

- Cached edge and edge-label route helpers no longer call `fret_core::Scene::default()` directly.
- Temporary scene construction lives behind a named build-state helper under `build_state/`.
- Existing edge and edge-label route-input adapters remain unchanged.
- Cache-local clip-op merging and replay/cache key behavior remain untouched.
- Source-policy coverage locks the boundary.

## In Scope

- `cached_edges/edges/single.rs`
- `cached_edges/edges/tiled.rs`
- `cached_edges/labels/single.rs`
- `cached_edges/labels/tiled.rs`
- `cached_edges/build_state.rs`
- `cached_edges/build_state/step.rs`
- new temporary scene helper under `cached_edges/build_state/`
- focused source-policy coverage in `ecosystem/fret-node/src/lib.rs`
- workstream evidence and gates

## Out Of Scope

- cache-local clip-op construction or merge policy,
- replay scene sinks,
- cache key semantics,
- route-input host/services/scale adapters,
- overlay routing,
- public scene schema changes.

## Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Temporary scene construction can move into build-state step helpers without changing behavior. | Confident | The route helpers create a fresh empty scene immediately before each budgeted step and only pass it into build-state stepping. | Keep a small wrapper but preserve the fresh-per-step scene invariant. |
| Clip-op merging is a separate lane. | Confident | `build_state/ops.rs` owns `initial_clip_ops` and `finish_build_state_step`; moving construction does not require changing merge policy. | Split a follow-on for clip-op construction/merge cleanup. |
| Edge and edge-label temporary scenes can share one helper. | Likely | Both paths use a fresh `fret_core::Scene::default()` for budgeted paint ops. | Split helpers only if future scene metadata diverges. |

## Architecture Direction

Prefer a narrow helper seam:

- `paint_root_cached_edge_build_state_temp_scene()`

Build-state step helpers should own temporary scene construction, then pass the scene to the existing
budgeted edge/label paint functions and `finish_build_state_step`.

## Closeout Condition

This lane can close when the four cached edge/label route helpers no longer construct temporary
scenes directly, source-policy coverage proves construction ownership, validation gates pass, and
clip-op construction remains recorded as a separate follow-on.

## Closeout State

Closed on 2026-05-25 with `CLOSEOUT_AUDIT_2026-05-25.md`. Cached edge and edge-label build-state
temporary scene construction now lives in `build_state/temp_scene.rs` and is owned by build-state
step helpers. Cache-local clip-op construction and merge policy remain the next separate follow-on
candidate.
