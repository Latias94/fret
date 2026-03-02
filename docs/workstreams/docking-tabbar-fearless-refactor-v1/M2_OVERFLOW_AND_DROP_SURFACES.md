# Docking TabBar Fearless Refactor v1 — Milestone 2 (Overflow + Drop Surfaces)

## Outcome

Overflow behaves as a first-class pipeline stage and drop surfaces remain predictable even when tabs
are hidden (dropdown / scroll).

## Deliverables

- Overflow pipeline produces:
  - visible tabs
  - overflow tabs
  - canonical index mapping
- Explicit “end drop surface” exists conceptually (geometry) and is gated:
  - dropping there resolves to `insert_index == tab_count` in canonical order
- Overflow control surfaces are not drop surfaces:
  - the overflow button is excluded
  - the reserved header space is treated as an end-drop surface
- Selecting a tab from the overflow menu scrolls the strip to make the active tab visible.
- Overflow menu rows expose a close affordance; clicking close does not activate the tab.
- Auto-scroll on drag near edges is hardened and diag-gated.

## Exit criteria

- Overflow diag gate stays green:
  - `insert_index == 10` for a 10-tab stack in an overflow layout scenario.
- Edge auto-scroll diag gate stays green:
  - `dock_tab_strip_active_scroll_px_ge` becomes true while parking the pointer at the strip edge.
- No regressions in non-overflow drop behavior.
- At least one integration test goes through `InternalDrag` and asserts the emitted `DockOp::MovePanel.insert_index`
  for an overflowed tab bar end-drop.
