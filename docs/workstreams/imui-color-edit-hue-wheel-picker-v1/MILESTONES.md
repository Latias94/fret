# ImUi Color Edit Hue Wheel Picker v1 Milestones

Status: Closed.

## M0 - Reference Shape

- [x] Read `repo-ref/imgui/imgui_widgets.cpp` for `PickerHueWheel` hit testing and rendering.
- [x] Record non-goals so options/history/eyedropper work does not widen this slice.

## M1 - Math and Policy

- [x] Add `ColorEditPopupPicker::HsvHueWheel`.
- [x] Add pure HueWheel geometry and SV triangle mapping helpers.
- [x] Lock ring angle, rotated triangle, and outside-hit behavior with unit tests.

## M2 - UI Composition

- [x] Add the Canvas-backed HueWheel picker.
- [x] Wire popup selection and optional vertical AlphaBar composition.
- [x] Keep the immediate facade thin and unchanged.

## M3 - Evidence

- [x] Update source-policy tests and workstream docs.
- [x] Run focused and editor IMUI gates.
