# ImUi Color Edit Hue Wheel Picker v1 TODO

Status: Closed.

- [x] Confirm Dear ImGui's `PickerHueWheel` interaction and rendering shape from `repo-ref/imgui`.
- [x] Add an explicit `ColorEditPopupPicker::HsvHueWheel` policy variant.
- [x] Add shared HueWheel geometry, ring hit testing, and SV triangle barycentric math.
- [x] Render the hue ring, rotated SV triangle, and Hue/SV cursors through `Canvas`.
- [x] Compose the existing vertical AlphaBar beside the HueWheel when alpha editing is visible.
- [x] Add focused tests and source-policy anchors.
- [x] Update roadmap, tracker, gap audit, and umbrella evidence.
- [x] Run focused gates and close the lane.

## Follow-On Candidates

- Picker options popup.
- Color history.
- Eyedropper behavior.
- Palette customization.
- Higher-fidelity triangle fill if a future renderer path supports per-vertex gradients directly.
