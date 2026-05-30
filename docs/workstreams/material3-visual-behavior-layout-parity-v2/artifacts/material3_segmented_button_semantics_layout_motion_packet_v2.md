# Material3 Segmented Button Semantics Layout Motion Packet v2

Date: 2026-05-29
Task: M3PV2-069

## Truth

- Single-choice SegmentedButton exposes a `RadioGroup` with `RadioButton` items, legacy binary
  checked flags, explicit `checked_state`, collection position metadata, and no selected flag.
- Multi-choice SegmentedButton exposes a generic group with `Checkbox` items, legacy binary checked
  flags, explicit `checked_state`, collection position metadata, and no selected flag.
- Segment touch targets remain 48px tall while visual chrome remains the Material 40px outlined
  container centered inside the target.
- Adjacent segments join without visible layout gap and keep stable `.chrome`, `.icon`, and
  `.label` part ids.
- Pressed state-layer opacity animates over the segment chrome through fixed frames instead of only
  being covered by broad scene goldens.

## Sources

- Compose Material3 `SegmentedButton.kt`: `SingleChoiceSegmentedButtonRow` applies
  `selectableGroup()`, `defaultMinSize(minHeight = ContainerHeight)`, intrinsic-width row
  negotiation, and negative spacing equal to the border width so adjacent outlines overlap.
- Compose Material3 `SegmentedButton.kt`: single-choice items apply `Role.RadioButton` semantics
  and selected state; multi-choice items use toggleable checked semantics.
- Compose Material3 `SegmentedButton.kt`: each item uses weight, default minimum button size,
  interaction z-index, and `SegmentedButtonDefaults.itemShape(index, count)` to round only the
  outer ends.
- Compose Material3 `OutlinedSegmentedButtonTokens.kt`: container height 40dp, outline width 1dp,
  full shape, icon size 18dp, label-large typography, selected secondary-container colors, and
  on-surface unselected colors.
- Base UI `toggle-group` / `radio-group` sources confirm the headless accessibility split: a
  grouped single-selection surface uses radio-like checked state, while multi-selection controls
  need explicit per-item pressed/checked state.

MUI Material UI does not provide a first-party SegmentedButton component in the local mirror used
for this packet; local Compose Material3 and Base UI references were sufficient for the audited
axes.

## Layer Finding

This packet found a Material recipe wiring/proof-density gap, not a core or kit mechanism gap:

- `fret-core` / `fret-ui` already expose `SemanticsCheckedState::{True,False}` and propagate it
  through `PressableA11y`.
- `fret-ui-kit` already contains radio, checkbox, and toggle-group helpers that demonstrate the
  intended checked-state contract, but Material SegmentedButton has recipe-specific collection
  metadata and single/multi role selection, so the minimal correct change stayed in the recipe.
- Material SegmentedButton already had roving focus, Home/End, RTL-aware horizontal arrow
  navigation, 48/40px touch/chrome split, joined segment borders, bounded ripple, and state-layer
  animation.
- The recipe only wrote the legacy binary `checked` flag and exposed `.chrome`; it did not publish
  explicit `checked_state` or stable content-slot part ids.

No core or kit primitive change was required in this packet.

## Artifacts

- `ecosystem/fret-ui-material3/src/segmented_button.rs`
- `ecosystem/fret-ui-material3/tests/segmented_button_state.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-ui-material3/tests/radio_alignment.rs`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json`

## Proof

Red gate before fix:

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test segmented_button_state
```

The cleaned red gate failed because SegmentedButton items did not write explicit `checked_state`
and did not expose `.icon` / `.label` content-slot part ids. The 48/40px geometry, joined seam, and
pressed state-layer probe already passed, confirming that the implementation gap was recipe
semantics and proof density.

Green gates:

```powershell
cargo fmt --package fret-ui-material3
cargo nextest run -p fret-ui-material3 --features diagnostics --test segmented_button_state
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_segmented_buttons_and_chips_expose_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --test radio_alignment segmented_button_semantics_roles_match_compose_baseline material3_headless_segmented_button_suite_goldens_v1
```

## Residual Risk

- This packet proves the current horizontal outlined SegmentedButtonSet surface. Compose's
  intrinsic-width row negotiation is represented in Fret by caller constraints plus equal flex
  segments; a future intrinsic sizing primitive could make this more exact.
- Exact Compose check-icon enter scale and content offset animation remain a focused follow-up.
  Current proof covers Material state-layer/ripple motion and stable content-slot anchors.
- Interaction z-index for overlapping selected/hovered outlines remains broad-golden covered rather
  than directly asserted as a draw-order invariant.
