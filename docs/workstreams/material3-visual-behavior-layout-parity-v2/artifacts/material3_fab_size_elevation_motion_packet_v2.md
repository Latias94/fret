# Material3 FAB Size, Elevation, And Motion Packet v2

Date: 2026-05-29
Task: M3PV2-062

## Truth

- Icon FABs keep the Material interactive target separate from the visual chrome: small FAB root is
  48px while the visual chrome remains 40px; regular, medium, and large FAB chrome resolves to
  56px, 80px, and 96px respectively.
- Extended FABs use size-specific Material tokens for height, minimum width, shape, icon size,
  label spacing, and label typography.
- FABs paint Material elevation by default, animate hover elevation instead of snapping on the
  first hover frame, and keep requesting frames while elevation motion is active.
- Primary lowered FABs resolve the lowered elevation token path instead of falling back to normal
  primary-container elevation.
- FAB semantics expose button role, accessible label, disabled flag, and disabled invoke behavior.

## Sources

- Compose Material3 `FloatingActionButton.kt`: icon FABs route through a `Surface` with
  `Role.Button`; small/default/medium/large size APIs use 40/56/80/96dp outcomes; extended FABs
  use size-specific min height, min width, padding, text style, and animated elevation.
- Compose Material3 FAB token files: `FabSmallTokens`, `FabMediumTokens`, `FabLargeTokens`,
  `ExtendedFabSmallTokens`, `ExtendedFabMediumTokens`, `ExtendedFabLargeTokens`, and
  `ExtendedFabPrimaryTokens`.
- Material Web v30 token exports: `md.comp.fab.*` and `md.comp.extended-fab.*` include regular,
  small, medium, large, lowered, hover, focus, pressed, state-layer, shape, icon, and spacing
  scalars.

MUI Material UI is not available in this worktree's `repo-ref/`; this packet used local Compose
and generated Material Web token snapshots.

## Layer Finding

This packet found Material recipe/token wiring gaps, not a core or kit mechanism gap:

- The FAB recipe used min-size chrome, so flex layout stretched the visual chrome to the 48px
  touch target and sometimes wider.
- `FabSize::Medium` was standing in for the default 56px FAB. The recipe now has an explicit
  default `Regular` size and reserves `Medium` for the Material 80px FAB.
- Extended FAB ignored `size`, so medium/large extended FABs could not express Material 80/96px
  outcomes.
- FAB hover elevation resolved the right target token but painted it immediately instead of using
  the shared `foundation::elevation` runtime proven by Button, Card, and CarouselItem.
- Primary/secondary/tertiary lowered elevation tokens use the shorter Material Web alias
  (`md.comp.fab.primary.lowered.*`), while the normal color/state paths use `primary-container`.

The existing Fret pressable, semantics, diagnostics test-id registry, frame scheduling, and scene
shadow primitives were sufficient.

## Artifacts

- `ecosystem/fret-ui-material3/src/fab.rs`
- `ecosystem/fret-ui-material3/src/tokens/fab.rs`
- `ecosystem/fret-ui-material3/tests/fab_state.rs`
- `goldens/material3-headless/v1/material3-fab.*.json`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json`

## Proof

Red gate before fix:

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test fab_state
```

Failed because small FAB `.chrome` stretched to 48px, medium/large extended FABs stayed 56px high,
primary hover elevation snapped to the hover shadow on the first hover frame, and primary lowered
FABs did not use the lowered elevation path.

Green gates:

```powershell
cargo fmt --package fret-ui-material3
cargo nextest run -p fret-ui-material3 --features diagnostics --test fab_state
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_surface_data_display_expose_stable_part_test_ids
$env:FRET_UPDATE_GOLDENS='1'; cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_fab_suite_goldens_v1
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_fab_suite_goldens_v1
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/WORKSTREAM.json | Out-Null
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json | Out-Null
python tools/check_workstream_catalog.py
git diff --check
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
```

## Residual Risk

- FAB show/hide and extended collapsed/expanded choreography remain outside this standalone FAB
  packet because Fret's current FAB API has no visibility/expanded state surface.
- Scaffold/BottomAppBar FAB placement remains caller/container owned.
