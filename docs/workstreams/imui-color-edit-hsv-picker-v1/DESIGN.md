# ImUi Color Edit HSV Picker v1

Status: Closed
Last updated: 2026-05-04

## Problem

Dear ImGui's `ColorEdit4` / `ColorPicker4` surface goes beyond hex entry and presets. The local
reference keeps RGB/HSV conversion helpers, an HSV picker path, hue selection, saturation/value
selection, alpha handling, and optional display/input modes in one editor-grade color workflow.

Fret's editor `ColorEdit` already shipped hex input, alpha-preserving RGB policy, alpha previews,
preset swatches, and a bounded AlphaBar. The remaining local gap was that opening the swatch still
did not provide a direct HSV picker affordance.

## Target

- Add editor-owned RGB/HSV conversion helpers and focused tests.
- Add a bounded saturation/value picking area to the `ColorEdit` popup.
- Add a bounded HueBar to the same popup.
- Preserve the current alpha channel when HSV edits commit a new RGB color.
- Keep exact `#RRGGBB` / `#RRGGBBAA` text input as the precise edit path.
- Keep `fret-imui` as a thin adapter over the editor control.

## Ownership

- `ecosystem/fret-ui-editor/src/controls/color_edit.rs`: picker UI, conversion helpers, and tests.
- `ecosystem/fret-ui-editor/tests/imui_surface_policy.rs`: source-policy guard that the editor
  control owns the picker depth.
- `apps/fret-cookbook/examples/imui_editor_controls_basics.rs`: public app-facing proof through
  `fret::imui::editor`.

## Non-Goals

- Dear ImGui-style global `SetColorEditOptions()`.
- RGB/HSV numeric display-mode toggles.
- Hue wheel picker parity.
- Color drag/drop payloads.
- Eyedropper integration.
- Moving picker policy into `fret-imui` or `crates/fret-ui`.
