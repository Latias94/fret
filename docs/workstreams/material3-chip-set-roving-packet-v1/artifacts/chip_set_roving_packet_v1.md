# Material 3 ChipSet Roving Packet v1

Date: 2026-05-28

## Truth

- ChipSet is a Material recipe container, not a core mechanism.
- ChipSet owns group semantics, gap/wrap defaults, roving focus, RTL-aware arrow behavior, and root
  selector stamping.
- Individual chips own their own chrome, selected state, disabled state, and primary/trailing action
  semantics.
- Kit extraction is deferred until cross-design-system reuse is proven.

## Artifacts

- `ecosystem/fret-ui-material3/src/chip_set.rs`
- `ecosystem/fret-ui-material3/src/chip.rs`
- `ecosystem/fret-ui-material3/src/filter_chip.rs`
- `ecosystem/fret-ui-material3/src/input_chip.rs`
- `ecosystem/fret-ui-material3/src/suggestion_chip.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-ui-material3/tests/radio_alignment.rs`
- `apps/fret-ui-gallery/src/ui/snippets/material3/state_matrix.rs`
- `docs/workstreams/material3-chip-visual-diagnostics-packet-v1/artifacts/chip_visual_diagnostics_packet_v1.md`

## Wiring

- `ChipSet` wraps chips in a group semantics surface and an existing roving flex container.
- Each child chip receives a roving tab-stop decision from ChipSet.
- Child chips keep their own disabled semantics; ChipSet has no container-level disabled state.
- InputChip and FilterChip keep their internal trailing-action focus behavior.
- The focused test proves parent roving treats a trailing-action focus as still inside the active
  chip when moving to the next chip.

## Proof

- `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_segmented_buttons_and_chips_expose_stable_part_test_ids`
- `cargo nextest run -p fret-ui-material3 --test radio_alignment chip_set_roving_treats_trailing_action_focus_as_active_chip`
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test chip_state chip_set_disabled_state_is_owned_by_child_chips`

## Residual Risk

- Keep the policy recipe-owned until another design system proves the same roving chip-group
  abstraction belongs in `fret-ui-kit`.
- Add wrapping/layout diagnostics only if a future product-visible drift appears.
