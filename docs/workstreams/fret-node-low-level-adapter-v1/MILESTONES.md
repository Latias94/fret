# Fret Node Low-Level Adapter v1 - Milestones

Status: Active
Last updated: 2026-05-25

## M0 - Retained Island Audit

Exit criteria:

- Remaining retained context families are listed.
- One first adapter seam is selected.

Status: Complete.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/low_level_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_low_level_adapter.rs`

## M1 - First Adapter Seam

Exit criteria:

- One behavior family no longer directly depends on retained contexts outside its adapter.
- Default and compat checks pass.

Status: Complete.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `cargo check -p fret-node`
- `cargo check -p fret-node --features compat-retained-canvas`

## M2 - Compatibility Edge Shrink

Exit criteria:

- At least one retained edge is deleted or quarantined with a source-policy gate.
- Next behavior family is split as a narrow task.

Status: Complete.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/wire_drag/commit_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/wire_drag/retained_commit_cx.rs`
- `ecosystem/fret-node/src/lib.rs`

## M3 - Command Dispatch Adapter

Exit criteria:

- Command dispatch has a named retained-context-agnostic adapter.
- One dedicated retained command dispatch edge is deleted.
- A source-policy gate prevents the deleted edge from returning.

Status: Complete.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/command_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_command_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_close_button_cx.rs`
- `ecosystem/fret-node/src/lib.rs`
