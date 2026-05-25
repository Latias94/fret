# Fret Node Paint Root Tail Cleanup Adapter v1

Status: Closed
Last updated: 2026-05-25

## Why This Lane Exists

`fret-node-paint-root-frame-grid-diagnostics-adapter-v1` closed grid diagnostics and left tail
cleanup / `SceneOp::PopClip` as the smallest remaining paint-root frame operation-family candidate.

## Relevant Authority

- `docs/workstreams/fret-node-paint-root-frame-grid-diagnostics-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `docs/workstreams/fret-node-paint-root-frame-clip-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `docs/adr/0330-retained-runtime-internal-and-compat-surface.md`
- `docs/adr/0128-canvas-widgets-and-interactive-surfaces.md`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/tail.rs`

## Problem

Root frame clip push already goes through `frame_clip_adapter`, but tail cleanup still emits the
matching root frame `SceneOp::PopClip` directly through retained `PaintCx.scene`.

That direct scene write is separate from overlays, cache pruning, cached layer internals, and
cached/immediate pass routing.

## Target State

- Root frame tail cleanup clip pop emission is behind a named adapter seam.
- `paint_root/tail.rs` no longer writes `cx.scene` or constructs `SceneOp::PopClip` directly.
- The retained `PaintCx` binding for root frame tail cleanup scene emission lives in a retained tail
  cleanup binding module.
- Source-policy coverage locks the tail cleanup adapter boundary.

## In Scope

- `paint_root/tail.rs`
- new tail cleanup adapter modules under `paint_root/`
- source-policy coverage in `ecosystem/fret-node/src/lib.rs`
- workstream evidence and gates

## Out Of Scope

- cached node/group/edge internal clip ops,
- cached/immediate pass clip emission,
- overlay paint,
- cache pruning logic,
- root frame clip push emission,
- public scene schema changes.

## Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Tail cleanup pop is the matching root frame clip lifecycle operation. | Confident | `frame_clip_adapter` owns `PushClipRect`; `tail.rs` ends the pass with `SceneOp::PopClip`. | Re-audit frame clip lifecycle before changing cached pass internals. |
| Cached layer `PopClip` ops are separate internals. | Confident | `cached_nodes.rs`, `cached_groups.rs`, and cached edge build-state manage cache-local clip stacks. | Open separate cached layer lanes if they need adapters later. |
| The seam should move scene emission only. | Confident | Tail still owns overlays and pruning sequencing. | Split overlay/prune changes into separate lanes if needed. |

## Architecture Direction

Prefer a narrow action adapter:
`pop_paint_root_tail_clip(cx)`. `tail.rs` should own operation order; the retained binding should
own `SceneOp::PopClip` construction and scene mutation.

## Closeout Condition

This lane can close when root frame tail cleanup pop emission is isolated behind the adapter,
source-policy coverage locks the seam, and validation gates pass.

## Closeout State

Closed on 2026-05-25 with `CLOSEOUT_AUDIT_2026-05-25.md`. Root frame tail cleanup pop emission now
uses the tail cleanup adapter seam. Cached/immediate pass clip emission, cached layer internal clip
ops, grid plan/chrome hint routing, overlays, and pruning remain separate follow-on candidates.
