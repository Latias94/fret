# Fret Node Paint Prepaint Adapter v1 - Milestones

Status: Active
Last updated: 2026-05-25

## M0 - Scope And Evidence Freeze

Exit criteria:

- Problem, target state, and non-goals are explicit.
- First proof target is prepaint cull-window behavior.
- Gate set is recorded.

Status: Complete.

Evidence:

- `docs/workstreams/fret-node-paint-prepaint-adapter-v1/DESIGN.md`
- `docs/workstreams/fret-node-paint-prepaint-adapter-v1/TODO.md`

## M1 - Prepaint Cull-Window Adapter Proof

Exit criteria:

- A named retained-agnostic prepaint adapter seam exists.
- Retained `PrepaintCx` binding is isolated.
- Source-policy tests lock cull-window helpers away from retained contexts.

Primary gates:

- `cargo check -p fret-node --features compat-retained-canvas`
- `cargo test -p fret-node --features compat-retained-canvas paint_prepaint_adapter`

Status: Complete.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/prepaint_cull_window_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_cull_window.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_cull_window_shift.rs`
- `ecosystem/fret-node/src/lib.rs`

## M2 - Paint Root Adapter Candidate

Exit criteria:

- Paint root adapter scope is audited after the prepaint proof.
- One small paint operation family is selected, or a narrower follow-on is opened.

Primary gates:

- `cargo check -p fret-node`
- `cargo check -p fret-node --features compat-retained-canvas`

## M3 - Closeout

Exit criteria:

- Evidence gates are fresh.
- Remaining paint/prepaint work is either complete, deferred, or split into a follow-on.
- `WORKSTREAM.json` status is updated.
