# Fret Node Paint Root Frame Background Adapter v1 - TODO

Status: Closed
Last updated: 2026-05-25

## FBA-M0 - Scope Freeze

- [x] FBA-010 [owner=codex] [deps=none] [scope=docs/workstreams/fret-node-paint-root-frame-background-adapter-v1]
  Goal: Open a narrow follow-on for background scene emission only.
  Validation: `python3 -m json.tool docs/workstreams/fret-node-paint-root-frame-background-adapter-v1/WORKSTREAM.json`
  Evidence: `DESIGN.md`, `EVIDENCE_AND_GATES.md`
  Handoff: Keep chrome-hint policy, grid paint, tail cleanup, cached/immediate passes, and
  diagnostics out of scope.

## FBA-M1 - Background Adapter Seam

- [x] FBA-020 [owner=codex] [deps=FBA-010] [scope=ecosystem/fret-node/src/ui/canvas/widget/paint_root]
  Goal: Move background scene quad emission behind a frame background adapter plus retained
  `PaintCx` binding.
  Validation: `cargo test -p fret-node --features compat-retained-canvas paint_root_frame_background_adapter`
  Evidence: `frame/background.rs`, `frame_background_adapter.rs`, `frame_background_retained_cx.rs`,
  `ecosystem/fret-node/src/lib.rs`
  Handoff: The adapter should receive viewport rect and resolved background color; do not move grid
  paint or chrome-hint policy in this slice. Complete; background scene quad emission lives in
  `frame_background_retained_cx.rs`.

## FBA-M2 - Closeout

- [x] FBA-030 [owner=codex] [deps=FBA-020] [scope=docs/workstreams/fret-node-paint-root-frame-background-adapter-v1]
  Goal: Close the lane and split residual frame paint operation families into follow-on candidates.
  Validation: `python3 tools/check_workstream_catalog.py`; `git diff --check`
  Evidence: `CLOSEOUT_AUDIT_2026-05-25.md`
  Handoff: Start a separate follow-on for grid paint, tail cleanup, cached/immediate pass clip
  emission, grid tile diagnostics, edge label budget diagnostics, or chrome hint routing.
