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
- Auto-scroll on drag near edges is hardened and (ideally) diag-gated (docking already has unit coverage).

## Exit criteria

- Overflow diag gate stays green:
  - `insert_index == 10` for a 10-tab stack in an overflow layout scenario.
- No regressions in non-overflow drop behavior.
