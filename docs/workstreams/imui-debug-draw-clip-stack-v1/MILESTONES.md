# ImUi Debug Draw Clip Stack v1 Milestones

Status: Closed
Last updated: 2026-05-04

## M0 - Command Surface

Closed on 2026-05-04.

- `ImUiDebugDrawList::push_clip_rect`
- `ImUiDebugDrawList::pop_clip_rect`

## M1 - Scene Lowering

Closed on 2026-05-04.

- Push lowers to `SceneOp::PushClipRect`.
- Pop lowers to `SceneOp::PopClip`.
- The paint loop auto-balances unmatched debug-draw pushes.

## M2 - Proof

Closed on 2026-05-04.

- Focused debug-draw tests pass.
- Public smoke compile test exercises clip stack API.
- Adapter seam and response-contract smoke tests still pass.
