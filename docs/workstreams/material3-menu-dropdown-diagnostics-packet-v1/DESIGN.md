# Material 3 Menu And Dropdown Diagnostics Packet v1 - Design

Status: Closed
Last updated: 2026-05-28

## Problem

The component matrix still left `Menu` and `DropdownMenu` in known-follow-on state. The recipe code
already exposed stable Material menu surface/item selectors and Rust gates covered pressed scene
stability, style overrides, and dismiss/focus-restore policy. The missing evidence was a dedicated
Material3 gallery diagnostics packet that proved the page-level DropdownMenu focus/dismiss path and
kept it separate from item chrome diagnostics.

## Truth

- `Menu` recipe owns Material surface chrome, item chrome, item roles, labels, disabled state, and
  recipe-local roving/typeahead until another design system proves a shared kit abstraction.
- `DropdownMenu` owns Material recipe composition and forwards menu behavior into the shared
  dismissible menu overlay policy.
- `fret-ui-kit` owns Escape dismissal, outside-press non-click-through behavior, overlay unmount,
  and focus restore.
- Diagnostics should prove both visual chrome and runtime focus/dismiss wiring on the dedicated
  Material3 Menu gallery page.

## Boundaries

- Do not move DropdownMenu dismiss/focus policy into the Material recipe.
- Do not move recipe-local Menu roving/typeahead into kit policy without another consumer.
- Do not change `crates/*` unless a concrete mechanism gap appears.
