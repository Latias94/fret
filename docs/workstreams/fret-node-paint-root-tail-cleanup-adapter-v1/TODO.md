# Fret Node Paint Root Tail Cleanup Adapter v1 - TODO

Status: Closed
Last updated: 2026-05-25

## TCA-M0 - Scope Freeze

- [x] TCA-010 [owner=codex] [deps=none] [scope=docs/workstreams/fret-node-paint-root-tail-cleanup-adapter-v1]
  Goal: Open a narrow follow-on for root frame tail cleanup pop emission only.
  Validation: `python3 -m json.tool docs/workstreams/fret-node-paint-root-tail-cleanup-adapter-v1/WORKSTREAM.json`
  Evidence: `DESIGN.md`, `EVIDENCE_AND_GATES.md`
  Handoff: Keep cached layer internal clip ops, cached/immediate passes, overlays, and cache pruning
  out of scope.

## TCA-M1 - Tail Cleanup Adapter Seam

- [x] TCA-020 [owner=codex] [deps=TCA-010] [scope=ecosystem/fret-node/src/ui/canvas/widget/paint_root]
  Goal: Move root frame tail cleanup `PopClip` emission behind a tail cleanup adapter plus retained
  `PaintCx` binding.
  Validation: `cargo test -p fret-node --features compat-retained-canvas paint_root_tail_cleanup_adapter`
  Evidence: `tail.rs`, `tail_cleanup_adapter.rs`, `tail_cleanup_retained_cx.rs`,
  `ecosystem/fret-node/src/lib.rs`
  Handoff: Do not move cached layer `PopClip` ops or cached/immediate pass clip emission in this
  slice. Complete; retained root frame pop emission lives in `tail_cleanup_retained_cx.rs`.

## TCA-M2 - Closeout

- [x] TCA-030 [owner=codex] [deps=TCA-020] [scope=docs/workstreams/fret-node-paint-root-tail-cleanup-adapter-v1]
  Goal: Close the lane and split remaining frame operation families into follow-on candidates.
  Validation: `python3 tools/check_workstream_catalog.py`; `git diff --check`
  Evidence: `CLOSEOUT_AUDIT_2026-05-25.md`
  Handoff: Start a separate follow-on for cached/immediate pass clip emission or grid plan/chrome
  hint routing.
