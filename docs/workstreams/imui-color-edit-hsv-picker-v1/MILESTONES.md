# ImUi Color Edit HSV Picker v1 Milestones

Status: Closed
Last updated: 2026-05-04

## M0 - Source Mapping

Exit criteria:

- Dear ImGui color picker reference points are identified.
- Fret ownership is explicit and editor-layer-only.

State: Complete.

## M1 - HSV Conversion Foundation

Exit criteria:

- RGB/HSV conversion helpers exist in the editor control implementation.
- Unit tests lock primary colors, grayscale behavior, preset roundtrips, picker coordinate mapping,
  and alpha-preserving HSV writes.

State: Complete.

## M2 - Bounded Popup Picker

Exit criteria:

- Opening editor `ColorEdit` exposes a saturation/value area and HueBar before preset swatches.
- Pointer down / drag on picker controls updates the color model, draft hex, error state, and redraw.
- Existing alpha preview, preset palette, and AlphaBar paths remain in place.

State: Complete.

## M3 - Closeout

Exit criteria:

- Focused tests and source-policy guards cover the new infrastructure.
- Workstream docs explain why full Dear ImGui picker parity remains separate.

State: Complete.
