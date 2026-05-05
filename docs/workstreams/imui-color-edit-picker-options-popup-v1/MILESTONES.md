# ImUi Color Edit Picker Options Popup v1 Milestones

Status: Closed.

## M1 - Runtime Options State

Status: Done.

Exit criteria:

- `ColorEditPopupOptions` has an app-owned flag for showing the options surface.
- Popup-local runtime state preserves user choices across frames.
- Runtime state resets when app-owned defaults change.

## M2 - Popup Surface

Status: Done.

Exit criteria:

- The popup shows HueBar/HueWheel choices when the app default exposes a picker shape.
- The popup shows an AlphaBar toggle when alpha editing is visible.
- The controls update the existing picker composition without closing the popup or touching global
  state.

## M3 - Evidence and Closeout

Status: Done.

Exit criteria:

- Focused tests and source-policy anchors pass.
- Roadmap, tracker, gap audit, umbrella evidence, and catalog entries point at this lane.
- The lane has a closeout audit with exact gates.
