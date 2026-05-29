# Material3 NavigationBar NavigationRail Semantics Layout Packet v2

Date: 2026-05-29
Task: M3PV2-074

## Truth

- NavigationBar exposes a horizontal `TabList` semantics node; NavigationRail exposes a vertical
  `TabList` semantics node.
- Each destination exposes `Tab` semantics, selected state, disabled state, and collection
  position/count.
- NavigationBar uses Compose Material3's 8dp horizontal gap between weighted destinations and keeps
  the 80dp container height.
- NavigationBar places the active indicator in the Compose top-icon lane with the 12dp top offset,
  64x32dp indicator geometry, and icon-centered active indicator paint.
- NavigationRail uses the collapsed rail 80dp item width, 56dp item height, 4dp vertical spacing,
  and 56x32dp active indicator geometry.

## Sources

- Compose Material3 `NavigationBar.kt`: `NavigationBar` uses a full-width row with
  `defaultMinSize(minHeight = NavigationBarHeight)`, `Arrangement.spacedBy(8.dp)`, and
  `NavigationBarHeight = NavigationBarTokens.TallContainerHeight`.
- Compose Material3 `NavigationBar.kt`: each `NavigationBarItem` uses `Role.Tab`; the indicator
  width/height are derived from the 24dp icon plus 20dp horizontal and 4dp vertical padding,
  yielding 64x32dp, with `IndicatorVerticalOffset = 12.dp`.
- Compose Material3 `NavigationRail.kt`: the collapsed rail uses
  `NavigationRailCollapsedTokens.NarrowContainerWidth` for the 80dp rail/item width,
  `NavigationRailItemHeight = NavigationRailVerticalItemTokens.ActiveIndicatorWidth` for the 56dp
  item height, and `NavigationRailVerticalPadding = 4.dp`.
- Base UI Tabs remains the supporting headless accessibility reference for `tablist` orientation
  and tab selected-state semantics.

MUI Material UI was not available in this checkout's `repo-ref/`; local Compose Material3 and Base
UI references were sufficient for the audited layout and accessibility axes.

## Layer Finding

This packet found Material recipe and typed-token accessor gaps, not core or kit mechanism gaps:

- `fret-core` / `fret-ui` already exposed `TabList`, `Tab`, orientation, selected, disabled, and
  collection metadata mechanisms.
- `fret-ui-kit` was not the owner here: current Material NavigationBar/Rail recipes own their
  Material visual density, while the existing roving policy already covered keyboard movement.
- NavigationBar did not write horizontal orientation, used no horizontal item gap, and centered the
  indicator/icon/label stack instead of using Compose's 12dp top indicator offset.
- NavigationRail did not write vertical orientation, applied the 4dp rail padding on all sides
  instead of vertical-only, and let destination chrome collapse to the interactive minimum rather
  than the collapsed rail's 80x56dp item geometry.
- NavigationBar/Rail active indicators now resolve targets against the container bounds, with
  deterministic fallbacks before probe bounds are available.

## Artifacts

- `ecosystem/fret-ui-material3/src/navigation_bar.rs`
- `ecosystem/fret-ui-material3/src/navigation_rail.rs`
- `ecosystem/fret-ui-material3/src/tokens/navigation_bar.rs`
- `ecosystem/fret-ui-material3/src/tokens/navigation_rail.rs`
- `ecosystem/fret-ui-material3/tests/navigation_state.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-ui-material3/tests/radio_alignment.rs`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json`

## Proof

Red gate before fix:

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test navigation_state
```

The red gate failed because both navigation roots had `TabList` orientation `None`,
NavigationBar's item gap was `0px` instead of `8px`, and NavigationRail item chrome collapsed to
`48px` width instead of the 80dp collapsed rail item width.

Green gates:

```powershell
cargo fmt --package fret-ui-material3
cargo nextest run -p fret-ui-material3 --features diagnostics --test navigation_state
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_navigation_bar_exposes_stable_part_test_ids material3_navigation_rail_exposes_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --test radio_alignment navigation_bar_roving
cargo nextest run -p fret-ui-material3 --test radio_alignment navigation_rail_roving
cargo nextest run -p fret-ui-material3 --lib navigation_bar
cargo nextest run -p fret-ui-material3 --lib navigation_rail
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/WORKSTREAM.json | Out-Null
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json | Out-Null
python tools/check_workstream_catalog.py
git diff --check
```

## Residual Risk

- This packet covers the current collapsed NavigationBar/Rail destination recipes. Adaptive
  NavigationSuite selection, wide rails, modal wide rails, and rail headers remain future API work.
- NavigationBar/Rail motion axes remain at their prior v1 coverage; this packet only verifies
  settled active-indicator geometry after the existing spring reaches the target.
- NavigationDrawer, ModalNavigationDrawer, and TopAppBar remain open navigation-family packets.
