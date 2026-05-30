# Material 3 NavigationDrawer Selector Completion Packet v1

Date: 2026-05-28
Status: Closed

## Problem

The Material 3 matrix still marks `navigation_drawer` as a queued selector follow-on. The previous
drawer overlay packet closed selected-pill geometry, modal overlay focus, and the gallery diagnostic,
but the standard `NavigationDrawer` selector surface remains thinner than NavigationBar/Rail:

- the drawer root has no `.chrome` selector,
- drawer items expose root and `.chrome`, but not `.icon`, `.label`, or `.badge`.

That makes drawer diagnostics use broader item roots when they need to assert specific Material item
parts.

## Target State

- `navigation_drawer.chrome` identifies the drawer container chrome.
- `navigation_drawer.item.icon`, `.label`, and `.badge` are available when an item has the
  corresponding part.
- Modal drawer content inherits the same `NavigationDrawer` selector surface because it composes the
  same recipe.
- Existing item root and `.chrome` ids remain unchanged.
- The matrix is updated from queued selector follow-on to selector-completed known follow-ons.

## Source Truth

- Material NavigationDrawer items are composed from icon, label, optional badge, and selected pill
  chrome.
- Compose Material3 exposes NavigationDrawer item slots (`icon`, `label`, optional `badge`) as
  first-class composition inputs.
- Fret NavigationBar and NavigationRail already expose item `.icon`, `.label`, and `.badge`
  selectors, so Drawer should use the same diagnostics vocabulary.

## Layer Ownership

This is recipe/diagnostics work in `ecosystem/fret-ui-material3`:

- `fret-ui` already supports semantic and element test ids.
- `fret-ui-kit` owns roving focus and overlay/focus policy, which this packet does not change.
- Material recipe owns item-part taxonomy and selector naming.

## Non-Goals

- Change roving focus behavior.
- Change modal drawer overlay motion, dismissal, or focus restoration.
- Refresh navigation goldens.
- Introduce a shared navigation foundation abstraction.
