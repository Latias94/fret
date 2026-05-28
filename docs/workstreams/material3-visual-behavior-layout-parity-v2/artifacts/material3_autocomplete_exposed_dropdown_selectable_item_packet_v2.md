# Material3 Autocomplete/ExposedDropdown Selectable Item Packet v2

Date: 2026-05-28
Task: M3PV2-027

## Truth

Autocomplete and ExposedDropdown reuse the Material selectable menu item surface:

- listbox/menu content has vertical padding, not broad horizontal padding;
- each option chrome owns the selectable item inset;
- selected option label color switches to selected content color;
- ExposedDropdown inherits the same listbox option rendering through Autocomplete composition.

## Sources

- Compose Material3 `Menu.kt`: selectable `DropdownMenuItem` applies selectable item padding around
  the item surface.
- Compose Material3 `MenuDefaults.kt`: selected item text/icon/container colors are selected-state
  outcomes, not normal item colors.
- Fret Select M3PV2-026: same Material selectable item rule, now used as a local regression exemplar.

## Findings

This was a shared Material recipe issue.

- Autocomplete listbox placed `8px` horizontal padding on the list container, so option chrome was
  inset by `8px` instead of Material's `4px` selectable item inset.
- The option label color was computed once at panel level, so selected options could not use
  selected content colors.
- Select had already needed the same rule, so the duplicated constants were extracted into
  `tokens::selectable_menu_item`.

## Implementation

- Added a shared `selectable_menu_item` token helper for selectable item inset, content padding,
  shape, and selected/disabled content colors.
- Refactored Select token helpers to delegate to the shared helper.
- Updated Autocomplete listbox rendering to use vertical listbox padding plus per-option inset
  chrome sized from the resolved popup width.
- Made selected Autocomplete option labels use selected list content outcomes.
- Added diagnostics assertions for Autocomplete and ExposedDropdown option chrome inset.

## Proof

- Red before fix:
  - `cargo nextest run -p fret-ui-material3 --lib autocomplete_selected_item_uses_selected_label_color`
    failed with selected option label still using normal item color.
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_autocomplete_exposes_stable_part_test_ids material3_exposed_dropdown_popup_matches_field_chrome_bounds`
    failed because option chrome was inset by the container's `8px` padding instead of `4px`.
- Green after fix:
  - `cargo fmt --package fret-ui-material3`
  - `cargo nextest run -p fret-ui-material3 --lib autocomplete_selected_item_uses_selected_label_color select_menu_selected_item_uses_selected_content_colors`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_autocomplete_exposes_stable_part_test_ids material3_exposed_dropdown_popup_matches_field_chrome_bounds material3_select_exposes_stable_part_test_ids`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_autocomplete_semantics_v1 material3_exposed_dropdown_trailing_icon_toggles_overlay_v1 material3_exposed_dropdown_reverts_query_to_committed_selection_on_blur_v1`
  - `cargo nextest run -p fret-ui-material3 --test select_behavior`
  - `cargo nextest run -p fret-ui-material3 --lib autocomplete::tests select::item_text_tests`

## Residual Risk

Motion remains seeded. This packet covers settled option chrome and selected label colors, not
fixed-timestep popup enter/exit or ripple timing.
