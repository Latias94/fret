# Material 3 Radio Scene Semantics Packet v1

Date: 2026-05-28

## Truth

- Radio exports checked semantics through `SemanticsRole::RadioButton`.
- RadioGroup exports group semantics and coordinates recipe-owned roving/typeahead behavior.
- The selected dot is centered in the outline and uses token-driven icon geometry.
- State-layer/ripple and minimum interactive sizing come from shared Material foundation.

## Artifacts

- `ecosystem/fret-ui-material3/src/radio.rs`
- `ecosystem/fret-ui-material3/src/foundation/indication.rs`
- `ecosystem/fret-ui-material3/src/foundation/interactive_size.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-ui-material3/tests/radio_alignment.rs`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_choice_controls_packet_v1.md`
- `docs/workstreams/material3-switch-diagnostics-packet-v1/artifacts/switch_diagnostics_packet_v1.md`

## Wiring

- `Radio` composes a pressable radio semantics node with shared Material indication and chrome
  stamping.
- `RadioGroup` wraps items in a radiogroup semantics surface and delegates focus movement through
  existing roving primitives.
- `automation_surface` proves root/chrome selectors are live.
- `radio_alignment` proves selected-dot centering, pointer-origin ripple, and pressed-scene
  stability.

## Proof

- `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_choice_controls_expose_stable_part_test_ids`
- `cargo nextest run -p fret-ui-material3 --test radio_alignment radio_selected_dot_is_centered_in_outline radio_ripple_origin_tracks_pointer_down_position radio_pressed_scene_structure_is_stable`

## Residual Risk

- Keep RadioGroup roving/typeahead recipe-owned until cross-design-system reuse proves a kit-policy
  need.
- Add gallery diagnostics only if a future product-visible Radio page drift appears.
