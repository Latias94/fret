# ImUi Color Edit Numeric Readout v1 Closeout Audit - 2026-05-04

Status: Closed.

This lane closes a bounded `ColorEdit4` display gap by making editor `ColorEdit` show current RGB
and HSV numeric values inside the popup.

## What Shipped

- Added `color_numeric_readout`, `rgb_numeric_text`, and `hsv_numeric_text`.
- Rendered the readout between the HSV picker controls and preset swatches.
- Included alpha percent in the RGB readout when `show_alpha=true`.
- Added formatting tests and source-policy guards.

## Proof

- `cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast` passes focused
  color edit tests, including the numeric readout formatting test.
- `cargo nextest run -p fret-ui-editor --features imui --test imui_surface_policy --no-fail-fast`
  passes the editor IMUI source-policy guard.

## Remaining Work

This is display-only. Editable RGB/HSV numeric input modes are covered by
`docs/workstreams/imui-color-edit-numeric-input-v1/CLOSEOUT_AUDIT_2026-05-04.md`, and per-control
popup defaults are covered by
`docs/workstreams/imui-color-edit-popup-options-v1/CLOSEOUT_AUDIT_2026-05-05.md`. Start separate
follow-ons for color drag/drop payloads, color history, palette customization, or eyedropper
behavior.
