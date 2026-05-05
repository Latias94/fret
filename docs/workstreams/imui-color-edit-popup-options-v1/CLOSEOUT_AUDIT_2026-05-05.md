# ImUi Color Edit Popup Options v1 Closeout Audit - 2026-05-05

Status: Closed.

This lane closes a bounded `ColorEdit4` option/default gap by making the editor `ColorEdit` popup
surface configurable per control.

## What Shipped

- Added `ColorEditPopupOptions`.
- Added `ColorEditPopupPicker` with the current HueBar picker as the default picker surface.
- Added `ColorEditPopupNumericInputs` for RGB+HSV, RGB-only, HSV-only, and hidden numeric rows.
- Added `ColorEditOptions::popup`.
- Re-exported the new option types from `fret_ui_editor::controls`.
- Disabled swatch activation/focus when the popup configuration has no visible content.
- Added focused unit, source-policy, and adapter smoke coverage.

## Proof

- `cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast` passes.
- `cargo nextest run -p fret-ui-editor --features imui --test imui_surface_policy --no-fail-fast`
  passes.
- `cargo nextest run -p fret-ui-editor --features imui --test imui_adapter_smoke --no-fail-fast`
  passes.

## Remaining Work

Full Dear ImGui color-edit depth is still not closed. Start separate follow-ons for color history,
palette customization, eyedropper integration, color drag/drop payloads, or visual HueWheel
fidelity.
