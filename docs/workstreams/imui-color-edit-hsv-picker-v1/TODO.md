# ImUi Color Edit HSV Picker v1 TODO

Status: Closed
Last updated: 2026-05-04

## M0 - Source Mapping

- [x] Confirm Dear ImGui keeps `ColorConvertRGBtoHSV` / `ColorConvertHSVtoRGB` helpers.
- [x] Confirm `ColorPicker4` owns HSV picker behavior above the lower-level color edit surface.
- [x] Keep this Fret slice in `fret-ui-editor`, not `fret-imui` or `crates/fret-ui`.

## M1 - Conversion Foundation

- [x] Add RGB-to-HSV and HSV-to-RGB helpers for the editor color surface.
- [x] Test primary colors, grayscale behavior, and preset roundtrips.
- [x] Preserve current alpha when HSV picker edits write a new RGB color.

## M2 - Popup Picker Controls

- [x] Add a saturation/value picker to the `ColorEdit` popup.
- [x] Add a HueBar to the `ColorEdit` popup.
- [x] Sync picker edits back to the model, draft hex text, error state, and redraw request path.
- [x] Keep existing preset swatches and AlphaBar behavior intact.

## M3 - Gates And Closeout

- [x] Add source-policy guards for the picker infrastructure.
- [x] Update IMUI gap audit and workstream indexes.
- [x] Leave follow-on policy for full `ColorPicker4` parity.
