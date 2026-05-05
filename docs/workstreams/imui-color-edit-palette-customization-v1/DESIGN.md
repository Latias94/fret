# ImUi Color Edit Palette Customization v1

Status: Closed narrow P1 feature follow-on
Last updated: 2026-05-05

Dear ImGui's color demo shows a custom picker popup with a persistent app-owned palette. Fret's
editor `ColorEdit` currently renders a hard-coded preset row. This lane keeps the useful default
palette but lets app authors replace it with their own palette entries.

## Ownership

- `color_edit.rs` owns the public palette entry type and default palette source.
- `popup.rs` passes the app-owned palette into popup composition.
- `popup/swatches.rs` renders the provided palette entries and keeps activation alpha-preserving.
- `color_edit/tests.rs` owns default/custom palette source tests.
- `imui_adapter_smoke.rs` keeps the public editor adapter surface compiling with custom palettes.
- `fret-imui` remains a thin adapter.

## Must-Be-True Outcomes

- Default `ColorEditOptions` still render the current 12-entry palette.
- App authors can pass a custom palette without changing popup mechanics.
- Empty custom palettes are allowed and simply render no preset swatches.
- Selecting a palette entry preserves the current alpha channel, matching Dear ImGui demo palette
  selection behavior.
- Palette data stays app-owned; this lane does not introduce global color history or global palette
  mutation state.

## Non-Goals

- No user-editable palette slots inside the popup.
- No drag/drop-to-palette-slot mutation.
- No color history.
- No eyedropper behavior.
- No new runtime or renderer contract.
