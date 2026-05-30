# Material 3 DatePicker Selectable Dates Packet v1 - Closeout Audit

Status: Closed
Date: 2026-05-28

## Completion Claim

The DatePicker selectable-date disabling follow-on is closed.

## Requirement Audit

| Requirement | Evidence | Result |
| --- | --- | --- |
| Docked DatePicker accepts a selectable-date predicate | `DockedDatePicker::selectable_dates` | Done |
| Dialog DatePicker accepts the same predicate | `DatePickerDialog::selectable_dates` | Done |
| Disabled dates remain visible and keep value anchors | `material3_date_picker_respects_selectable_dates` checks `m3-date-picker.cell.2026-01-10` | Done |
| Disabled docked dates do not mutate selection | `material3_date_picker_respects_selectable_dates` blocked docked click assertion | Done |
| Allowed docked dates still mutate selection | `material3_date_picker_respects_selectable_dates` allowed docked click assertion | Done |
| Disabled modal dates cannot commit through OK | `material3_date_picker_respects_selectable_dates` modal click + OK assertion | Done |
| No mechanism change required | Existing `PressableProps.enabled = false` supplies disabled semantics and event blocking | Done |

## Layer Audit

- `material_recipe`: owns DatePicker predicate API, day-cell enabled state, and disabled opacity.
- `kit_policy`: unchanged.
- `mechanism`: unchanged; Pressable behavior was sufficient.
- `diagnostics/test_harness`: owns the focused automation-surface proof.

## Residual Risk

- DatePicker locale-aware day/month labels were closed later by
  `docs/workstreams/material3-date-picker-locale-strings-packet-v1/`.
- DatePicker month live-region announcements were closed later by
  `docs/workstreams/material3-date-picker-month-live-region-packet-v1/`.
- No year-level predicate type was added; callers can disable a whole year by returning false for
  each date in that year.
