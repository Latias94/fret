# Material 3 DatePicker Selectable Dates Packet v1 - TODO

Status: Closed
Last updated: 2026-05-28

Task IDs use `M3DPS-*`.

## M0 - Packet Setup

- [x] M3DPS-010 [owner=codex] [deps=none] [scope=docs/workstreams/material3-date-picker-selectable-dates-packet-v1]
  Goal: Split the DatePicker selectable-date follow-on from the broader picker packet and define
  the recipe/mechanism boundary.
  Validation: JSON and catalog gates.
  Review: DONE. The gap maps to Material recipe state/semantics wiring; existing Pressable
  disabled behavior is sufficient.
  Evidence: `DESIGN.md`.
  Handoff: Continue with M3DPS-020.

## M1 - Recipe API And Grid Behavior

- [x] M3DPS-020 [owner=codex] [deps=M3DPS-010] [scope=ecosystem/fret-ui-material3/src/date_picker.rs]
  Goal: Add a selectable-date predicate to docked and dialog DatePicker surfaces and apply it to
  day-cell enabled/focusable state.
  Validation: Focused material3 automation test.
  Review: DONE. The predicate defaults to all dates, both picker surfaces share the same grid path,
  and blocked cells remain visible while disabled.
  Evidence: `ecosystem/fret-ui-material3/src/date_picker.rs`.
  Handoff: Continue with M3DPS-030.

## M2 - Automation Proof And Matrix Closeout

- [x] M3DPS-030 [owner=codex] [deps=M3DPS-020] [scope=ecosystem/fret-ui-material3/tests/automation_surface.rs,docs/workstreams/material3-component-alignment-sweep-v1]
  Goal: Prove disabled semantics and selection blocking for docked/modal DatePicker, then update
  the component matrix and picker packet residual risk.
  Validation: Focused automation-surface test plus docs/matrix gates.
  Review: DONE. Selectable-date disabling is closed; locale labels and live-region announcements
  remain separate accessibility follow-ons.
  Evidence: `CLOSEOUT_AUDIT_2026-05-28.md`.
  Handoff: Return to the broader Material3 alignment goal.
