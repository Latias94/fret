# Workspace TabStrip (Fearless Refactor v1) — Milestone 1 (Foundation)

## Outcome

Lock a stable, testable contract for Workspace TabStrip surface classification, click intent
arbitration, and end-drop insert index.

## Deliverables

- Contract surface documented (surfaces, hit targets, insert index semantics).
- At least one deterministic gate for "drop at end" (`insert_index == tab_count`).
- A baseline "active tab visible" gate.

## Exit criteria

- Nextest tests cover at least:
  - end-drop insert index (no overflow)
  - end-drop insert index (with overflow)
- A diag suite run can be used as evidence (bundle path recorded).

Status:

- Contracts: in progress (see `DESIGN.md`)
- Tests: pending
- Diag gate: pending
