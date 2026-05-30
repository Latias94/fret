# Material 3 NavigationDrawer Selector Completion Packet v1

Date: 2026-05-28
Task: M3NDS-020
Component: NavigationDrawer

## Truth

- Standard `NavigationDrawer` exposes a root `.chrome` selector.
- Drawer item roots and `.chrome` selectors remain live.
- Drawer item `.icon`, `.label`, and optional `.badge` selectors are live when those parts exist.
- Modal drawer content inherits the same selectors from the `NavigationDrawer` recipe.

## Artifacts

- `ecosystem/fret-ui-material3/src/navigation_drawer.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`

## Wiring

Selectors are stamped in the recipe where the drawer chrome and item slots are assembled. No
`fret-ui-kit` roving/overlay policy changes are required.

## Proof

Focused automation-surface tests assert standard and modal drawer root chrome, item root/chrome, and
item icon/label/badge selectors.

## Residual Risk

Modal drawer motion/interruption and shared navigation foundation extraction remain separate
follow-ons only if future diagnostics prove drift.

