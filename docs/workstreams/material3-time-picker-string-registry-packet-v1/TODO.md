# Material 3 TimePicker String Registry Packet v1 - TODO

Status: Closed
Last updated: 2026-05-28

Task IDs use `M3TPSTR-*`.

## M0 - Packet Setup

- [x] M3TPSTR-010 [owner=codex] [deps=none] [scope=docs/workstreams/material3-time-picker-string-registry-packet-v1]
  Goal: Split the TimePicker localization follow-on from the broader picker residual-risk list and
  classify the owner layer.
  Validation: JSON and catalog gates.
  Review: DONE. The gap maps to Material foundation plus TimePicker recipe consumption; existing
  runtime i18n mechanisms are sufficient.
  Evidence: `DESIGN.md`.
  Handoff: Continue with M3TPSTR-020.

## M1 - Material String Foundation

- [x] M3TPSTR-020 [owner=codex] [deps=M3TPSTR-010] [scope=ecosystem/fret-ui-material3/src/foundation/strings.rs,ecosystem/fret-ui-material3/src/time_picker.rs,ecosystem/fret-bootstrap/src/lib.rs]
  Goal: Add Material-owned string lookup helpers, route TimePicker strings through them, and seed
  bootstrap default Fluent resources.
  Validation: Focused TimePicker registry test and bootstrap i18n test.
  Review: DONE. The TimePicker recipe now consumes typed Material string helpers and preserves
  English fallback behavior when no app lookup exists.
  Evidence: `ecosystem/fret-ui-material3/src/foundation/strings.rs`.
  Handoff: Continue with M3TPSTR-030.

## M2 - Proof And Matrix Closeout

- [x] M3TPSTR-030 [owner=codex] [deps=M3TPSTR-020] [scope=ecosystem/fret-ui-material3/tests,docs/workstreams/material3-component-alignment-sweep-v1]
  Goal: Prove registry wiring through automation semantics, close the TimePicker localization
  follow-on, and update the Material3 matrix.
  Validation: Focused automation, input-error regression, bootstrap i18n, JSON, catalog, and diff
  checks.
  Review: DONE. TimePicker localization registry wiring is closed; DatePicker locale-aware date
  descriptions remain separate.
  Evidence: `CLOSEOUT_AUDIT_2026-05-28.md`.
  Handoff: Return to the broader Material3 alignment goal.
