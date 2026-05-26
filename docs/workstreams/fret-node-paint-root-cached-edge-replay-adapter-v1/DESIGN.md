# Fret Node Paint Root Cached Edge Replay Adapter v1

Status: Closed
Last updated: 2026-05-25

## Why This Lane Exists

`fret-node-paint-root-cached-static-scene-adapter-v1` closed cached static group/node replay and
left cached edge and edge-label replay as the next direct retained `cx.scene` surface.

## Relevant Authority

- `docs/workstreams/fret-node-paint-root-cached-static-scene-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `docs/workstreams/fret-node-paint-root-pass-clip-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `docs/adr/0330-retained-runtime-internal-and-compat-surface.md`
- `docs/adr/0128-canvas-widgets-and-interactive-surfaces.md`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/edges/replay.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/labels/replay.rs`

## Problem

Cached edge replay still writes to retained scene fields directly:

- `cached_edges/edges/replay.rs` calls `cx.scene.replay_ops_translated` and passes `cx.scene` into
  `try_replay_with`.
- `cached_edges/labels/replay.rs` does the same for edge-label ops.

These replay sinks are separate from edge build-state preparation, edge label budget stepping,
temporary cache-local scenes, cache keys, and overlay routing.

## Target State

- Cached edge and edge-label replay use a named replay adapter for retained scene access.
- `edges/replay.rs` and `labels/replay.rs` no longer mention `PaintCx` or `cx.scene`.
- The retained `PaintCx` binding owns the retained scene field read.
- Source-policy coverage locks the boundary.

## In Scope

- `cached_edges/edges/replay.rs`
- `cached_edges/labels/replay.rs`
- new cached edge replay adapter modules under `cached_edges/`
- source-policy coverage in `ecosystem/fret-node/src/lib.rs`
- workstream evidence and gates

## Out Of Scope

- edge build-state initialization,
- edge and label budget stepping,
- temporary `fret_core::Scene` construction,
- cache key semantics,
- cache-local clip-op emission,
- overlay routing,
- public scene schema changes.

## Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Edge and edge-label replay can share one scene replay adapter. | Confident | Both replay files need only retained scene access for replaying cached ops. | Split edge-label replay if text cache touch semantics require a separate contract. |
| Build-state retained field reads are outside this lane. | Likely | The previous closeout singled out direct `cx.scene` replay as the sharper next seam. | Open a build-state route-input follow-on after replay is isolated. |
| The adapter should expose only replay scene access. | Confident | Cache ownership and paint-cache touch policy already live in replay helpers. | Widen only if compile-time borrowing forces cache operations into the adapter. |

## Architecture Direction

Prefer a narrow replay adapter:

- `paint_root_cached_edge_replay_scene(cx)`

Replay helpers should keep cache lookup, cache store, and paint-cache touch policy. The retained
binding should own only the retained scene sink.

## Closeout Condition

This lane can close when cached edge and edge-label replay no longer read retained `PaintCx` scene
fields directly, source-policy coverage locks the seam, and validation gates pass.

## Closeout State

Closed on 2026-05-25 with `CLOSEOUT_AUDIT_2026-05-25.md`. Cached edge and edge-label replay now use
the cached edge replay adapter seam. Edge build-state route inputs, temporary scene construction,
cache-local clip-op emission, cache keys, and overlay routing remain separate follow-on candidates.
