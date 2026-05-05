# ImUi Color Edit Picker Options Thumbnail Preview v1

Status: Closed narrow P1 polish follow-on
Last updated: 2026-05-05

Dear ImGui's `ColorPickerOptionsPopup()` draws small preview versions of both picker types before
the user selects `PickerHueBar` or `PickerHueWheel`. Fret already had a popup-local picker options
surface; this lane upgrades that surface from text-only buttons to thumbnail radio cards using the
existing editor picker preview renderers.

## Ownership

- `popup/options.rs` owns the picker options surface and thumbnail card composition.
- `popup/picker.rs` exposes existing HueBar, SV, and HueWheel preview helpers to sibling popup
  modules; no new renderer contract is introduced.
- `ColorEditPopupOptions::picker_options` remains the app-owned switch for showing the surface.
- `crates/fret-ui`, `fret-imui`, and global `SetColorEditOptions()` state are not widened.

## Must-Be-True Outcomes

- The picker options surface previews both `HsvHueBar` and `HsvHueWheel` choices visually.
- The thumbnails reuse the same preview primitives as the interactive picker, avoiding a second
  hand-rolled color-rendering path.
- Selection behavior and per-control runtime option reconciliation remain unchanged.
- The AlphaBar toggle remains a compact checkbox-style row below the picker cards when visible.

## Non-Goals

- No eyedropper behavior.
- No global options popup state.
- No right-click picker options context menu.
- No renderer or platform contract changes.
