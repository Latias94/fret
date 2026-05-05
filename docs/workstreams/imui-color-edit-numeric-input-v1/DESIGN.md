# ImUi Color Edit Numeric Input v1

Status: Closed narrow product follow-on
Last updated: 2026-05-04

This lane turns the editor-owned `ColorEdit` popup numeric rows from readout-only text into editable
RGB/HSV text inputs. It follows the readout slice and keeps the existing exact hex field as the
main compact edit path.

## Ownership

- `fret-ui-editor` owns the editor `ColorEdit` control, RGB/HSV parsing, field state, and popup
  error display.
- `fret-imui` remains a thin immediate-mode mounting layer.
- `crates/fret-ui` runtime contracts are not in scope.

## Must-Be-True Outcomes

- The `ColorEdit` popup renders RGB and HSV numeric rows as editable text inputs.
- RGB commits accept 0-255 channel values and preserve alpha when no alpha percent is supplied.
- When alpha is visible, the RGB row also accepts an alpha percent.
- HSV commits accept hue degrees plus saturation/value percentages and preserve alpha.
- Invalid numeric input is rejected with an editor-owned popup error instead of silently clamping or
  widening runtime behavior.
- Source-policy tests keep the numeric editing behavior in the editor-owned implementation.

## Non-Goals

- No Dear ImGui-style global `SetColorEditOptions()`.
- No option-popup defaults.
- No color history, palette customization, eyedropper behavior, or drag/drop color payloads.
- No runtime text-input callback API changes.
