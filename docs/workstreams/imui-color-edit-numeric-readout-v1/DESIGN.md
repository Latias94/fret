# ImUi Color Edit Numeric Readout v1

Status: Closed narrow product follow-on
Last updated: 2026-05-04

This lane adds a visible numeric readout to the editor-owned `ColorEdit` popup. It follows the HSV
picker slice by making the current RGB and HSV values inspectable while keeping exact editing on
the existing hex text input.

## Ownership

- `fret-ui-editor` owns the editor `ColorEdit` control, readout layout, and formatting helpers.
- `fret-imui` remains a thin immediate-mode mounting layer.
- `crates/fret-ui` runtime contracts are not in scope.

## Must-Be-True Outcomes

- The `ColorEdit` popup renders RGB and HSV numeric readout lines after the HSV picker and before
  preset swatches.
- The RGB readout includes alpha percent when `show_alpha=true`.
- The readout uses editor muted foreground tokens and stable popup-derived test ids.
- Source-policy tests keep the readout in the editor-owned implementation.

## Non-Goals

- No editable RGB/HSV numeric input fields yet.
- No Dear ImGui-style global `SetColorEditOptions()`.
- No option popup defaults.
- No color drag/drop payloads, history, palette customization, or eyedropper behavior.
