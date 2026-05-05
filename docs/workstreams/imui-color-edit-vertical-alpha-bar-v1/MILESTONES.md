# ImUi Color Edit Vertical Alpha Bar v1 Milestones

Status: Closed.

## M0 - Upstream Behavior

- Dear ImGui `AlphaBar` is vertical in `ColorPicker4`.
- Alpha interaction maps local Y so top is full alpha and bottom is transparent.

## M1 - Fret Layout

- `hsv_picker` accepts whether alpha should be inlined.
- `popup.rs` shows a standalone AlphaBar only when the picker surface is hidden.
- `vertical_alpha_bar` shares the picker-side height with the SV and HueBar area.

## M2 - Interaction

- Pointer down/move on vertical AlphaBar maps local Y to inverted alpha.
- Existing horizontal `alpha_bar` remains available for picker-hidden popup combinations.

## M3 - Closeout

- Focused tests and source-policy anchors pass.
- Workstream, roadmap, tracker, gap audit, and umbrella evidence are updated.
- Gates pass.
