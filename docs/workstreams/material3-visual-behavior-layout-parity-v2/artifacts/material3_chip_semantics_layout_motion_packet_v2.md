# Material3 Chip Semantics Layout Motion Packet v2

Date: 2026-05-29
Task: M3PV2-071

## Truth

- AssistChip and SuggestionChip expose `Button` semantics and route primary activation through the
  chip root.
- FilterChip and InputChip expose `Checkbox` semantics, legacy binary checked flags, and explicit
  `checked_state`.
- Chip roots keep a 48px touch target while visual chrome stays at the Material 32px container
  height and icon slots stay at 18px.
- Chip content exposes stable `.label`, `.leading-icon`, `.trailing-icon`, and actionable
  `.trailing-icon.glyph` part ids for diagnostics and automation.
- Input/Filter trailing actions route independently from primary chip toggles.
- ChipSet exposes a labelled `Group`, keeps the 8px Material gap, and preserves the same gap when
  wrap layout places chips on separate rows.
- Pressed chip state-layer opacity animates over the 32px chrome through fixed frames.

## Sources

- Compose Material3 `Chip.kt`: non-selectable chips apply button-like semantics through the
  ordinary `Chip` path, while `FilterChip` and `InputChip` use the selectable chip path with
  checkbox-like checked semantics.
- Compose Material3 `Chip.kt`: selectable chip content is split into label, leading/avatar, and
  trailing slots, and animated chip content is reserved for selectable leading/trailing icon
  presence changes.
- Compose Material3 token files:
  - `AssistChipTokens.kt`
  - `SuggestionChipTokens.kt`
  - `FilterChipTokens.kt`
  - `InputChipTokens.kt`

  These define the 32dp container height and 18dp icon sizes used by the current Fret recipe
  tokens.
- Base UI `Button` and checkbox sources confirm the headless accessibility split: ordinary chips
  are button-like action surfaces, while selectable chips need checkbox-like checked state.

The local `repo-ref/` mirror for this checkout contains Compose Multiplatform Core and Base UI but
does not contain the MUI Material UI mirror, so this packet uses Compose as the primary source for
toolkit semantics, touch sizing, and state-layer behavior, with Base UI as the headless semantics
cross-check.

## Layer Finding

This packet found a Material recipe wiring/proof-density gap, not a core or kit mechanism gap:

- `fret-core` / `fret-ui` already expose `SemanticsCheckedState::{True,False}` through
  `PressableA11y`.
- Material chips already used the Material interactive-size helper, 32px chip chrome tokens, 18px
  icon tokens, and the shared Material ink/state-layer runtime.
- FilterChip and InputChip wrote checked metadata but still exposed the selectable root as
  `Button`.
- Chip content had stable root/chrome ids, but not the shadcn-style content part ids needed to
  prove label/icon/trailing-action geometry.
- ChipSet already owned the roving/wrap policy at the Material recipe layer; the missing piece was
  a focused v2 gap/wrap proof.

No core or kit primitive change was required in this packet.

## Artifacts

- `ecosystem/fret-ui-material3/src/chip.rs`
- `ecosystem/fret-ui-material3/src/suggestion_chip.rs`
- `ecosystem/fret-ui-material3/src/filter_chip.rs`
- `ecosystem/fret-ui-material3/src/input_chip.rs`
- `ecosystem/fret-ui-material3/tests/chip_state.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-ui-material3/tests/radio_alignment.rs`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json`

## Proof

Red gate before fix:

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test chip_state
```

The cleaned red gate failed because FilterChip/InputChip exposed `Button` instead of `Checkbox`
roles, chip content did not expose stable `.label` / icon part ids, and the focused pressed
state-layer probe initially had no packet-level assertion. During behavior proof, a root-center
click on short InputChip was also found to hit the trailing action slot; the final gate clicks the
primary and trailing regions separately and proves they route independently.

Green gates:

```powershell
cargo fmt --package fret-ui-material3
cargo nextest run -p fret-ui-material3 --features diagnostics --test chip_state
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_segmented_buttons_and_chips_expose_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --test radio_alignment chips_export_checked_state_for_selected_semantics chip_set_roving_treats_trailing_action_focus_as_active_chip material3_headless_controls_suite_goldens_v1
```

## Residual Risk

- Exact Compose `AnimatingChipContent` expand/shrink/fade behavior for selectable icon presence is
  not yet implemented or directly proven.
- InputChip avatar-slot parity remains residual because the current Fret InputChip API exposes
  leading/trailing icons but no avatar slot.
- Material Expressive corner morphing samples are not represented by the current public chip API.
- This packet proves current chip state-layer motion, not full color/elevation token interpolation
  across every hovered/focused/disabled combination.
