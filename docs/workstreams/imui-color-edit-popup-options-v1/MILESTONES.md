# ImUi Color Edit Popup Options v1 Milestones

Status: Closed
Last updated: 2026-05-05

## M0 - Reference And Scope

- Confirmed Dear ImGui's relevant option/default reference points:
  - `ImGuiColorEditFlags_DefaultOptions_`
  - `ColorEditOptionsPopup()`
  - `ColorPickerOptionsPopup()`
  - `SetColorEditOptions()`
- Chose a Fret-native per-control option surface instead of global mutable defaults.

## M1 - Implementation

- Added `ColorEditPopupOptions`.
- Added `ColorEditPopupPicker`.
- Added `ColorEditPopupNumericInputs`.
- Added `ColorEditOptions::popup`.
- Made popup content assembly respect the configured picker, numeric rows, presets, and AlphaBar.
- Disabled swatch activation/focus when the configured popup has no visible content.

## M2 - Proof

- Focused `color_edit` tests cover default options, hidden popup behavior, and numeric row mode
  ordering.
- Source-policy tests guard against returning to a stub popup and require the new option surface.
- Adapter smoke tests prove the option types compile through the public editor controls surface.
