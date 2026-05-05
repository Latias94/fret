# ImUi Color Edit Vertical Hue Bar v1 Milestones

Status: Closed.

## M0 - Upstream Behavior

- Dear ImGui `PickerHueBar` renders the SV rectangle and a vertical HueBar side by side.
- Hue interaction uses local Y over the SV-picker height.

## M1 - Fret Layout

- `hsv_picker` lays out the SV picker and HueBar horizontally.
- `hue_bar` uses fixed vertical dimensions aligned with the SV area.
- The hue gradient uses rows instead of columns.

## M2 - Interaction

- Pointer down/move on the HueBar maps local Y to hue.
- Existing SV editing and alpha-preserving HSV conversion continue to pass focused tests.

## M3 - Closeout

- Focused tests and source-policy anchors pass.
- Workstream, roadmap, tracker, gap audit, and umbrella evidence are updated.
- Gates pass.
