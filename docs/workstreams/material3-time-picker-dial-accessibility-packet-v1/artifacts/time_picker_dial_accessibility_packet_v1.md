# Material 3 TimePicker Dial Accessibility Packet v1

Date: 2026-05-28
Task: M3TPD-020
Component: TimePicker

## Truth

- The clock dial remains a semantic group with the existing base-derived `clock-dial` id.
- Each rendered hour label exposes a stable value-derived id: `time_picker.clock-dial.hour.<HH>`.
- Each rendered minute label exposes a stable value-derived id: `time_picker.clock-dial.minute.<MM>`.
- The label ids describe semantic dial values, not render indices or layout positions.

## Artifacts

- `ecosystem/fret-ui-material3/src/time_picker.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`

## Wiring

The dial item ids are stamped through the same `PressableA11y` path that already owns the label role,
selected state, and semantic label. No `fret-ui` mechanism change is required.

## Proof

The focused automation surface test renders a docked TimePicker, asserts the parent dial ids, then
asserts all 12-hour labels and all minute-step labels after switching to the minute selector.

## Residual Risk

- 24-hour mode receives the same value-derived helper, but this packet gates the default 12-hour
  dial because that is the currently exposed automation surface.
- Invalid input error text was closed by `material3-time-picker-input-error-packet-v1`.
- Verbose selector/dial spoken labels were closed by
  `material3-time-picker-a11y-labels-packet-v1`.
- Localized TimePicker strings remain a follow-on.
