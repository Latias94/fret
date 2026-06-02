# Material3 Layout Direction Provider Bridge v1 Milestones

Status: Closed
Last updated: 2026-05-30

## Milestone 1: Foundation Bridge

Exit criteria:

- Explicit Material layout direction scopes install the core `LayoutDirection` provider.
- A helper exists for components to resolve Material fallback direction and provide it to descendants.
- Tests prove element metadata captures the provided direction.

Status: Complete.

## Milestone 2: Consumer Proof

Exit criteria:

- Tabs uses the resolved Material direction bridge around its horizontal row subtree.
- RTL theme rendering produces mirrored physical tab order.

Status: Complete.

## Milestone 3: Closeout

Exit criteria:

- Targeted tests and quality gates pass.
- Workstream docs name residual follow-ons instead of widening this lane.

Status: Complete.
