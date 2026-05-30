# `fret-node` Retained Canvas Mirror Cleanup (v1) - TODO

Status: complete
Last updated: 2026-05-27

Task IDs use `NCM` for retained node-canvas mirrors.

## M0 - Scope And Evidence Freeze

- [x] NCM-010 [owner=planner] [deps=none] [scope=docs/workstreams/fret-node-retained-canvas-mirror-cleanup-v1]
  Goal: Freeze problem, target state, non-goals, and retained compatibility gates.
  Validation: `python3 -m json.tool docs/workstreams/fret-node-retained-canvas-mirror-cleanup-v1/WORKSTREAM.json`
  Evidence: `DESIGN.md`, `EVIDENCE_AND_GATES.md`
  Handoff: First implementation slice should quarantine retained canvas model mirrors, not delete
  public compatibility constructors.

## M1 - Retained Canvas Mirror Owner

- [x] NCM-020 [owner=codex] [deps=NCM-010] [scope=ecosystem/fret-node/src/ui/canvas/widget.rs,ecosystem/fret-node/src/ui/canvas/widget/widget_surface,ecosystem/fret-node/src/ui/canvas/widget/view_state,ecosystem/fret-node/src/surface_policy_tests.rs]
  Goal: Move retained canvas graph/view/editor-config mirror model fields behind a private
  `NodeGraphCanvasMirrors` owner while preserving retained constructor and store-sync behavior.
  Validation: `cargo nextest run -p fret-node --features compat-retained-canvas retained_canvas_mirror_owner`
  Review: Ensure this is a mirror-boundary cleanup, not a public API expansion.
  Evidence: `ecosystem/fret-node/src/ui/canvas/widget.rs`,
  `ecosystem/fret-node/src/surface_policy_tests.rs`
  Completion: `NodeGraphCanvasWith` now stores graph/view/editor-config model mirrors in private
  `NodeGraphCanvasMirrors`; focused policy and compatibility gates passed on 2026-05-27.

## M2 - Store-First Retained Sync Audit

- [x] NCM-030 [owner=codex] [deps=NCM-020] [scope=ecosystem/fret-node/src/ui/canvas/widget]
  Goal: Audit retained canvas store-backed sync after mirror quarantine and delete or narrow one
  redundant mirror update path if compatibility gates prove it is safe.
  Validation: `cargo nextest run -p fret-node --features compat-retained-canvas store_backed retained`
  Review: Split a follow-on if deleting a mirror changes retained app-observable behavior.
  Evidence: `EVIDENCE_AND_GATES.md`
  Completion: Removed the unused `commit_legacy` retained canvas transaction pipeline after
  source-policy coverage proved the duplicate mirror writer was gone; current `commit` remains the
  single retained transaction path.

## M3 - Closeout

- [x] NCM-040 [owner=planner] [deps=NCM-030] [scope=docs/workstreams/fret-node-retained-canvas-mirror-cleanup-v1]
  Goal: Verify the lane, record remaining retained compatibility risks, and close or split a
  narrower follow-on.
  Validation: `cargo fmt --check`; `cargo nextest run -p fret-node --features compat-retained-canvas retained_canvas_mirror_owner`; `cargo check -p fret-node --features compat-retained-canvas`; `cargo check -p fret-node --no-default-features`; `python3 tools/check_layering.py`
  Review: Use `verify-rust-workstream` before marking complete.
  Evidence: `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, closeout audit.
  Completion: Closeout gates passed and `CLOSEOUT_AUDIT_2026-05-27.md` records the shipped retained
  mirror cleanup.
