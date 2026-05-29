# Material3 NavigationDrawer Item Motion Packet v2

Date: 2026-05-29
Task: M3PV2-084

## Truth

- Standalone Material NavigationDrawer destinations use Material state-layer / ripple indication
  for pressed interaction.
- The state layer is bounded to the destination active-indicator chrome, not the full drawer
  container.
- Idle drawer destinations do not paint a visible state layer.
- ModalNavigationDrawer panel slide and scrim fade remain covered by M3PV2-075; this packet closes
  the standard drawer item motion axis.

## Sources

- Compose Material3 `NavigationDrawer.kt`: `NavigationDrawerItem` renders through `Surface` with
  `selected`, `onClick`, `shape`, color resolution, and an optional `MutableInteractionSource`,
  while the row is constrained to the active-indicator height and full item width.
- Compose Material3 `NavigationDrawerItem`: destination semantics use `Role.Tab`, row content uses
  16dp/24dp horizontal padding, and icon/badge spacing remains 12dp.
- Fret Material tokens expose `md.comp.navigation-drawer.*.state-layer.color` and state opacity
  accessors for hover, focus, and pressed outcomes.

MUI Material UI is not available in this checkout's `repo-ref/`; local Compose Material3 was the
primary source for this motion proof.

## Layer Finding

This packet found a proof-density gap, not an implementation or infrastructure gap:

- `NavigationDrawer` already used `material_ink_layer_for_pressable(...)` with
  `RippleClip::Bounded`, NavigationDrawer state-layer tokens, and the active-indicator shape.
- Core already exposed pointer input, pressable state, scene quads, and repaint frames needed to
  observe the motion.
- The missing piece was a focused fixed-frame gate proving standard drawer destination pressed
  indication.

No `crates/*`, `fret-ui-kit`, token, or recipe implementation change was needed. The only source
change removes the stale module-level `MVP` label now that the current recipe has v2 proof coverage.

## Artifacts

- `ecosystem/fret-ui-material3/src/navigation_drawer.rs`
- `ecosystem/fret-ui-material3/tests/navigation_drawer_state.rs`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json`

## Wiring

- `navigation_drawer_state::navigation_drawer_pressed_state_layer_animates_over_item_chrome`
  renders a standalone NavigationDrawer, captures the inactive destination `.chrome` bounds,
  verifies idle state paints no visible state layer, dispatches pointer move/down at the destination
  center, advances fixed frames, and asserts a partial-alpha state-layer quad over the item chrome.

## Proof

Proof gate:

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test navigation_drawer_state navigation_drawer_pressed_state_layer_animates_over_item_chrome
```

The gate passed without implementation changes, proving the current NavigationDrawer recipe already
animates pressed item indication through the shared Material ink runtime.

## Residual Risk

- NavigationDrawer style remains `covered_v1`; this packet only closes standard destination item
  motion, not a full shape/elevation/color variant matrix.
- Standalone standard drawers are always-present surfaces, so they do not have open/close motion.
- Dismissible drawer gestures, predictive-back scaling, RTL slide direction, permanent drawer
  insets, drawer headers, and adaptive NavigationSuite ownership remain future API work.
