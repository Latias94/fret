# Material 3 TimePicker Input Error Packet v1 - TODO

Status: Closed
Last updated: 2026-05-28

Task IDs use `M3TPIE-*`.

## M0 - Packet Setup

- [x] M3TPIE-010 [owner=codex] [deps=none] [scope=docs/workstreams/material3-time-picker-input-error-packet-v1]
  Goal: Split TimePicker input error handling from the broader picker residual-risk list and
  classify it by layer.
  Validation: JSON and catalog gates.
  Review: DONE. The gap maps to Material recipe state and token wiring; existing Fret semantics
  mechanisms are sufficient.
  Evidence: `DESIGN.md`.
  Handoff: Continue with M3TPIE-020.

## M1 - Recipe Error State

- [x] M3TPIE-020 [owner=codex] [deps=M3TPIE-010] [scope=ecosystem/fret-ui-material3/src/time_picker.rs,ecosystem/fret-ui-material3/src/tokens/time_input.rs]
  Goal: Keep invalid editable input separate from committed time, expose invalid semantics, and
  render Material supporting error text.
  Validation: Focused TimePicker input behavior test.
  Review: DONE. Invalid hour/minute values no longer clamp into committed `Time`; supporting text
  is live and field invalid semantics are set only while invalid.
  Evidence: `ecosystem/fret-ui-material3/src/time_picker.rs`.
  Handoff: Continue with M3TPIE-030.

## M2 - Automation Proof And Matrix Closeout

- [x] M3TPIE-030 [owner=codex] [deps=M3TPIE-020] [scope=ecosystem/fret-ui-material3/tests,docs/workstreams/material3-component-alignment-sweep-v1]
  Goal: Prove invalid input, recovery, supporting-text selectors, and picker matrix updates.
  Validation: Focused `radio_alignment` and diagnostics automation tests.
  Review: DONE. Input error handling is closed; the later TimePicker localization follow-on is
  closed by material3-time-picker-string-registry-packet-v1.
  Evidence: `CLOSEOUT_AUDIT_2026-05-28.md`.
  Handoff: Return to the broader Material3 alignment goal.
