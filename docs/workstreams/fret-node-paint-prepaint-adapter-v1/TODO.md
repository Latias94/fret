# Fret Node Paint Prepaint Adapter v1 - TODO

Status: Active
Last updated: 2026-05-25

## NPA-M0 - Scope And Evidence Freeze

- [x] NPA-010 [owner=codex] [deps=none] [scope=docs/workstreams/fret-node-paint-prepaint-adapter-v1]
  Goal: Freeze paint/prepaint adapter scope, first proof target, and gates.
  Validation: `python3 -m json.tool docs/workstreams/fret-node-paint-prepaint-adapter-v1/WORKSTREAM.json`
  Evidence: `docs/workstreams/fret-node-paint-prepaint-adapter-v1/DESIGN.md`
  Handoff: First proof should target prepaint cull-window operations before broad paint root work.

## NPA-M1 - Prepaint Cull-Window Adapter Proof

- [x] NPA-020 [owner=codex] [deps=NPA-010] [scope=ecosystem/fret-node/src/ui/canvas/widget]
  Goal: Introduce a named prepaint adapter seam for cull-window bounds, view-state sync, and debug
  recording, isolating retained `PrepaintCx` binding.
  Validation: `cargo check -p fret-node --features compat-retained-canvas`; narrow source-policy test.
  Evidence: `prepaint_cull_window_adapter.rs`, `retained_widget_cull_window.rs`,
  `retained_widget_cull_window_shift.rs`, source-policy test in `ecosystem/fret-node/src/lib.rs`
  Handoff: Complete. Full paint tree migration remains out of scope for this task.

## NPA-M2 - Paint Root Adapter Candidate

- [ ] NPA-030 [owner=unassigned] [deps=NPA-020] [scope=ecosystem/fret-node/src/ui/canvas/widget]
  Goal: Audit whether paint root scene emission has one small adapter seam or should split again.
  Validation: source audit plus `cargo check -p fret-node --features compat-retained-canvas`
  Evidence: `retained_widget_runtime_paint.rs`, `paint_root_helpers.rs`, follow-on note if split.
  Handoff: If more than one operation family is involved, create a narrower follow-on instead.

## NPA-M3 - Closeout Or Follow-On Split

- [ ] NPA-040 [owner=planner] [deps=NPA-030] [scope=docs/workstreams/fret-node-paint-prepaint-adapter-v1]
  Goal: Close this paint/prepaint adapter lane or split the next paint-family follow-on.
  Validation: `python3 tools/check_workstream_catalog.py`; `git diff --check`
  Evidence: `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, optional closeout audit.
  Handoff: Broad paint tree migration remains explicitly out of scope unless split.
