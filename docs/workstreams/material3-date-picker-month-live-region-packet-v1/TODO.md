# Material 3 DatePicker Month Live Region Packet v1 - TODO

Status: Closed
Last updated: 2026-05-28

Task IDs use `M3DPMLR-*`.

## M0 - Packet Setup

- [x] M3DPMLR-010 [owner=codex] [deps=none] [scope=docs/workstreams/material3-date-picker-month-live-region-packet-v1]
  Goal: Split DatePicker month-label live-region coverage from the broader locale/a11y residual
  risk.
  Validation: JSON and catalog gates.
  Review: DONE. The gap maps to Material recipe semantics decoration; existing Fret live-region
  mechanisms are sufficient.
  Evidence: `DESIGN.md`.
  Handoff: Continue with M3DPMLR-020.

## M1 - Recipe Semantics

- [x] M3DPMLR-020 [owner=codex] [deps=M3DPMLR-010] [scope=ecosystem/fret-ui-material3/src/date_picker.rs]
  Goal: Attach stable part ids and polite atomic live-region semantics to docked/modal month labels.
  Validation: Focused DatePicker automation test.
  Review: DONE. The month label text remains the accessible label and updates after navigation.
  Evidence: `ecosystem/fret-ui-material3/src/date_picker.rs`.
  Handoff: Continue with M3DPMLR-030.

## M2 - Gate And Matrix Closeout

- [x] M3DPMLR-030 [owner=codex] [deps=M3DPMLR-020] [scope=ecosystem/fret-ui-material3/tests/automation_surface.rs,docs/workstreams/material3-component-alignment-sweep-v1]
  Goal: Prove live-region semantics and update the component matrix/picker packet residual risk.
  Validation: Focused automation-surface tests plus docs gates.
  Review: DONE. DatePicker live-region coverage is narrowed to the month label; localization remains
  a separate follow-on.
  Evidence: `CLOSEOUT_AUDIT_2026-05-28.md`.
  Handoff: Return to the broader Material3 alignment goal.
