# Material3 Select Selected Item Style/Layout Packet v2

Date: 2026-05-28
Task: M3PV2-026

## Truth

Material selectable menu items have two independent outcomes:

- selected item content uses selected label and leading/trailing icon colors;
- selectable item chrome is inset from the listbox edge, while the row keeps the full hit target.

## Sources

- Compose Material3 `Menu.kt`: selectable `DropdownMenuItem` wraps content in a selected `Surface`
  with `DropdownMenuSelectableItemPadding`.
- Compose Material3 `MenuDefaults.kt`: selectable item colors expose selected text, leading icon,
  trailing icon, and container colors.
- Compose Material3 `tokens/MenuTokens.kt` and `tokens/StandardMenuTokens.kt`: selected menu/list
  content colors differ from normal item colors.
- Fret generated Material Web v30 tokens: Select owns the selected container color, while selected
  content colors are supplied by list/menu selected tokens.

## Findings

This was a Material recipe gap, not a core or kit mechanism issue.

- `Select` already had popup width and behavior gates, but selected menu rows painted only the
  selected container background.
- Selected label, leading icon, and trailing icon still used normal item colors.
- The selected background filled the listbox width instead of using the Material selectable item
  horizontal inset.
- `Fill + margin` was not the right expression for this nested retained layout path; the popup
  placement already owns the final listbox width, so the item recipe now receives that width and
  computes the inset chrome width explicitly.

## Implementation

- Added Select token helpers for selectable item outer padding, content padding, icon/text gap, and
  selected/unselected item shape.
- Made Select selected/disabled item content colors delegate to list selected outcomes.
- Passed the resolved listbox width from popup placement into listbox items and sized the visible
  item chrome as `listbox_width - 2 * selectable_inset`.
- Kept the pressable row full-height/full-width so pointer and roving-focus behavior remain owned by
  the existing Select recipe.

## Proof

- Red before fix:
  - `cargo nextest run -p fret-ui-material3 --lib select_menu_selected_item_uses_selected_content_colors`
    failed with selected label color still using the normal menu item color.
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_select_exposes_stable_part_test_ids`
    failed because selected item chrome was not inset.
- Green after fix:
  - `cargo fmt --package fret-ui-material3`
  - `cargo nextest run -p fret-ui-material3 --lib select_menu_selected_item_uses_selected_content_colors`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_select_exposes_stable_part_test_ids`
  - `cargo nextest run -p fret-ui-material3 --test select_behavior`
  - `cargo nextest run -p fret-ui-material3 --lib select::item_text_tests`

## Residual Risk

Select motion remains seeded. This packet covers settled selected item style/layout, not fixed-timestep
open/close or ripple timing.
