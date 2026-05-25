# Fret Node Paint Root Frame Background Adapter v1 - Closeout Audit

Status: Closed
Last updated: 2026-05-25

## Verdict

This lane is closed.

It proved the paint-root frame background scene-emission adapter seam without widening into chrome
hint policy, grid paint, tail cleanup, path-cache diagnostics, cached/immediate passes, grid tile
diagnostics, or edge label budget diagnostics.

## Shipped State

- `frame_background_adapter.rs` defines `PaintRootFrameBackgroundCx` and
  `paint_root_frame_background`.
- `frame_background_retained_cx.rs` is the retained `PaintCx` binding for background
  `SceneOp::Quad` emission.
- `frame/background.rs` now owns only canvas chrome hint resolution, fallback background color
  selection, and delegation to the background adapter.
- Source-policy coverage in `ecosystem/fret-node/src/lib.rs` keeps the background adapter free of
  retained lifecycle context names and `SceneOp`, verifies the retained binding owns scene emission,
  and verifies chrome hint policy stays in `frame/background.rs`.

## Split State

The following paint-root frame operation families remain intentionally outside this lane:

- grid paint,
- tail cleanup / `SceneOp::PopClip`,
- cached/immediate pass clip emission,
- grid tile diagnostics and edge label budget diagnostics,
- chrome hint routing policy.

The next follow-on should choose one operation family. The smallest likely candidate is grid paint
because it still combines grid plan policy, retained paint context, cache warming, tile diagnostics,
and scene emission.

## Closeout Evidence

- `docs/workstreams/fret-node-paint-root-frame-diagnostics-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame/background.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame_background_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame_background_retained_cx.rs`
- `ecosystem/fret-node/src/lib.rs`

## Fresh Gates

- `cargo fmt --package fret-node` - passed.
- `cargo test -p fret-node --features compat-retained-canvas paint_root_frame_background_adapter` -
  passed.
- `cargo check -p fret-node` - passed.
- `cargo check -p fret-node --features compat-retained-canvas` - passed.
- `python3 -m json.tool docs/workstreams/fret-node-paint-root-frame-background-adapter-v1/WORKSTREAM.json` -
  passed.
- `python3 tools/check_workstream_catalog.py` - passed.
- `python3 tools/check_layering.py` - passed.
- `git diff --check` - passed.

## Residual Risks

- `paint_root/frame.rs` still takes retained `PaintCx` because grid paint and remaining tail work
  still need retained context access.
- Grid paint still owns the broadest remaining frame operation family and should not be folded into
  this closed background lane.
