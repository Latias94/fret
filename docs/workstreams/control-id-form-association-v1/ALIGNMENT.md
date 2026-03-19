# ControlId form association (v1) — alignment

This workstream aligns "form label ↔ control" outcomes across the shadcn ecosystem layer using a
shared `ControlId` + registry pattern:

- `Label::for_control(ControlId)` forwards focus (and optional action) to the registered control.
- Controls that opt into `control_id(ControlId)` register a focus target and (when possible) use
  `aria-labelledby` / `aria-describedby`-like semantics by reading from the registry.

## Layering (non-negotiable)

- Mechanism/contract: `ecosystem/fret-ui-kit`
  - `primitives/control_registry.rs` (`ControlId`, `ControlRegistry`, `register_control`, etc.)
  - `primitives/label.rs` (`Label::for_control`, label click → focus + invoke action)
- Policy/recipes: `ecosystem/fret-ui-shadcn`
  - Individual controls expose `control_id(...)` and register the concrete focusable element id.
  - Controls attach labelled-by / described-by semantics from the registry unless an explicit
    `a11y_label` is provided.

## Expected outcomes (v1)

| Outcome | Definition |
| --- | --- |
| Click label → focus control | `Label::for_control(id)` click requests focus for the registered control entry. |
| Auto `labelled-by` | If the control has `control_id(id)` and does **not** have an explicit `a11y_label`, it uses the registry's label element as `labelled_by_element`. |
| Auto `described-by` | If the control has `control_id(id)`, it uses the registry's description/error element as `described_by_element` (even when `a11y_label` is present). |
| Nested pressables inside label content | Label forwarding should skip embedded pressables inside wrapped label content so nested buttons/links keep ownership of their own click path. |

## Status table

Legend:
- `Yes`: implemented + demo + gate exists
- `Partial`: implemented but missing demo and/or gate
- `No`: not implemented

| Component | Surface | `control_id` | Registers focus target | Auto `labelled-by` | Auto `described-by` | UI Gallery demo | Diag gate | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `Input` | `ecosystem/fret-ui-shadcn/src/input.rs` | Yes | Yes | Yes | Yes | Existing | Existing | Baseline pattern. |
| `Textarea` | `ecosystem/fret-ui-shadcn/src/textarea.rs` | Yes | Yes | Yes | Yes | Existing | Existing | Baseline pattern. |
| `Checkbox` | `ecosystem/fret-ui-shadcn/src/checkbox.rs` | Yes | Yes | Yes | Yes | Existing | Existing | Label click mirrors checkbox activation via the registry (command/payload + toggle when applicable). |
| `Switch` | `ecosystem/fret-ui-shadcn/src/switch.rs` | Yes | Yes | Yes | Yes | Existing | Existing | Label click mirrors switch activation via the registry (command dispatch and/or toggle). |
| `InputGroup` | `ecosystem/fret-ui-shadcn/src/input_group.rs` | Yes | Yes | Yes | Yes | Yes | Yes | Inline addons forward focus correctly. |
| `Select` | `ecosystem/fret-ui-shadcn/src/select.rs` | Partial | Yes | Yes | Yes | Yes | Yes | Focus target is the trigger pressable. |
| `NativeSelect` | `ecosystem/fret-ui-shadcn/src/native_select.rs` | Partial | Yes | Yes | Yes | Yes | Yes | Trigger is a combobox-like pressable. |
| `Slider` | `ecosystem/fret-ui-shadcn/src/slider.rs` | Partial | Yes | Yes | Yes | Yes | Yes | Focus target is the active thumb (`*-thumb-0`). |
| `RadioGroup` | `ecosystem/fret-ui-shadcn/src/radio_group.rs` | Partial | Yes | Yes | Yes | Yes | Yes | Focus target is the active item; adds `{prefix}-item-{idx}` test ids. |
| `Toggle` | `ecosystem/fret-ui-shadcn/src/toggle.rs` | Partial | Yes | Yes | Yes | No | No | Label click now mirrors toggle activation via the registry; UI Gallery / diag coverage is still pending. |
| `ToggleGroup` | `ecosystem/fret-ui-shadcn/src/toggle_group.rs` | Partial | Yes | N/A | N/A | No | No | Focus target is the group's tab-stop item. |
| `Combobox` | `ecosystem/fret-ui-shadcn/src/combobox.rs` | Partial | Yes | Yes | Yes | No | No | Suppresses dynamic label fallback when `control_id` is set. |
| `DatePicker` | `ecosystem/fret-ui-shadcn/src/date_picker.rs` (+ `button.rs`) | Partial | Yes | Yes | Yes | Yes | No | Uses `Button::control_id(...)` for trigger association; UI Gallery label demo exists, dedicated diag gate is still pending. |
| `DateRangePicker` | `ecosystem/fret-ui-shadcn/src/date_range_picker.rs` (+ `button.rs`) | Partial | Yes | Yes | Yes | No | No | Matches `DatePicker` trigger semantics and derives `{prefix}-trigger` / `-content` / `-calendar` anchors. |
| `DatePickerWithPresets` | `ecosystem/fret-ui-shadcn/src/date_picker_with_presets.rs` (+ `button.rs`, `select.rs`) | Partial | Yes | Yes | Yes | No | No | Outer trigger is the form control; derived prefixes also flow into the inner `Select` and `Calendar`. |

## Evidence anchors

- Registry + label primitive:
  - `ecosystem/fret-ui-kit/src/primitives/control_registry.rs`
  - `ecosystem/fret-ui-kit/src/primitives/label.rs`
  - `crates/fret-ui/src/declarative/host_widget/event/pointer_region.rs`
- Representative recipe implementations:
  - `ecosystem/fret-ui-shadcn/src/input.rs`
  - `ecosystem/fret-ui-shadcn/src/select.rs`
  - `ecosystem/fret-ui-shadcn/src/slider.rs`
  - `ecosystem/fret-ui-shadcn/src/radio_group.rs`
  - `ecosystem/fret-ui-shadcn/src/date_picker.rs`
  - `ecosystem/fret-ui-shadcn/src/date_range_picker.rs`
  - `ecosystem/fret-ui-shadcn/src/date_picker_with_presets.rs`
  - `ecosystem/fret-ui-shadcn/src/field.rs`
- UI Gallery demos + scripts (new in this workstream):
  - `apps/fret-ui-gallery/src/ui/snippets/select/label.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/native_select/label.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/slider/label.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/radio_group/label.rs`
  - `tools/diag-scripts/ui-gallery/select/ui-gallery-select-label-click-focus.json`
  - `tools/diag-scripts/ui-gallery/native-select/ui-gallery-native-select-label-click-focus.json`
  - `tools/diag-scripts/ui-gallery/slider/ui-gallery-slider-label-click-focus.json`
  - `tools/diag-scripts/ui-gallery/radio-group/ui-gallery-radio-group-label-click-focus.json`
