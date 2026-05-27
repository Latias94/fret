# Material 3 DatePicker Day Cell Selectors Packet v1

Date: 2026-05-28
Task: M3DCS-020
Component: DatePicker

## Truth

- Existing row/column day-cell selectors stay live.
- Rendered day cells also expose `date_picker.cell.<yyyy-mm-dd>` anchors.
- Date-derived selectors work for docked and modal picker surfaces.
- The ids target calendar values, not current grid position.

## Artifacts

- `ecosystem/fret-ui-material3/src/date_picker.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`

## Wiring

Value-derived ids are stamped as hidden diagnostic anchors next to the semantic day cell. The
existing semantic row/column id remains the primary pressable target.

## Proof

The focused DatePicker automation-surface test asserts row/column ids and representative
date-derived aliases for both docked and modal render paths.

## Residual Risk

This packet does not implement `SelectableDates`, localized spoken labels, or live-region month
announcements.

