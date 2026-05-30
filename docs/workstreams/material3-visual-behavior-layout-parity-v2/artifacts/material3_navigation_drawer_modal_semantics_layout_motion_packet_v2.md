# Material3 NavigationDrawer ModalNavigationDrawer Semantics Layout Motion Packet v2

Date: 2026-05-29
Task: M3PV2-075

## Truth

- NavigationDrawer exposes a vertical `TabList` semantics node with author label, disabled state,
  and destination `Tab` children.
- Each drawer destination exposes selected state, disabled state, and collection position/count.
- NavigationDrawer uses the Material 360dp drawer width, 12dp external item padding, 336x56dp
  active indicator/item chrome, 16dp leading content padding, 24dp trailing content padding, and
  12dp icon-label spacing.
- ModalNavigationDrawer exposes a panel-level `Dialog` semantics node labelled `Navigation menu`
  and a scrim semantics label of `Close drawer`.
- ModalNavigationDrawer uses a 360dp left-aligned panel, full-window scrim, panel slide from the
  negative drawer-width closed anchor, and scrim alpha fade without panel opacity fade.

## Sources

- Compose Material3 `NavigationDrawer.kt`: `ModalNavigationDrawer` sets the drawer panel
  `paneTitle` to `Strings.NavigationMenu`, exposes a dismiss action while open, closes on Escape,
  and offsets the drawer by the current anchored-draggable offset from the negative sheet width to
  `0`.
- Compose Material3 `NavigationDrawer.kt`: `Scrim` uses `Strings.CloseDrawer` for its content
  description and fades alpha with the drawer open fraction.
- Compose Material3 `DrawerSheet`: width is constrained to `NavigationDrawerTokens.ContainerWidth`
  and the sheet fills max height.
- Compose Material3 `NavigationDrawerItem`: item semantics use `Role.Tab`,
  `heightIn(min = NavigationDrawerTokens.ActiveIndicatorHeight)`, `fillMaxWidth`, row padding
  start `16.dp` / end `24.dp`, icon spacing `12.dp`, and optional badge spacing `12.dp`.
- Compose Material3 `NavigationDrawerItemDefaults.ItemPadding` is `12.dp` horizontal, matching the
  token-derived `360dp - 336dp` external item inset.
- Base UI Tabs remains the supporting headless accessibility reference for vertical tablist
  orientation and selected tab semantics.

MUI Material UI was not available in this checkout's `repo-ref/`; local Compose Material3 and Base
UI references were sufficient for the audited layout, accessibility, and motion axes.

## Layer Finding

This packet found Material recipe proof and wiring gaps, not new core mechanism gaps:

- `fret-core` / `fret-ui` already exposed `TabList`, `Tab`, orientation, selected, disabled,
  collection metadata, `Dialog` role, labels, and layout-transparent semantics decorations.
- `fret-ui-kit` already owned the modal overlay/focus-trap/dismiss infrastructure used by
  ModalNavigationDrawer; the existing focus containment and restore gates stayed green.
- NavigationDrawer already had Material item geometry, roving behavior, selected state, disabled
  state, and collection metadata, but did not write the vertical tablist orientation.
- ModalNavigationDrawer already slid the panel from `-width` to `0` and faded the scrim, but the
  panel had only generic test-id semantics and the scrim had no close label.
- The fix stays in the Material recipe: root Drawer semantics now include vertical orientation, the
  modal panel attaches `Dialog`/`Navigation menu` semantics without adding layout nodes, and the
  scrim publishes `Close drawer`.

## Artifacts

- `ecosystem/fret-ui-material3/src/navigation_drawer.rs`
- `ecosystem/fret-ui-material3/src/modal_navigation_drawer.rs`
- `ecosystem/fret-ui-material3/tests/navigation_drawer_state.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-ui-material3/tests/radio_alignment.rs`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json`

## Proof

Red gate before fix:

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test navigation_drawer_state
```

The red gate failed because NavigationDrawer's `TabList` orientation was `None` and
ModalNavigationDrawer's panel semantics resolved as `Generic` instead of `Dialog`. The fixed-frame
modal slide test already proved the existing slide transform; the implementation gap was semantics
plus proof density.

Green gates:

```powershell
cargo fmt --package fret-ui-material3
cargo nextest run -p fret-ui-material3 --features diagnostics --test navigation_drawer_state
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_navigation_drawer_exposes_stable_part_test_ids material3_modal_navigation_drawer_exposes_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --test radio_alignment navigation_drawer_roving_skips_disabled_and_updates_model navigation_drawer_roving_wraps_and_skips_disabled_on_reverse navigation_drawer_roving_does_not_wrap_when_loop_navigation_false navigation_drawer_roving_single_enabled_item_does_not_move_under_no_loop modal_navigation_drawer_focus_is_contained_and_restored_across_schemes
cargo nextest run -p fret-ui-material3 --lib navigation_drawer modal_navigation_drawer
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/WORKSTREAM.json | Out-Null
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json | Out-Null
python tools/check_workstream_catalog.py
git diff --check
```

## Residual Risk

- This packet covers the current left-edge modal drawer and standard drawer destination recipe.
  Dismissible drawer gestures, predictive-back scaling, RTL slide direction, permanent drawer
  window insets, drawer headers, and adaptive NavigationSuite ownership remain future API work.
- NavigationDrawer standard item pressed state-layer motion is covered by
  `material3_navigation_drawer_item_motion_packet_v2.md`; standalone standard drawers still have
  no open/close surface motion because they are always-present navigation surfaces.
- Exact surface shape/elevation conflicts between Compose defaults and Material Web token aliases
  remain style-axis v1 coverage unless a later packet chooses a single authoritative variant model.
