# Material 3 Choice Controls And Chips Packet v1

Date: 2026-05-27
Task: M3CAS-080
Scope: `Checkbox`, `Radio`, `Switch`, `Slider`, `RangeSlider`, `SegmentedButtonSet`,
`AssistChip`, `SuggestionChip`, `FilterChip`, `InputChip`, `ChipSet`, and icon buttons.

## Outcome Contract

Choice controls need a different proof shape from overlay components. Their Material alignment is
mostly about intrinsic component behavior:

- selected/checked state must be exported through semantics,
- state layer and ripple must come from the shared Material indication path,
- visual chrome must stay token-driven while the interaction target observes minimum touch target
  policy,
- grouped controls must use roving focus and correct group/item semantics,
- action subparts such as chip trailing actions need stable selectors and keyboard focus routing.

The upstream axis used for this packet is:

- Compose Material3 for non-DOM state machines, state layer/ripple, semantics, and touch-target
  outcomes (`Checkbox.kt`, `RadioButton.kt`, `Switch.kt`, `Slider.kt`, `SegmentedButton.kt`,
  `Chip.kt`, `IconButton.kt` plus component token files under the local repo-ref mirror).
- Base UI for headless part decomposition and slider/checkbox/radio/switch accessibility patterns.
- Fret-side shadcn remains an implementation exemplar for stable `test_id` surfaces only, not the
  Material visual truth.

## Component Classification

| Component | Classification | Result |
| --- | --- | --- |
| `Checkbox` | `material_recipe`, `material_foundation`, `test_harness` | Recipe owns tri-state bool/optional model mapping and checkbox semantics. Existing `foundation::indication` and `foundation::interactive_size` own state layer/ripple and minimum target policy. New automation coverage proves root and `.chrome` selectors. |
| `Radio` / `RadioGroup` | `material_recipe`, `material_foundation`, `kit_policy` | Recipe owns radio dot geometry, APG-style group semantics, roving/typeahead wiring, and item selectors. Existing Material indication owns ripple/state-layer behavior. Existing gates prove selected-dot centering and pointer-origin ripple. |
| `Switch` | `material_recipe`, `material_foundation` | Existing seed packet remains valid. Switch owns track/handle/icon selectors and toggle animation; shared indication/minimum target policy remains in Material foundation. |
| `Slider` / `RangeSlider` | `material_recipe`, `diagnostics` | Recipe owns value model, keyboard/pointer value updates, range thumb semantics, canvas track/handle rendering, and value indicator. Stable selectors are root and range thumb semantics; internal track/handle canvas parts stay scene/golden-gated rather than fake semantic selectors. |
| `SegmentedButtonSet` | `material_recipe`, `kit_policy`, `test_harness` | Recipe owns single/multi selection, roles, checked state, per-segment chrome, and RTL-aware roving. Current `fret-ui` roving mechanism is sufficient; no new kit abstraction is proven by this packet. |
| `AssistChip` / `SuggestionChip` | `material_recipe`, `material_foundation` | Recipe owns chip label/icon composition and chrome selectors; shared indication/minimum target policy covers state layer/ripple/touch target behavior. |
| `FilterChip` / `InputChip` | `material_recipe`, `material_foundation`, `kit_policy` | Recipe owns selected semantics and trailing-action composition. Trailing actions expose `.trailing-icon` selectors and keyboard focus routing. Shared indication/minimum target policy remains Material foundation. |
| `ChipSet` | `material_recipe`, `test_harness` | Recipe owns group semantics, roving focus, wrapping/gap policy, and chip tab-stop delegation. Existing gate proves trailing-action focus still counts as the active chip. |
| `IconButton` / `IconToggleButton` | `material_recipe`, `material_foundation` | Recipe owns toggle semantics, selected shape morphing, and chrome selectors. Shared indication/minimum target policy owns state-layer/ripple and target sizing. |

## Stable Selector Surface

Newly proven or confirmed selector contracts:

