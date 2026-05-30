# Material3 Select Popup RTL v1 Milestones

Status: Closed
Last updated: 2026-05-30

## Milestone 1: Popup Placement

Status: Complete

Exit criteria:

- Select popup placement reads the resolved Material direction.
- RTL `Start` alignment maps to the physical right edge when the menu is wider than the trigger.

## Milestone 2: Listbox Row Direction

Status: Complete

Exit criteria:

- Listbox row text is evaluated under the resolved direction provider.
- Leading/trailing icon slots visually swap under RTL without changing selection/focus ownership.

## Milestone 3: Closeout

Status: Complete

Exit criteria:

- Focused RTL tests, check, clippy, layering, catalog, and diff checks pass.
- Residual trigger-row and field-family reuse work remains in follow-on notes.
