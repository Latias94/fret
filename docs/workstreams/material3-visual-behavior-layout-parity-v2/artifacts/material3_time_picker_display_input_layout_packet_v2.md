# Material3 TimePicker Display Input Layout Packet v2

Date: 2026-05-28
Task: M3PV2-029

## Truth

- TimePicker display and input structure is intrinsic Material recipe layout, not caller-owned page
  layout.
- Compose Material3 display mode lays out hour selector, fixed display separator, minute selector,
  and vertical period selector in one row before the dial.
- Display mode uses 96x80 time selectors, a 24x80 separator slot, a 52x80 period selector, and a
  12px start margin before the period selector.
- Input mode uses 96x72 fields, a 24x72 separator slot, a 52x72 period selector, and the same 12px
  period margin with top alignment.
- The 256px clock dial is centered in the picker chrome and no longer competes with the period
  selector for horizontal row layout.

## Sources

- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/TimePicker.kt`
  - `VerticalClockDisplay` renders `ClockDisplayNumbers` plus a vertical period toggle with
    `Modifier.padding(start = PeriodToggleMargin)`.
  - `ClockDisplayNumbers` uses time selectors sized by `TimeSelectorContainerWidth` and
    `TimeSelectorContainerHeight`, with `DisplaySeparator` in a fixed `24.dp` width slot.
  - `VerticalTimePicker` renders the clock display, then the `ClockFace` dial sized by
    `ClockDialContainerSize`.
  - `TimeInputImpl` uses fixed hour/minute text fields, the same separator slot, and a period
    toggle with `PeriodToggleMargin`.
- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/TimePickerTokens.kt`
  - `ClockDialContainerSize = 256.dp`.
  - `TimeSelectorContainerWidth = 96.dp`, `TimeSelectorContainerHeight = 80.dp`.
  - `PeriodSelectorVerticalContainerWidth = 52.dp`, `PeriodSelectorVerticalContainerHeight = 80.dp`.
- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/TimeInputTokens.kt`
  - `TimeFieldContainerWidth = 96.dp`, `TimeFieldContainerHeight = 72.dp`.
  - `PeriodSelectorContainerWidth = 52.dp`, `PeriodSelectorContainerHeight = 72.dp`.

## Artifacts

- `ecosystem/fret-ui-material3/src/time_picker.rs`
- `ecosystem/fret-ui-material3/src/tokens/time_picker.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `goldens/material3-headless/v1/material3-time-picker.*.json`

## Wiring

- Display mode now renders the period selector in the display row instead of beside the clock dial.
- The display separator and input separator are fixed centered slots with stable part ids:
  `separator` and `input.separator`.
- The display row uses zero flex gap; the only period spacing is the Material 12px start padding.
- Input mode uses zero flex gap, fixed separator size, 12px period start padding, and top row
  alignment.
- The clock dial is wrapped in a full-width centered row so its 256px container aligns with the
  TimePicker chrome center.

## Proof

Red before fix:

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_time_picker_exposes_stable_part_test_ids
```

The new gate failed because the period selector was beside the dial (`y = 304px`) instead of in
the display row (`y = 120px`).

Green after fix:

```powershell
cargo fmt --package fret-ui-material3
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_time_picker_exposes_stable_part_test_ids material3_time_picker_uses_compose_aligned_accessibility_labels material3_time_picker_uses_material_string_registry
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_time_picker_suite_goldens_v1
cargo nextest run -p fret-ui-material3 --lib time_picker
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
```

## Residual Risk

- TimePicker modal open/close motion is covered later by
  `material3_time_picker_modal_motion_packet_v2.md`; clock-face selector/crossfade motion remains
  open. This packet only proves settled display/input/dial geometry.
- Horizontal landscape TimePicker layout is not separately proven by this vertical/default packet.
- Touch target and dial label behavior were already covered by existing automation/a11y gates; this
  packet does not add a new pointer interaction matrix.
