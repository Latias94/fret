# Fret Node Low-Level Adapter v1 - Milestones

Status: Closed
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

## M4 - Second Command Dispatch Consumer

Exit criteria:

- One additional command dispatch consumer inherits `CanvasCommandDispatchCx`.
- Its dedicated retained command adapter edge is deleted.
- A source-policy gate prevents the retained edge from returning.

Status: Complete.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/keyboard_shortcuts.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/keyboard_shortcuts_commands.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_keyboard_route.rs`
- `ecosystem/fret-node/src/lib.rs`

## M5 - Closeout And Follow-On Split

Exit criteria:

- Event runtime adapter work is split into its own lane.
- Paint/prepaint adapter work is split into its own lane.
- This lane has a closeout audit and no remaining active tasks.

Status: Complete.

Evidence:

- `docs/workstreams/fret-node-low-level-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `docs/workstreams/fret-node-event-runtime-adapter-v1/`
- `docs/workstreams/fret-node-paint-prepaint-adapter-v1/`
