# Material3 Icon Button Semantics Layout Motion Packet v2

Date: 2026-05-29
Task: M3PV2-070

## Truth

- Non-toggle IconButton exposes `Button` semantics and does not publish checked state.
- Toggle-style IconButton and IconToggleButton expose `Checkbox` semantics, legacy binary checked
  flags, explicit `checked_state`, and do not set selected semantics.
- Icon buttons keep a 48px touch target, 40px visual chrome, and 24px icon content.
- Icon buttons expose stable `.chrome` and `.icon` part ids for diagnostics and automation.
- Pressed state-layer opacity animates over the 40px chrome through fixed frames.

## Sources

- Compose Material3 `IconButton.kt`: `SurfaceIconButton` applies `Role.Button`, while
  `SurfaceIconToggleButton` applies `Role.Checkbox` and checked state through the Surface
  toggleable path.
- Compose Material3 `IconButton.kt`: small icon button content uses
  `Modifier.size(IconButtonDefaults.smallContainerSize())` with centered content.
- Compose Material3 `SmallIconButtonTokens.kt`: 40dp container height, 24dp icon size, 8dp
  leading/trailing space, 1dp outlined stroke, full default shape, small pressed shape, and
  selected shape tokens.
- Compose Material3 icon button shape helpers use interaction-driven pressed/checked shape
  resolution backed by Material motion scheme springs.
- Base UI `Button` and `CheckboxRoot` sources confirm the accessibility split: ordinary icon
  buttons are button-like, while toggleable icon buttons need checkbox-like checked state.

MUI Material UI is not needed for this packet because the audited axes are toolkit semantics,
touch/chrome sizing, and Material motion tokens; local Compose Material3 plus Base UI references
are the higher-precedence sources for those axes.

## Layer Finding

This packet found a Material recipe wiring/proof-density gap, not a core or kit mechanism gap:

- `fret-core` / `fret-ui` already expose explicit `SemanticsCheckedState::{True,False}` through
  `PressableA11y`.
- Material IconButton already had 48px minimum touch target enforcement, 40px chrome, 24px icon
  tokens, shape motion through `FastSpatial`, and state-layer/ripple wiring.
- The recipe only wrote the legacy binary `checked` flag for toggle surfaces and did not expose a
  stable `.icon` part id.
- Existing broad scene tests covered pressed structural stability, but not the explicit state-layer
  alpha path or part-level geometry.

No core or kit primitive change was required in this packet.

## Artifacts

- `ecosystem/fret-ui-material3/src/icon_button.rs`
- `ecosystem/fret-ui-material3/tests/icon_button_state.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-ui-material3/tests/radio_alignment.rs`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json`

## Proof

Red gate before fix:

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test icon_button_state
```

The cleaned red gate failed because toggle IconButton/IconToggleButton did not publish explicit
`checked_state` and IconButton did not expose `.icon` part ids. The pressed state-layer probe
already passed, confirming that the implementation gap was semantics/part wiring plus proof
density.

Green gates:

```powershell
cargo fmt --package fret-ui-material3
cargo nextest run -p fret-ui-material3 --features diagnostics --test icon_button_state
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_choice_controls_expose_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --test radio_alignment icon_toggle_button_semantics_role_and_checked_state_are_stable icon_button_pressed_scene_structure_is_stable
```

## Residual Risk

- This packet proves the current small IconButton/IconToggleButton API. Medium/large/x-large
  expressive icon button sizes remain unexposed by the current Fret Material3 API.
- Exact shape corner interpolation is still covered by pressed structural stability plus the
  existing spring path, not a dedicated corner-radius timeline assertion.
- This packet does not audit icon-button placement inside app bars or toolbars; those remain
  navigation/top-app-bar layout packet concerns.
