# Fret Node Low-Level Adapter v1 - TODO

Status: Active
Last updated: 2026-05-25

## NLA-M0 - Adapter Target Audit

- [x] NLA-010 [owner=codex] [deps=none] [scope=ecosystem/fret-node/src/ui/canvas/widget]
  Goal: Catalog remaining direct retained context families and pick the first adapter seam.
  Validation: `rg -n "EventCx|LayoutCx|PaintCx|PrepaintCx|SemanticsCx|CommandCx|Widget" ecosystem/fret-node/src/ui/canvas/widget`
  Evidence: `docs/workstreams/fret-node-low-level-adapter-v1/HANDOFF.md`
  Handoff: First seam selected: low-level redraw / paint invalidation / handled / pointer-capture release adapter.

## NLA-M1 - First Adapter Proof

- [x] NLA-020 [owner=codex] [deps=NLA-010] [scope=ecosystem/fret-node/src/ui/canvas/widget]
  Goal: Move one retained context family behind a named node adapter seam.
  Validation: `cargo check -p fret-node --features compat-retained-canvas`
  Evidence: `ecosystem/fret-node/src/ui/canvas/widget/low_level_adapter.rs`, `ecosystem/fret-node/src/ui/canvas/widget/retained_low_level_adapter.rs`, source-policy test in `ecosystem/fret-node/src/lib.rs`.
  Handoff: Remaining retained bindings should migrate one behavior family at a time into named adapters.

## NLA-M2 - Delete Or Quarantine One Retained Edge

- [x] NLA-030 [owner=codex] [deps=NLA-020] [scope=ecosystem/fret-node/src/ui/canvas/widget/wire_drag]
  Goal: Delete or quarantine the old retained edge replaced by the first adapter proof.
  Validation: `cargo check -p fret-node`; `cargo check -p fret-node --features compat-retained-canvas`; `cargo test -p fret-node --features compat-retained-canvas retained_canvas_low_level_adapter_policy_helpers_stay_off_retained_bridge`
  Evidence: `ecosystem/fret-node/src/ui/canvas/widget/wire_drag/commit_cx.rs`, `ecosystem/fret-node/src/ui/canvas/widget/wire_drag/retained_commit_cx.rs`, source-policy test in `ecosystem/fret-node/src/lib.rs`
  Handoff: `WireCommitCx` now inherits low-level redraw / paint invalidation / pointer-capture release operations from `low_level_adapter`.

## Follow-On Candidates

- [ ] NLA-040 [owner=unassigned] [deps=NLA-030] [scope=ecosystem/fret-node/src/ui/canvas/widget]
  Goal: Choose the next behavior-family adapter split for event routing, command dispatch, or paint/prepaint.
  Validation: narrow task-specific source-policy test plus `cargo check -p fret-node --features compat-retained-canvas`
  Evidence: follow-on task or workstream.
  Handoff: Keep this as one behavior family, not a broad retained canvas rewrite.
