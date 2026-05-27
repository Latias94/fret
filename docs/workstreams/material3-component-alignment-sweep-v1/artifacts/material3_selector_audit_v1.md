# Material 3 Selector Audit v1

Status: active audit note
Date: 2026-05-27

## Packet-ready selectors

- Tabs: `tabs`, `tabs.chrome`, `tabs.active-indicator`, `tabs.item`, `tabs.item.chrome`
- NavigationBar: `navigation_bar`, `navigation_bar.chrome`, `navigation_bar.active-indicator`, `navigation_bar.item`, `navigation_bar.item.chrome`, `navigation_bar.item.icon`, `navigation_bar.item.label`, `navigation_bar.item.badge`
- NavigationRail: `navigation_rail`, `navigation_rail.chrome`, `navigation_rail.active-indicator`, `navigation_rail.item`, `navigation_rail.item.chrome`, `navigation_rail.item.icon`, `navigation_rail.item.label`, `navigation_rail.item.badge`

## Source notes

- Tabs, NavigationBar, and NavigationRail now stamp stable dotted part ids from the recipe layer.
- The automation-surface gate confirms the parts are live in rendered trees.
- NavigationBar and NavigationRail still use item-level `icon`, `label`, and `badge` selectors for future packet scripts; those are intentionally recipe-owned, not gallery-owned.

## Remaining selector gaps

- TextField, Autocomplete, ExposedDropdown, Menu, DropdownMenu, Dialog, Checkbox, Radio, and Slider still need packet-specific selector audits before their first diagnostic scripts should depend on them.
