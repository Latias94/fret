# TimePicker Accessibility Labels Packet v1

## Truth

- `time_picker.hour-selector` and `time_picker.minute-selector` are `RadioButton` semantics nodes.
- The selectors expose selected state and spoken values, not only visible two-digit text.
- Clock dial value labels keep stable value-derived ids while exposing spoken hour/minute labels.
- `time_picker.period` and `time_picker.input.period` group AM/PM controls under `Select AM or PM`.

## Artifacts

- `ecosystem/fret-ui-material3/src/time_picker.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`

## Wiring

The alignment is wired through existing `PressableA11y` fields and `SemanticsDecoration`; no
runtime contract was widened.

## Proof

`material3_time_picker_uses_compose_aligned_accessibility_labels` renders docked dial and input
TimePicker surfaces and asserts role, label, value, selected-state, and period-group outcomes.

## Residual Risk

Localized Material strings remain a follow-on.
