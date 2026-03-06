# ControlId form association (v1) - milestones

## Milestones

| Milestone | Goal | Exit criteria |
| --- | --- | --- |
| M0 | Shared mechanism stable | `ControlId` registry + `Label::for_control` contract is stable and documented. |
| M1 | Core form controls covered | Input/Textarea/Checkbox/Switch/InputGroup/Select/NativeSelect/Slider/RadioGroup accept `control_id` and register focus targets. |
| M2 | Composite triggers covered | Combobox/DatePicker triggers can be labeled via `ControlId` without dynamic a11y label drift. |
| M3 | Regression gates | Each covered control has a UI Gallery demo + a minimal `fretboard diag` script that asserts label click -> focus. |
| M4 | Stable automation anchors | High-churn composite widgets (`Select`, `NativeSelect`, `Tabs`, `Combobox`, then `CommandPalette`) expose prefix-only stable child `test_id` conventions. |

## Definition of done (v1)

- For each covered control:
  - `control_id(...)` surface exists on the recipe component.
  - Control registers a `ControlEntry` targeting a concrete focusable element id.
  - When the control does not set an explicit `a11y_label`, it prefers `labelled-by` from the registry.
  - `described-by` is wired via registry (description/error).
  - UI Gallery has a copy-pastable snippet demonstrating the pattern.
  - At least one diag script gates the core behavior (`focus_is`).