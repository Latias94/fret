# Fret Node Paint Root Frame Clip Adapter v1 - TODO

Status: Closed
Last updated: 2026-05-25

## FCA-M0 - Scope Freeze

- [x] FCA-010 [owner=codex] [deps=none] [scope=docs/workstreams/fret-node-paint-root-frame-clip-adapter-v1]
  Goal: Open the narrow follow-on for root frame clip emission only.
  Validation: `python3 -m json.tool docs/workstreams/fret-node-paint-root-frame-clip-adapter-v1/WORKSTREAM.json`
  Evidence: `DESIGN.md`, `EVIDENCE_AND_GATES.md`
  Handoff: Do not reopen the closed frame setup lane.

## FCA-M1 - Clip Adapter Proof

- [x] FCA-020 [owner=codex] [deps=FCA-010] [scope=ecosystem/fret-node/src/ui/canvas/widget/paint_root]
  Goal: Move root frame `SceneOp::PushClipRect` emission behind a minimal clip adapter seam.
  Validation: `cargo test -p fret-node --features compat-retained-canvas paint_root_frame_clip_adapter`
  Evidence: frame clip adapter modules, `paint_root/frame.rs`, source-policy test in
  `ecosystem/fret-node/src/lib.rs`
  Handoff: Keep path-cache diagnostics, background paint, grid paint, cached/immediate passes, and
  tail cleanup out of scope. Complete; root frame clip emission now delegates through the frame clip
  adapter while the retained `PaintCx::scene` binding lives in `frame_clip_retained_cx.rs`.

## Closeout

- [x] FCA-030 [owner=codex] [deps=FCA-020] [scope=docs/workstreams/fret-node-paint-root-frame-clip-adapter-v1]
  Goal: Close the lane and split residual frame paint operation families into follow-on candidates.
  Validation: `python3 tools/check_workstream_catalog.py`; `git diff --check`
  Evidence: `CLOSEOUT_AUDIT_2026-05-25.md`
  Handoff: This lane is closed. Start a separate follow-on for path-cache diagnostics, background
  paint, grid paint, tail cleanup, or cached/immediate pass clip emission.
