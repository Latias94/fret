# Material3 DatePicker Calendar Grid Layout Packet v2

Date: 2026-05-28
Task: M3PV2-028

## Truth

- Calendar content is an intrinsic DatePicker recipe layout, not caller-owned page layout.
- Compose Material3 applies `DatePickerHorizontalPadding = 12.dp` around month navigation,
  weekdays, and the month grid.
- Weekday headers and date cells occupy 48px interactive slots so columns align even when the
  visual date container is 40px.
- The date visual remains token-driven and centered inside the interactive slot.
- Stable DatePicker automation ids should target the slot for row/column layout assertions and the
  date visual for date-specific chrome assertions.

## Sources

- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/DatePicker.kt`
  - `WeekDays` wraps each label in a box sized to `LocalMinimumInteractiveComponentSize`.
  - `Month` rows use evenly arranged day cells and `RecommendedSizeForAccessibility * 6` height.
  - `DatePickerHorizontalPadding = 12.dp`.
- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/DatePickerModalTokens.kt`
  - Modal date visual container is `40.dp`.
- Fret Material foundation already owns the shared minimum-touch-target outcome in
  `ecosystem/fret-ui-material3/src/foundation/interactive_size.rs`.

## Artifacts

- `ecosystem/fret-ui-material3/src/date_picker.rs`
- `ecosystem/fret-ui-material3/src/tokens/date_picker.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `goldens/material3-headless/v1/material3-date-picker.*.json`

## Wiring

- DatePicker body now applies the 12px Material calendar horizontal padding.
- Weekday labels render inside fixed interactive slots and keep their existing a11y labels/test ids.
- Date pressables use fixed interactive slot layout; visual date chrome is centered with
  `foundation::interactive_size::centered_fill`.
- Modal DatePicker no longer lets the panel-level padding override calendar content geometry;
  title and actions get their own padding wrappers.

## Proof

Red before fix:

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_date_picker_exposes_stable_part_test_ids
```

The new gate failed because docked weekday slot bounds were intrinsic text bounds (`10px` wide)
instead of Material's 48px interactive slot.

Green after fix:

```powershell
cargo fmt --package fret-ui-material3
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_date_picker_exposes_stable_part_test_ids material3_date_picker_month_label_is_polite_live_region material3_date_picker_uses_material_string_registry_and_date_descriptions material3_date_picker_respects_selectable_dates
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_date_picker_suite_goldens_v1
cargo nextest run -p fret-ui-material3 --lib date_picker
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
```

## Residual Risk

- DatePicker motion remains seeded; this packet only proves settled calendar layout.
- Year selection and input-mode layout still need their own source-backed v2 packet.
- Date range picker behavior is not covered by this crate's current DatePicker-only surface.
