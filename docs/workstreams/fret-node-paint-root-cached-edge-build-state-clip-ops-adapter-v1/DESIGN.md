# Fret Node Paint Root Cached Edge Build State Clip Ops Adapter v1

Status: Closed
Last updated: 2026-05-25

## Why This Lane Exists

`fret-node-paint-root-cached-edge-build-state-temp-scene-adapter-v1` moved temporary scene
construction into build-state stepping and left cache-local clip stack construction/merge policy as
the next focused follow-on.

## Relevant Authority

- `docs/workstreams/fret-node-paint-root-cached-edge-build-state-temp-scene-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `docs/adr/0330-retained-runtime-internal-and-compat-surface.md`
- `docs/adr/0128-canvas-widgets-and-interactive-surfaces.md`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/build_state/ops.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/build_state/init.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/build_state/step.rs`

## Problem

`build_state/ops.rs` still owns both completion bookkeeping and low-level cache-local clip stack
details:

- `initial_clip_ops` constructs `SceneOp::PushClipRect` and `SceneOp::PopClip`.
- `extend_clip_stack_ops` rewrites cached op vectors around the trailing `PopClip` sentinel.
- `finish_build_state_step` mixes step completion bookkeeping with clip-stack temp-op merging.

The completion bookkeeping can remain in `ops.rs`, but clip stack construction and merge policy
should move behind a named helper seam.

## Target State

- Cache-local clip stack construction and temp-op merge policy live in `build_state/clip_ops.rs`.
- `build_state/ops.rs` no longer directly mentions `SceneOp::PushClipRect` or `SceneOp::PopClip`.
- `finish_build_state_step` still owns `next_edge` bookkeeping and delegates clip temp-op merging.
- Replay, cache keys, route-input adapters, and temporary scene construction remain untouched.
- Source-policy coverage locks the boundary.

## In Scope

- `cached_edges/build_state/ops.rs`
- new `cached_edges/build_state/clip_ops.rs`
- source-policy coverage in `ecosystem/fret-node/src/lib.rs`
- workstream evidence and gates

## Out Of Scope

- temporary scene construction,
- replay scene sinks,
- cache key semantics,
- route-input host/services/scale adapters,
- overlay routing,
- public scene schema changes.

## Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Clip stack construction and temp-op merging can move without changing behavior. | Confident | Both are private helpers in `build_state/ops.rs` and only operate on `Vec<SceneOp>` plus temp ops. | Keep wrappers in `ops.rs` but preserve the exact sentinel behavior. |
| `finish_build_state_step` should remain the completion owner. | Confident | Step helpers use it for `next_edge` bookkeeping and continued-work return value. | Split bookkeeping only in a later lane if needed. |
| This lane does not need a retained binding. | Confident | Clip ops are cache-local scene op policy, not retained context access. | Do not add retained-facing traits. |

## Architecture Direction

Prefer a narrow clip helper seam:

- `paint_root_cached_edge_build_state_initial_clip_ops(clip_rect)`
- `paint_root_cached_edge_build_state_merge_temp_ops(ops, temp_ops)`

`ops.rs` should call those helpers and keep completion bookkeeping. The helper should be the only
cached edge build-state module that directly constructs or maintains `PushClipRect`/`PopClip`.

## Closeout Condition

This lane can close when `build_state/ops.rs` delegates clip stack construction/merge to
`build_state/clip_ops.rs`, focused source-policy coverage proves the seam, validation gates pass,
and replay/cache-key/adapter behavior remains unchanged.

## Closeout State

Closed on 2026-05-25 with `CLOSEOUT_AUDIT_2026-05-25.md`. Cache-local clip stack construction and
temp-op merge policy now live in `build_state/clip_ops.rs`; `build_state/ops.rs` keeps completion
bookkeeping and delegates clip policy.
