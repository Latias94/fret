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