- `checkbox`
- `checkbox.chrome`
- `radio_group`
- `radio`
- `radio.chrome`
- `switch`
- `switch.chrome`
- `switch.track`
- `switch.handle`
- `switch.icon-on`
- `switch.icon-off`
- `slider`
- `range_slider`
- `range_slider.start`
- `range_slider.end`
- `segmented_button`
- `segmented_button.item`
- `segmented_button.item.chrome`
- `assist_chip`
- `assist_chip.chrome`
- `suggestion_chip`
- `suggestion_chip.chrome`
- `filter_chip`
- `filter_chip.chrome`
- `filter_chip.trailing-icon`
- `input_chip`
- `input_chip.chrome`
- `input_chip.trailing-icon`
- `chip_set`
- `icon_button`
- `icon_button.chrome`
- `icon_toggle_button`
- `icon_toggle_button.chrome`

Intentionally not added in this packet:

- Slider internal canvas parts such as `slider.track`, `slider.handle`, and `slider.state-layer`.

Those are paint operations inside a single canvas. They are better protected by scene/golden tests
until Fret has a general diagnostics mechanism for named canvas draw regions.

## Implementation Anchors

- `ecosystem/fret-ui-material3/src/checkbox.rs`
- `ecosystem/fret-ui-material3/src/radio.rs`
- `ecosystem/fret-ui-material3/src/switch.rs`
- `ecosystem/fret-ui-material3/src/slider.rs`
- `ecosystem/fret-ui-material3/src/segmented_button.rs`
- `ecosystem/fret-ui-material3/src/chip.rs`
- `ecosystem/fret-ui-material3/src/suggestion_chip.rs`
- `ecosystem/fret-ui-material3/src/filter_chip.rs`
- `ecosystem/fret-ui-material3/src/input_chip.rs`
- `ecosystem/fret-ui-material3/src/chip_set.rs`
- `ecosystem/fret-ui-material3/src/icon_button.rs`
- `ecosystem/fret-ui-material3/src/foundation/indication.rs`
- `ecosystem/fret-ui-material3/src/foundation/interactive_size.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-ui-material3/tests/radio_alignment.rs`

## Gates

Passed on 2026-05-27:

```powershell
cargo fmt --package fret-ui-material3
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface
cargo nextest run -p fret-ui-material3 --test radio_alignment checkbox_pressed_scene_structure_is_stable
cargo nextest run -p fret-ui-material3 --test radio_alignment checkbox_tristate_semantics_and_toggle_outcomes
cargo nextest run -p fret-ui-material3 --test radio_alignment radio_selected_dot_is_centered_in_outline
cargo nextest run -p fret-ui-material3 --test radio_alignment radio_ripple_origin_tracks_pointer_down_position
cargo nextest run -p fret-ui-material3 --test radio_alignment switch_ripple_origin_tracks_pointer_down_position
cargo nextest run -p fret-ui-material3 --test radio_alignment switch_ripple_holds_for_minimum_press_duration_before_fade
cargo nextest run -p fret-ui-material3 --test radio_alignment icon_button_pressed_scene_structure_is_stable
cargo nextest run -p fret-ui-material3 --test radio_alignment chips_export_checked_state_for_selected_semantics
cargo nextest run -p fret-ui-material3 --test radio_alignment segmented_button_semantics_roles_match_compose_baseline
cargo nextest run -p fret-ui-material3 --test radio_alignment chip_set_roving_treats_trailing_action_focus_as_active_chip
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_segmented_button_suite_goldens_v1
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_slider_suite_goldens_v1
```

## Follow-Ons

- `M3CAS-080-F1`: Add diagnostics support for named canvas draw regions before exposing slider
  internal track/handle/state-layer selectors.
- `M3CAS-080-F2`: Evaluate whether RadioGroup and ChipSet roving/typeahead should move to
  `fret-ui-kit` after another design-system consumer needs the same policy.
- `M3CAS-080-F3`: Split broader chip visual parity if Assist/Suggestion/Filter/Input chip
  spacing/elevation drift appears in gallery diagnostics.
- `M3CAS-080-F4`: Consider a shared selected-control helper only if Checkbox, Radio, Switch, chips,
  and segmented buttons start duplicating nontrivial checked semantics or selected indicator logic.
