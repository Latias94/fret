# Material3 Card Semantics And Elevation Packet v2

Date: 2026-05-29

## Truth

- Non-interactive cards are surfaces/groups, not disabled buttons.
- Clickable cards keep button semantics, enabled/disabled state, focus ring, state layer, and
  bounded ripple behavior.
- Interactive card elevation changes animate on hover/focus/press instead of snapping on the first
  state frame.
- Elevation animation is a Material foundation concern shared by Button and Card.

## Sources

- Compose Material3 `Card.kt`: non-clickable card overload delegates to `Surface` without input
  semantics; clickable overload delegates to clickable `Surface` with `enabled` and interaction
  source.
- Compose Material3 `CardElevation.shadowElevation`: interaction elevation is animated and disabled
  transitions snap.
- MUI Material UI `Card.js`: Card is a Paper-backed root surface; interactivity is not intrinsic to
  the base Card component.
- Existing Fret Material Button packet: Button already had local Compose-like elevation animation,
  which justified extracting it into Material foundation once Card needed the same behavior.

## Layer Finding

This packet found both a recipe gap and a foundation gap:

- Recipe: `Card` reused a `Pressable` wrapper even for non-interactive cards, so static cards leaked
  disabled button semantics.
- Foundation: elevation animation was implemented locally in Button but Card needed the same
  Compose `animateElevation` behavior. The shared runtime now lives in
  `ecosystem/fret-ui-material3/src/foundation/elevation.rs`.

No `crates/fret-ui` mechanism changes were needed; existing semantics decoration, pressable, and
frame scheduling surfaces were sufficient.

## Artifacts

- `ecosystem/fret-ui-material3/src/card.rs`
- `ecosystem/fret-ui-material3/src/button.rs`
- `ecosystem/fret-ui-material3/src/foundation/elevation.rs`
- `ecosystem/fret-ui-material3/tests/card_state.rs`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json`

## Proof

Red gate before fix:

```powershell
cargo nextest run -p fret-ui-material3 --test card_state
```

Failed because static cards exposed `Button` role and interactive filled cards painted hover
shadows on the first hover frame.

Green gates:

```powershell
cargo nextest run -p fret-ui-material3 --test card_state
cargo nextest run -p fret-ui-material3 --test button_state
cargo nextest run -p fret-ui-material3 --lib button
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_surface_data_display_expose_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_controls_suite_goldens_v1
```

## Residual Risk

- Card layout remains caller-owned: the recipe does not impose width, height, grid placement, or
  content padding beyond its surface chrome.
- Selectable cards are not a distinct public recipe yet; adding them should reuse the same
  foundation elevation runtime and add selection semantics in a separate packet.
