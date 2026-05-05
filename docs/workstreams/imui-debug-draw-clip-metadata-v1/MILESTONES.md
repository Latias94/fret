# ImUi Debug Draw Clip Metadata v1 Milestones

Status: Closed.

## M0 - Source-Level Clip Summary

Exit criteria:

- Command summaries include effective source-level clip rect and depth.
- List summaries include maximum and final clip depth.

Result: Complete.

## M1 - Channel-Order Semantics

Exit criteria:

- Clip state is simulated after active channel split ordering is flattened.
- No mutation or implicit merge is required for introspection.

Result: Complete.

## M2 - Evidence

Exit criteria:

- Unit tests cover nested clip metadata.
- Public smoke tests compile against the new fields.
- Workstream/audit indexes record that `ImDrawCmd::ClipRect`-style source metadata is no longer a
  total gap.

Result: Complete.
