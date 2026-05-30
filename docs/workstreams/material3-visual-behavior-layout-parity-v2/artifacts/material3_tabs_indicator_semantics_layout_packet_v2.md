# Material3 Tabs Indicator Semantics Layout Packet v2

Date: 2026-05-29
Task: M3PV2-073

## Truth

- Tabs expose a `TabList` semantics node with horizontal orientation.
- Each tab exposes `Tab` semantics, selected state, disabled state, and collection position/count.
- Fixed primary tabs draw the active indicator at Material's content-sized width with a 24px
  minimum, centered under the selected tab content rather than stretched across the whole tab slot.
- Scrollable primary tabs use Material's 52px edge padding and 90px minimum tab width.
- Tab labels expose stable `.label` part ids so content-width probes and automation can target the
  content node rather than the whole tab chrome.

## Sources

- Compose Material3 `TabRow.kt`: `PrimaryTabRow` uses `PrimaryIndicator` with
  `tabIndicatorOffset(..., matchContentSize = true)`, while `PrimaryScrollableTabRow` uses
  `ScrollableTabRowEdgeStartPadding = 52.dp` and `ScrollableTabRowMinTabWidth = 90.dp`.
- Compose Material3 `TabRow.kt`: primary indicator content width is clamped to a minimum `24.dp`.
- Compose Material3 `Tab.kt`: tabs use `Role.Tab` through `selectable`, with text content centered
  in the 48dp primary navigation tab container.
- Base UI Tabs list/tab/panel sources confirm the headless accessibility pattern: `tablist`
  orientation, `tab` selected state, and panel relations when a panel API exists.

MUI Material UI was not available in this checkout's `repo-ref/`; local Compose Material3 and Base
UI references were sufficient for the audited layout and accessibility axes.

## Layer Finding

This packet found a Material recipe gap plus a shared Material foundation gap:

- `fret-core` / `fret-ui` already had `TabList`, `Tab`, orientation, selected, disabled,
  collection metadata, and relation mechanisms. No core change was required.
- `fret-ui-kit` already has a richer headless Tabs primitive with horizontal orientation and
  panel relation helpers. Material `Tabs` is currently a tab-row recipe, not the panel-owning API,
  so no kit policy change was required.
- Material `Tabs` did not write horizontal orientation, did not expose tab label part ids, measured
  the primary indicator from the full tab slot, and let scrollable tab measurement drift away from
  Compose's edge/min-width defaults.
- `foundation::active_indicator` existed, but its canvas relied on absolute inset sizing without
  explicit fill dimensions. In the current Fret layout engine that left a live test id without a
  painted indicator quad. The shared helper now fills its parent explicitly.

## Artifacts

- `ecosystem/fret-ui-material3/src/tabs.rs`
- `ecosystem/fret-ui-material3/src/tokens/tabs.rs`
- `ecosystem/fret-ui-material3/src/foundation/active_indicator.rs`
- `ecosystem/fret-ui-material3/tests/tabs_state.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-ui-material3/tests/radio_alignment.rs`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json`

## Proof

Red gate before fix:

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test tabs_state
```

The red gate failed because `TabList` orientation was `None`, fixed primary tabs produced no
painted active-indicator quad, and scrollable tabs started at the tab row edge instead of after the
52px Material edge padding.

Green gates:

```powershell
cargo fmt --package fret-ui-material3
cargo nextest run -p fret-ui-material3 --features diagnostics --test tabs_state
cargo nextest run -p fret-ui-material3 --lib tabs
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_tabs_exposes_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --test radio_alignment tabs_pressed_scene_structure_is_stable
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_navigation_bar_exposes_stable_part_test_ids material3_navigation_rail_exposes_stable_part_test_ids
```

## Residual Risk

- This packet covers the current text-only primary Tabs recipe. Leading-icon tabs, secondary tabs,
  and panel-owning Material Tabs APIs remain unexposed by the current Fret Material3 surface.
- Scroll-to-selected behavior for overflowed scrollable tabs remains behavior/motion residual work;
  this packet only locks edge padding and per-tab minimum sizing.
- NavigationBar and NavigationRail share the fixed active-indicator canvas foundation, but their
  own layout/accessibility rows remain separate matrix packets.
