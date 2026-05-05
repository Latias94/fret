# ImUi Color Edit Hue Wheel Picker v1

Status: Closed narrow P1 feature follow-on
Last updated: 2026-05-05

Dear ImGui exposes two picker shapes for `ColorPicker4`: the default `PickerHueBar` and the
alternative `PickerHueWheel`. The HueWheel path uses a hue ring and a rotated saturation/value
triangle, with the optional AlphaBar still sitting beside the picker. This lane adds the missing
HueWheel shape to editor `ColorEdit` as explicit per-control popup policy.

## Ownership

- `color_edit.rs` owns the public `ColorEditPopupPicker::HsvHueWheel` policy variant.
- `model.rs` owns HueWheel geometry, hit-zone classification, hue angle mapping, and SV triangle
  barycentric conversion.
- `popup.rs` selects the HueWheel picker variant.
- `popup/picker.rs` owns Canvas rendering, picker-local pointer capture, and AlphaBar composition.
- `color_edit/tests.rs` owns the focused coordinate regression tests.
- `imui_surface_policy.rs` keeps the picker owner auditable.
- `fret-imui` remains a thin adapter.

## Must-Be-True Outcomes

- Apps can opt into `ColorEditPopupPicker::HsvHueWheel` without changing the default
  `HsvHueBar` policy.
- Hue ring interaction maps screen angle like Dear ImGui: right = 0, down = 0.25, left = 0.5,
  up = 0.75.
- SV triangle interaction follows Dear ImGui's rotated triangle and barycentric mapping:
  hue vertex = saturated bright color, black vertex = near-zero value, white vertex = near-zero
  saturation at full value.
- The HueWheel picker composes with the existing vertical AlphaBar when alpha editing is visible.
- The behavior stays in `fret-ui-editor`, not `crates/fret-ui` or `fret-imui`.

## Non-Goals

- No picker options popup or global `SetColorEditOptions()` state.
- No color history.
- No eyedropper behavior.
- No palette customization.
- No attempt to make the HueWheel the default picker.
