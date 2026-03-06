# ControlId form association (v1) - TODO

## Remaining coverage

| Component | Missing | Suggested owner layer | Gate |
| --- | --- | --- | --- |
| `Toggle` | UI Gallery demo + diag script | `apps/fret-ui-gallery` + `tools/diag-scripts` | `focus_is` + optionally `pressed_state` invariant |
| `ToggleGroup` | UI Gallery demo + diag script | `apps/fret-ui-gallery` + `tools/diag-scripts` | `focus_is` on tab-stop item |
| `Combobox` | UI Gallery demo + diag script | `apps/fret-ui-gallery` + `tools/diag-scripts` | `focus_is` on trigger |
| `CommandPalette` | Add shared `test_id_prefix(...)` convenience surface | `ecosystem/fret-ui-shadcn` | derived `input/listbox/item-/heading-` ids |
| `DropdownMenu` / `Menubar` | Normalize menu trigger/content/item prefix conventions | `ecosystem/fret-ui-shadcn` | stable trigger/content/item ids in diagnostics |
| `DatePicker` | UI Gallery demo + diag script | `apps/fret-ui-gallery` + `tools/diag-scripts` | `focus_is` on trigger |
| `DateRangePicker` | `control_id` wiring (if used as a form control) | `ecosystem/fret-ui-shadcn` | label click focuses trigger(s) |
| `InputOTP` | decide mapping (first cell vs active cell) | `ecosystem/fret-ui-shadcn` | label click focuses active cell |

## Refactor opportunities (after gates exist)

| Item | Why | Notes |
| --- | --- | --- |
| Extract a small helper for registry reads | Avoid repeating `label_for` / `described_by_for` boilerplate | Keep it in `ecosystem/*` (policy). |
| Normalize `test_id` conventions for focus targets | Keep scripts robust during refactors | Prefer `{prefix}-trigger`, `{prefix}-thumb-0`, `{prefix}-item-0`. |
| Consolidate prefix-only authoring surfaces | Reduce explicit `trigger_test_id(...)` boilerplate in demos | Prioritize `CommandPalette`, then menu-family recipes. |