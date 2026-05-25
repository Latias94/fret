# Fret Node Paint Root Frame Clip Adapter v1

Status: Active
Last updated: 2026-05-25

## Why This Lane Exists

`fret-node-paint-root-frame-setup-adapter-v1` proved the first narrow frame setup seam by moving
bounds/viewport/render-cull route inputs behind a retained-agnostic adapter. Its closeout explicitly
left clip scene emission as the smallest next operation-family follow-on.

## Relevant Authority

- `docs/workstreams/fret-node-paint-root-frame-setup-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `docs/workstreams/fret-node-paint-root-frame-setup-adapter-v1/FRAME_SETUP_SCOPE_AUDIT_2026-05-25.md`
- `docs/adr/0330-retained-runtime-internal-and-compat-surface.md`
- `docs/adr/0128-canvas-widgets-and-interactive-surfaces.md`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame.rs`

## Problem

`prepare_paint_root_frame` still writes the root frame clip directly through retained
`PaintCx::scene`:

- `cx.scene.push(SceneOp::PushClipRect { rect: viewport_rect })`

That is a distinct scene emission family from viewport calculation, diagnostics, background paint,
grid paint, cached/immediate passes, and tail cleanup.

## Target State

- Root frame clip push is behind a named adapter seam.
- The retained `PaintCx::scene` binding for this root frame clip lives in a retained binding module.
- `frame.rs` no longer directly emits `SceneOp::PushClipRect` for the root frame clip.
- Source-policy coverage prevents the clip adapter from depending on retained lifecycle context
  names.

## In Scope

- `paint_root/frame.rs`
- new frame clip adapter modules under `paint_root/`
- source-policy coverage in `ecosystem/fret-node/src/lib.rs`
- workstream evidence and gates

## Out Of Scope

- path-cache diagnostics,
- background paint,
- grid paint,
- cached/immediate pass clip emission,
- `paint_root/tail.rs` cleanup and `SceneOp::PopClip`,
- renderer scene semantics or public scene contracts.

## Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Root frame clip push is a separate operation family. | Confident | `FRAME_SETUP_SCOPE_AUDIT_2026-05-25.md` lists clip scene emission separately. | Re-audit before implementing a broad scene adapter. |
| The first seam should push only `SceneOp::PushClipRect`. | Confident | `frame.rs` has one root frame clip push between viewport prep and background/grid paint. | Split again rather than moving background/grid scene writes. |
| Tail `PopClip` should not move with this slice. | Likely | `paint_root/tail.rs` owns tail cleanup and was out of scope in the parent lane. | If stack balance becomes hard to reason about, open a tail cleanup follow-on. |

## Architecture Direction

Prefer a narrow action adapter: `push_paint_root_frame_clip(cx, viewport_rect)`. The adapter should
not compute viewport geometry, read diagnostics state, or paint background/grid content.

## Closeout Condition

This lane can close when the root frame clip push is isolated behind the adapter, source-policy
coverage locks the seam, and the validation gates pass.
