# ImUi Color Edit Picker Options Thumbnail Preview v1 Milestones

Status: Closed
Last updated: 2026-05-05

## M1 - Thumbnail Composition

- HueBar options show an SV preview plus a HueBar strip.
- HueWheel options show the HueWheel canvas preview.
- Thumbnail helpers reuse picker preview code rather than duplicating color math.

## M2 - Existing Semantics Preserved

- Picker option selection still updates popup-local runtime state.
- `ColorEditPopupOptions::picker_options` still controls visibility.
- AlphaBar remains a checkbox row and is not converted into a thumbnail card.

## M3 - Evidence and Closeout

- Focused color-edit tests continue passing.
- Source-policy tests anchor the new thumbnail composition functions.
- Workstream docs name remaining eyedropper and side-preview polish gaps.
