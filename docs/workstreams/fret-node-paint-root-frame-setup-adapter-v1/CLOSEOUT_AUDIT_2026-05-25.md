# Fret Node Paint Root Frame Setup Adapter v1 - Closeout Audit

Status: Closed
Last updated: 2026-05-25

## Verdict

This lane is closed.

It audited paint-root frame setup operation families and proved the first narrow adapter seam:
bounds/viewport/render-cull route inputs now cross a retained-agnostic frame viewport adapter.

## Shipped State

- `frame_viewport_adapter.rs` defines `PaintRootFrameViewportCx` and
  `prepare_paint_root_frame_viewport`.
- `frame_viewport_retained_cx.rs` is the retained `PaintCx::bounds` binding.
- `paint_root/frame.rs` delegates viewport preparation through the frame viewport adapter instead
  of reading `cx.bounds` directly for viewport/render-cull route inputs.
- Source-policy coverage in `ecosystem/fret-node/src/lib.rs` keeps the frame viewport adapter free
  of retained lifecycle context names and verifies the retained binding remains explicit.

## Split State

The following frame setup operation families remain intentionally outside this lane:

- path-cache diagnostics,
- clip scene emission,
- background paint,
- grid paint.

The next follow-on should pick one of those operation families. The smallest next candidate is clip
scene emission because it only needs a scene sink for `SceneOp::PushClipRect` and should not absorb
diagnostics, background, or grid paint.

## Closeout Evidence

- `docs/workstreams/fret-node-paint-root-frame-setup-adapter-v1/FRAME_SETUP_SCOPE_AUDIT_2026-05-25.md`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame_viewport_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame_viewport_retained_cx.rs`
- `ecosystem/fret-node/src/lib.rs`

## Fresh Gates

- `cargo fmt --package fret-node` - passed in FSA-030.
- `cargo test -p fret-node --features compat-retained-canvas paint_root_frame_viewport_adapter` -
  passed in FSA-030.
- `cargo check -p fret-node` - passed in FSA-030.
- `cargo check -p fret-node --features compat-retained-canvas` - passed in FSA-030.
- `python3 tools/check_layering.py` - passed in FSA-030.
- `python3 tools/check_workstream_catalog.py` - passed in FSA-030 and rerun for closeout.
- `python3 -m json.tool docs/workstreams/fret-node-paint-root-frame-setup-adapter-v1/WORKSTREAM.json` -
  passed in FSA-030 and rerun for closeout.
- `git diff --check` - passed in FSA-030 and rerun for closeout.

## Residual Risks

- `paint_root/frame.rs` still takes retained `PaintCx` because diagnostics, scene emission,
  background, and grid paint remain in the retained paint path.
- A broad frame adapter would still be the wrong next step; split follow-ons by operation family.
