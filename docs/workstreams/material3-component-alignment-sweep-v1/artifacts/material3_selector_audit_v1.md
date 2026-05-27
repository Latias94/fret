# Material 3 Selector Audit v1

Status: active audit note
Date: 2026-05-27

## Packet-ready selectors

- Tabs: `tabs`, `tabs.chrome`, `tabs.active-indicator`, `tabs.item`, `tabs.item.chrome`
- NavigationBar: `navigation_bar`, `navigation_bar.chrome`, `navigation_bar.active-indicator`, `navigation_bar.item`, `navigation_bar.item.chrome`, `navigation_bar.item.icon`, `navigation_bar.item.label`, `navigation_bar.item.badge`
- NavigationRail: `navigation_rail`, `navigation_rail.chrome`, `navigation_rail.active-indicator`, `navigation_rail.item`, `navigation_rail.item.chrome`, `navigation_rail.item.icon`, `navigation_rail.item.label`, `navigation_rail.item.badge`
- NavigationDrawer seed: `navigation_drawer`, `navigation_drawer.item`, `navigation_drawer.item.chrome`
- ModalNavigationDrawer seed: `modal_navigation_drawer`, `modal_navigation_drawer.scrim`, `modal_navigation_drawer.scrim.chrome`, `modal_navigation_drawer.panel`

## Source notes

- Tabs, NavigationBar, and NavigationRail now stamp stable dotted part ids from the recipe layer.
- The automation-surface gate confirms the parts are live in rendered trees.
- NavigationBar and NavigationRail still use item-level `icon`, `label`, and `badge` selectors for future packet scripts; those are intentionally recipe-owned, not gallery-owned.
- M3CAS-100 consolidated repeated `.chrome` part-id construction into `foundation::test_id` and
  added a NavigationDrawer automation-surface selector gate.
- M3CAS-120 replaced ModalNavigationDrawer's legacy hyphenated scrim id with dotted part ids and
  added a root/scrim/panel automation-surface selector gate.

## Remaining selector gaps

- Choice controls and chips are covered by `material3_choice_controls_packet_v1.md`.
- Surface/data-display components are covered by `material3_surface_data_display_packet_v1.md`.
- Badge, Button, Card, CarouselItem, Divider, FAB, List, ProgressIndicator, and TopAppBar have live
  root selectors in `automation_surface`; Button/Card/CarouselItem/FAB/List/TopAppBar actions also
  expose recipe-owned `.chrome` selectors where their chrome is an inspectable element.
- Slider internal canvas paint parts (`track`, `handle`, `state-layer`) still need a diagnostics
  mechanism for named canvas draw regions before scripts should depend on part selectors for those
  paint ops.
- ProgressIndicator internal canvas paint parts (`track`, active segment, circular arc segments)
  need the same named draw-region diagnostics before scripts should depend on part selectors for
  those paint ops.
