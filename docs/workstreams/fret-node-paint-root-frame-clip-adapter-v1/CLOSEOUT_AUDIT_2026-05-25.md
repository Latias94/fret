# Fret Node Paint Root Frame Clip Adapter v1 - Closeout Audit

Status: Closed
Last updated: 2026-05-25

## Verdict

This lane is closed.

It proved the root frame clip scene-emission adapter seam without widening into path-cache
diagnostics, background paint, grid paint, cached/immediate passes, or tail cleanup.

## Shipped State

- `frame_clip_adapter.rs` defines `PaintRootFrameClipCx` and
  `push_paint_root_frame_clip`.
- `frame_clip_retained_cx.rs` is the retained `PaintCx::scene` binding for the root frame
  `SceneOp::PushClipRect`.
- `paint_root/frame.rs` delegates root frame clip emission through the frame clip adapter instead
  of pushing `SceneOp::PushClipRect` directly.
- Source-policy coverage in `ecosystem/fret-node/src/lib.rs` keeps the clip adapter free of
  retained lifecycle context names and verifies the retained binding owns the scene op.

## Split State

The following paint-root frame operation families remain intentionally outside this lane:

- path-cache diagnostics,
- background paint,
- grid paint,
- tail cleanup / `SceneOp::PopClip`,
- cached/immediate pass clip emission.

The next follow-on should choose one operation family. The smallest likely candidates are either
path-cache diagnostics, if the priority is removing retained `window/node/app` diagnostics reads,
or background paint, if the priority is separating policy lookup from scene emission.

## Closeout Evidence

- `docs/workstreams/fret-node-paint-root-frame-setup-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame_clip_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame_clip_retained_cx.rs`
- `ecosystem/fret-node/src/lib.rs`

## Fresh Gates

- `cargo fmt --package fret-node` - passed in FCA-020.
- `cargo test -p fret-node --features compat-retained-canvas paint_root_frame_clip_adapter` -
  passed in FCA-020.
- `cargo check -p fret-node` - passed in FCA-020.
- `cargo check -p fret-node --features compat-retained-canvas` - passed in FCA-020.
- `python3 -m json.tool docs/workstreams/fret-node-paint-root-frame-clip-adapter-v1/WORKSTREAM.json` -
  passed in FCA-020 and rerun for closeout.
- `python3 tools/check_workstream_catalog.py` - passed in FCA-020 and rerun for closeout.
- `python3 tools/check_layering.py` - passed in FCA-020.
- `git diff --check` - passed in FCA-020 and rerun for closeout.

## Residual Risks

- `paint_root/frame.rs` still takes retained `PaintCx` because diagnostics, background paint, and
  grid paint still need retained context access.
- The root clip push and tail `PopClip` now live in different adapter/cleanup owners by design.
  Preserve that split unless a future tail cleanup lane proves a better owner boundary.
