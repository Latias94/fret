# Material 3 TimePicker Accessibility Labels Packet v1

Date: 2026-05-28
Component: TimePicker
Status: closed

## Truth

- Dial hour/minute selectors expose Compose-aligned selectable semantics: `RadioButton`, selected
  state, and labels `Select hour` / `Select minutes`.
- Dial selector values expose spoken value text separate from visible text: `<hour> o'clock` or
  `<hour> hours` for 24h mode, and `<minute> minutes`.
- Clock dial value items keep stable value-derived ids while their accessibility labels use the
  same spoken hour/minute strings.
- AM/PM period controls are grouped under a recipe-level `Select AM or PM` semantic container for
  docked dial and input modes.

## Source Axis

- Axis: semantics.
- Primary reference: Compose Material3 `TimePicker.kt` and `TimePickerTest.kt`.
- Supporting reference: local Fret semantics contract via `PressableA11y` and
  `SemanticsDecoration`.

## Layer Mapping

- `material_recipe`: TimePicker owns its part ids, labels, roles, values, and period grouping.
- `kit_policy`: no overlay/focus or shared interaction policy change was needed.
- `mechanism`: existing `fret-ui` semantics roles, values, selected state, and group labels were
  sufficient.

## Residual Risk

- Strings were still English literals when this packet closed. That later localization/string
  registry work is closed by `material3-time-picker-string-registry-packet-v1`.
