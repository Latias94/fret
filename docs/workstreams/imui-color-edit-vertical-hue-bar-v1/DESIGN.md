# ImUi Color Edit Vertical Hue Bar v1

Status: Closed narrow P1 feature follow-on
Last updated: 2026-05-05

Dear ImGui's default `PickerHueBar` shape is an SV square with a vertical hue bar on the right.
Fret's editor `ColorEdit` already had HSV conversion and a HueBar picker, but the first version used
a horizontal hue strip below the SV area. This lane fixes that shape mismatch without widening the
runtime, adding global color edit state, or changing the `fret-imui` facade.

## Ownership

- `color_edit/model.rs` owns pure local-coordinate to hue conversion.
- `popup/picker.rs` owns the HSV picker layout and vertical hue bar rendering.
- `color_edit/tests.rs` owns the focused coordinate mapping regression.
- `imui_surface_policy.rs` keeps the picker owner auditable.
- `fret-imui` remains a thin adapter.

## Must-Be-True Outcomes

- `ColorEditPopupPicker::HsvHueBar` presents an SV square next to a vertical HueBar.
- Hue pointer interaction maps local Y to hue, matching Dear ImGui's `PickerHueBar` interaction
  axis.
- The SV square continues to use local X/Y for saturation/value.
- AlphaBar remains a separate follow-on; this lane does not change alpha interaction shape.
- The behavior stays in `fret-ui-editor`, not `crates/fret-ui` or `fret-imui`.

## Non-Goals

- No HueWheel picker.
- No vertical AlphaBar parity.
- No picker options popup.
- No context menu or global `SetColorEditOptions()` state.
