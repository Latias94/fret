# ImUi Color Edit Picker Options Popup v1

Status: Closed narrow P1 feature follow-on
Last updated: 2026-05-05

Dear ImGui's `ColorPickerOptionsPopup()` lets users choose between `PickerHueBar` and
`PickerHueWheel`, and can expose an AlphaBar toggle. Fret already has explicit per-control popup
defaults and both picker shapes; this lane adds the missing user-facing options surface while
keeping the state local to the editor control.

## Ownership

- `color_edit.rs` owns public popup policy, local runtime option state, and default reconciliation.
- `popup.rs` resolves runtime options and inserts the options surface into the popup contents.
- `popup/options.rs` owns the compact picker option controls.
- `color_edit/tests.rs` owns pure state/default regression tests.
- `imui_surface_policy.rs` keeps the source ownership auditable.
- `fret-imui` remains a thin adapter.

## Must-Be-True Outcomes

- The default editor `ColorEdit` popup exposes a discoverable options surface for picker shape and,
  when alpha editing is visible, AlphaBar visibility.
- Switching picker shape is popup-local and per control; it never writes a Dear ImGui-style global
  `ColorEditOptions` bitmask.
- App-owned `ColorEditPopupOptions` still define the defaults and can disable the options surface.
- Runtime picker/AlphaBar choices survive ordinary re-renders but reset when app-owned defaults
  change.
- Hidden picker policy remains respected: a control that hides the picker does not let the options
  surface re-enable HueBar/HueWheel unless the app default exposes a picker shape.

## Non-Goals

- No global `SetColorEditOptions()` compatibility layer.
- No separate right-click context-menu primitive.
- No color history.
- No eyedropper behavior.
- No palette customization.
- No renderer contract changes.
