# Material3 Menu Item Motion Packet v2

Date: 2026-05-29
Task: M3PV2-083

## Truth

- Standalone Material Menu items use Material state-layer / ripple indication for pressed
  interaction.
- The state layer is bounded to the item chrome, not the whole menu container.
- Disabled-but-focusable menu items remain non-invokable and do not participate in pressed
  indication.
- DropdownMenu overlay fade-scale motion remains covered by the earlier Menu/DropdownMenu packet;
  this packet closes the standalone Menu item motion axis.

## Sources

- Compose Material3 `Menu.kt`: plain `DropdownMenuItemContent` applies `clickable(...,
  indication = ripple(true))` to the item row.
- Compose Material3 `Menu.kt`: selectable dropdown menu items route item interaction through a
  `Surface` with a mutable interaction source and animated selected container color.
- Material Web v30 token exports in Fret define `md.comp.menu.list-item.*.state-layer.color` and
  state opacity tokens for hover, focus, and pressed outcomes.

MUI Material UI is not available in this checkout's `repo-ref/`; local Compose Material3 and
generated Material Web token snapshots were sufficient for this motion proof.

## Layer Finding

This packet found a proof-density gap, not an implementation or infrastructure gap:

- `Menu` already used `material_ink_layer_for_pressable(...)` with `RippleClip::Bounded`,
  `menu_tokens::item_outcomes(...)`, and `menu_tokens::pressed_state_layer_opacity(...)`.
- Core already exposed pointer input, pressable state, scene quads, and repaint frames needed to
  observe the motion.
- The missing piece was a focused fixed-frame gate proving standalone Menu item pressed indication.

No `crates/*`, `fret-ui-kit`, token, or recipe implementation change was needed.

## Artifacts

- `ecosystem/fret-ui-material3/tests/menu_state.rs`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json`

## Wiring

- `menu_state::menu_pressed_state_layer_animates_over_item_chrome` renders a standalone Menu,
  captures the item `.chrome` bounds, verifies idle state paints no visible state layer, dispatches
  pointer move/down at the item center, advances fixed frames, and asserts a partial-alpha state
  layer quad over the item chrome.

## Proof

Proof gate:

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test menu_state menu_pressed_state_layer_animates_over_item_chrome
```

The gate passed without implementation changes, proving the current Menu recipe already animates
pressed item indication through the shared Material ink runtime.

## Residual Risk

- Menu style remains `covered_v1`; this packet only closes item motion, not full color,
  typography, elevation, and shape token parity.
- DropdownMenu overlay motion is covered by
  `material3_menu_dropdown_layout_focus_motion_packet_v2.md`; this packet does not add new overlay
  behavior assertions.
- Leading/trailing icons, supporting text, group labels, checkbox/radio items, submenu triggers,
  shortcut text, and scroll/max-height behavior remain future Menu component-surface work.
