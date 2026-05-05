# ImUi Color Edit Hue Wheel Picker v1 Closeout Audit - 2026-05-05

Status: Closed.

## Shipped Surface

- Added `ColorEditPopupPicker::HsvHueWheel` as an opt-in popup picker policy while keeping
  `HsvHueBar` as the default.
- Added shared HueWheel geometry and interaction math for ring hit testing, screen-angle hue
  mapping, rotated SV triangle hit testing, closest-point clamping, and Dear ImGui-style
  barycentric SV conversion.
- Added a Canvas-backed HueWheel picker with a sweep-gradient hue ring, tessellated SV triangle,
  Hue/SV cursors, and optional existing vertical AlphaBar composition.
- Added focused tests and source-policy anchors for the new picker variant.

## Evidence

- `ecosystem/fret-ui-editor/src/controls/color_edit.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/model.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/popup.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/popup/picker.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/tests.rs`
- `ecosystem/fret-ui-editor/tests/imui_surface_policy.rs`
- `repo-ref/imgui/imgui_widgets.cpp`
- `repo-ref/imgui/imgui.h`

## Gates

- `cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast`

## Remaining ColorEdit Follow-Ons

- Picker options popup.
- Color history.
- Eyedropper behavior.
- Palette customization.
- Full visual parity polish for ColorPicker4 remains broader than this narrow HueWheel slice.
