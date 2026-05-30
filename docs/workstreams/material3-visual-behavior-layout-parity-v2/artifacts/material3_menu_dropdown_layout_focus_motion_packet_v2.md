# Material3 Menu Dropdown Layout Focus Motion Packet v2

Date: 2026-05-29
Task: M3PV2-081

## Truth

- Menu items use the Compose Material3 48dp row height, 12dp horizontal content padding, 112dp
  minimum width, 280dp maximum width, and the menu surface applies 8dp vertical padding.
- DropdownMenu estimates anchored panel size with the same 112..280dp width bounds and includes
  menu vertical padding in placement geometry.
- Menu and DropdownMenu expose `Menu` / `MenuItem` semantics, collection position metadata, and
  disabled item semantics.
- Disabled menu items remain roving-focus targets, matching APG/Base UI menu behavior, but do not
  expose invoke actions and do not dispatch selection.
- DropdownMenu open and close frames use the shared Material overlay fade plus 0.8..1.0 scale
  motion, and initial focus lands on the first menu item, including a disabled first item.

## Sources

- Compose Material3 `Menu.kt`: `DropdownMenuContent` wraps the content column in
  `padding(vertical = DropdownMenuVerticalPadding)` and uses `width(IntrinsicSize.Max)`.
- Compose Material3 `Menu.kt`: plain `DropdownMenuItemContent` applies `sizeIn(minWidth =
  DropdownMenuItemDefaultMinWidth, maxWidth = DropdownMenuItemDefaultMaxWidth, minHeight =
  MenuListItemContainerHeight)` and horizontal 12dp content padding.
- Compose Material3 `Menu.kt`: dropdown menu visibility animates with `FastSpatial` scale from
  0.8 to 1.0 and `FastEffects` alpha from 0 to 1.
- Base UI Menu root/item tests and source were used as the headless accessibility reference for
  `menu` / `menuitem` roles, typeahead, and disabled-but-focusable menu navigation.

MUI Material UI was not available in this checkout's `repo-ref/`; local Compose Material3 and Base
UI references were sufficient for the audited layout, focus, accessibility, and motion axes.

## Layer Finding

This packet found a Material recipe/token gap, not a core or kit mechanism gap:

- `crates/fret-ui` already exposed the needed mechanisms: `Menu` / `MenuItem` roles, collection
  metadata, transparent `SemanticsDecoration` overrides, `PressableKeyActivation::None`, opacity,
  render transforms, and roving-focus primitives.
- `fret-ui-kit` already owned overlay dismissal/focus-restore policy for `dismissible_menu`; the
  existing DropdownMenu Escape/outside-press/focus-restore gate stayed green.
- Material Menu had row height and colors, but not Material menu width bounds or container vertical
  padding, and its roving focus skipped disabled items.
- Material DropdownMenu used a legacy 128dp minimum width, omitted vertical padding from placement
  size estimation, and had no fixed-frame motion/focus proof for the current recipe surface.

## Artifacts

- `ecosystem/fret-ui-material3/src/menu.rs`
- `ecosystem/fret-ui-material3/src/dropdown_menu.rs`
- `ecosystem/fret-ui-material3/src/tokens/menu.rs`
- `ecosystem/fret-ui-material3/tests/menu_state.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-ui-material3/tests/radio_alignment.rs`
- `goldens/material3-headless/v1/material3-menu-dialog-style.*.json`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json`

## Wiring

- `tokens::menu` now exposes typed accessors for item min/max width, container vertical padding,
  and item horizontal content padding.
- `Menu::into_element(...)` applies the menu surface's 8dp vertical padding and 112..280dp item
  width constraints to the menu surface, roving flex, pressable item, pointer region, and chrome
  row.
- Disabled `MenuItem`s remain focusable through roving focus by keeping the pressable enabled for
  focus routing, suppressing keyboard activation with `PressableKeyActivation::None`, not
  installing selection handlers, and attaching disabled/non-invokable semantics.
- `DropdownMenu::new(...)` now defaults to the Material 112dp minimum width, estimates popup
  placement with 112..280dp bounds, and includes menu vertical padding in panel height.
- DropdownMenu continues to use `foundation::overlay_motion::drive_overlay_open_close_motion` for
  Material fade-scale open/close frames.

## Proof

Red gate before the fix:

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test menu_state
```

Failed because menu rows measured as 48px wide instead of respecting Material's 112dp minimum,
DropdownMenu filled a 360px trigger instead of clamping to the 280dp menu maximum, and roving focus
skipped disabled menu items.

Green gates:

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test menu_state
cargo nextest run -p fret-ui-material3 --lib dropdown_menu menu
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_menu_and_dropdown_expose_stable_part_test_ids
$env:FRET_UPDATE_GOLDENS='1'; cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_menu_dialog_style_suite_goldens_v1; Remove-Item Env:\FRET_UPDATE_GOLDENS
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_menu_dialog_style_suite_goldens_v1 menu_pressed_scene_structure_is_stable menu_style_overrides_apply_to_container_and_label dropdown_menu_dismisses_and_restores_focus_across_schemes
```

The focused `menu_state` gate now proves Material menu item geometry bounds, 8dp vertical padding,
`Menu` / `MenuItem` semantics, disabled-but-focusable roving behavior, disabled non-invokable
semantics, DropdownMenu first-item initial focus, and open/close fade-scale motion. Refreshed
menu-dialog goldens record the intentional settled menu geometry shift across scale and theme
variants.

## Residual Risk

- Menu style remains `covered_v1`; this packet did not re-audit every menu color, typography,
  shape, elevation, and state-layer token beyond the existing style override and headless golden
  coverage.
- The current Menu API still lacks Compose/Base-UI-grade component breadth: leading/trailing icon
  slots, supporting text, group labels, checkbox/radio items, submenu triggers, shortcut text, and
  scroll/max-height behavior are future component-surface packets.
- The packet proves Material width bounds and padding, but not a full content-driven
  `IntrinsicSize.Max` width algorithm for every text/font combination. If exact intrinsic menu
  width becomes critical, add a dedicated width-probe or layout-intrinsic packet rather than
  hard-coding caller layout.
