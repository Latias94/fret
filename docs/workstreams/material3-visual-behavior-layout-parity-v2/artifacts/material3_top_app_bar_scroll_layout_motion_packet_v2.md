# Material3 TopAppBar Scroll Layout Motion Packet v2

Date: 2026-05-29
Task: M3PV2-076

## Truth

- TopAppBar exposes a toolbar semantics root with author label and stable automation anchors for
  `.chrome`, `.title`, `.collapsed-title`, and `.expanded-title`.
- Large TopAppBar collapses from 152dp to 64dp through the current scroll behavior, with a
  deterministic half-collapsed 108dp state at half scroll offset.
- The expanded Large title starts at the Material 16dp content inset under the 4dp app-bar
  horizontal padding plus Compose's 12dp title inset.
- Two-row TopAppBar title motion uses Compose's `TopTitleAlphaEasing` for the collapsed/top title
  and keeps the expanded/bottom title alpha linear at `1 - collapsedFraction`.
- Medium/Large container color moves through a scroll fraction using Material's
  FastOutLinearIn-style easing, while explicit `.scrolled(true)` without a scroll behavior still
  uses the fully scrolled color state.

## Sources

- Compose Material3 `AppBar.kt`: `TopAppBarHorizontalPadding = 4.dp`, `TopAppBarTitleInset = 16.dp
  - TopAppBarHorizontalPadding`, and Medium/Large top app bars use collapsed height 64dp with
  expanded heights 112dp and 152dp.
- Compose Material3 `AppBar.kt`: `TwoRowsTopAppBar` uses `TopTitleAlphaEasing =
  CubicBezierEasing(.8f, 0f, .8f, .15f)` for the top title alpha and `1f - collapsedFraction` for
  the bottom title alpha.
- Compose Material3 `AppBar.kt`: top-app-bar container color is driven by a transition fraction
  with FastOutLinearIn-style easing rather than only a boolean scrolled state.
- Material Web v30 top-app-bar tokens remain the Fret token source for container, elevation,
  shape, icon, and headline colors.
- Base UI was not a primary source for this packet because TopAppBar is not a composite widget
  requiring a headless keyboard/focus state machine.

MUI Material UI was not available in this checkout's `repo-ref/`; local Compose Material3 and
Material Web token references were sufficient for the audited layout and motion axes.

## Layer Finding

This packet found Material recipe/token proof gaps, not core or kit mechanism gaps:

- `fret-core` / `fret-ui` already exposed toolbar semantics, labels, test ids, scene bounds, and
  declarative scroll handles needed to prove the behavior.
- `fret-ui-kit` was not the owner: TopAppBar's scroll/collapse geometry, token colors, and title
  choreography are Material recipe concerns.
- TopAppBar already had pinned, enter-always, and exit-until-collapsed scroll behavior logic, but
  lacked stable part probes for the chrome and title layers.
- Medium/Large container color used a boolean scrolled state instead of the Compose transition
  fraction, and the top/collapsed title alpha used a linear fraction instead of Compose's
  top-title easing.
- During the packet, the existing headless golden caught a compatibility regression: Medium/Large
  bars with explicit `.scrolled(true)` but no scroll behavior must still resolve to the fully
  scrolled color. The final implementation keeps that path while using fractional color only when
  a scroll behavior supplies layout state.

## Artifacts

- `ecosystem/fret-ui-material3/src/top_app_bar.rs`
- `ecosystem/fret-ui-material3/src/tokens/top_app_bar.rs`
- `ecosystem/fret-ui-material3/tests/top_app_bar_alignment.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-ui-material3/tests/radio_alignment.rs`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json`

## Proof

Regression gate during implementation:

```powershell
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_top_app_bar_suite_goldens_v1
```

The regression gate failed while Medium/Large explicit `.scrolled(true)` bars without a scroll
behavior were mapped to `collapsed_fraction = 0`, which rendered the unscrolled container color.
The final state distinguishes scroll-behavior-driven fractions from the explicit boolean scrolled
surface.

Green gates:

```powershell
cargo fmt --package fret-ui-material3
cargo nextest run -p fret-ui-material3 --test top_app_bar_alignment
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_top_app_bar_suite_goldens_v1
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_surface_data_display_expose_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --lib top_app_bar
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/WORKSTREAM.json | Out-Null
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json | Out-Null
python tools/check_workstream_catalog.py
git diff --check
```

## Residual Risk

- This packet proves Large TopAppBar geometry and two-row motion directly; Medium uses the same
  token and helper paths but still relies on existing headless goldens for broad signature
  coverage.
- Pinned and enter-always scroll behavior state machines remain covered by existing lib tests, not
  by a fixed-frame visual diagnostics script.
- Flexible/expressive app bars, subtitle rows, predictive-back integration, RTL-specific title
  placement, and exhaustive per-theme color snapshots remain future API or broader visual-matrix
  work.
