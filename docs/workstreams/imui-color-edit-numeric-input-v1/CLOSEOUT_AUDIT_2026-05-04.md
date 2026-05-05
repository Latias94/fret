# ImUi Color Edit Numeric Input v1 Closeout Audit - 2026-05-04

Status: Closed.

This lane closes a bounded `ColorEdit4` input gap by making editor `ColorEdit` popup RGB/HSV numeric
rows editable.

## What Shipped

- Added editor-owned `color_numeric_inputs` and `color_numeric_input_field` helpers.
- Added RGB parsing for 0-255 channels with optional alpha percent when alpha is visible.
- Added HSV parsing for hue degrees and saturation/value percentages.
- Preserved current alpha for RGB rows without alpha and for HSV commits.
- Added popup-local numeric error state so invalid numeric input is not cleared by the main hex
  field sync path.
- Added focused parsing tests and source-policy guards.

## Proof

- `cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast` passes focused
  color edit tests, including numeric input parsing and rejection tests.
- `cargo nextest run -p fret-ui-editor --features imui --test imui_surface_policy --no-fail-fast`
  passes the editor IMUI source-policy guard.

## Remaining Work

Full Dear ImGui color-edit depth is still not closed. Per-control popup defaults are covered by
`docs/workstreams/imui-color-edit-popup-options-v1/CLOSEOUT_AUDIT_2026-05-05.md`. Start separate
follow-ons for color history, palette customization, eyedropper behavior, color drag/drop payloads,
or HueWheel fidelity.
