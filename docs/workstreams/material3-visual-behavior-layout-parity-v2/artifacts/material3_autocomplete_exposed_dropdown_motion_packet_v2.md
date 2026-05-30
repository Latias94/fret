# Material3 Autocomplete / ExposedDropdown Motion Packet v2

Date: 2026-05-28
Task: M3PV2-041

## Truth

- Autocomplete and ExposedDropdown popup motion is menu-like overlay motion: the listbox fades and
  scales from the transform origin while remaining mounted during close.
- ExposedDropdown's trailing icon is a visible expanded-state affordance. Fret's Autocomplete owns
  the shared trigger, so both Autocomplete with `trailing_dropdown_icon(true)` and ExposedDropdown
  must rotate the chevron on open and close state changes.
- This is Material recipe policy. `fret-ui-kit` already provides the overlay wrapping and Fret
  core already provides transforms/opacity; the drift was in Autocomplete's local trigger motion.

## Sources

- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ExposedDropdownMenu.kt`
  - `ExposedDropdownMenu` keeps a `MutableTransitionState` mounted while current or target state is
    expanded and delegates popup content to `DropdownMenuContent`.
  - `ExposedDropdownMenuDefaults.TrailingIcon` rotates the arrow to `180f` when expanded.
- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Menu.kt`
  - `DropdownMenuContent` animates scale with `MotionSchemeKeyTokens.FastSpatial`.
  - `DropdownMenuContent` animates alpha with effects motion and applies the transform origin.
- Fret Select M3PV2-037 is the local exemplar for chevron `FastSpatial` and overlay alpha/scale
  fixed-frame gates.

## Artifacts

- `ecosystem/fret-ui-material3/src/autocomplete.rs`
- `ecosystem/fret-ui-material3/src/tokens/dropdown_menu.rs`
- `ecosystem/fret-ui-material3/tests/autocomplete_motion.rs`
- `goldens/material3-headless/v1/material3-autocomplete.*.json`

## Wiring

- Autocomplete popup rendering was already routed through `foundation::overlay_motion`, so the
  alpha/scale overlay path was a proof gap rather than a new implementation gap.
- Autocomplete chevron motion now uses `SpringAnimator` with the scoped Material `FastSpatial`
  spring, matching Select's trigger motion foundation.
- Removed unused dropdown-menu duration/easing helpers after chevron motion stopped using the old
  duration/easing animator.
- ExposedDropdown inherits the fix through its Autocomplete composition.

## Proof

Red gate before fix:

```powershell
cargo nextest run -p fret-ui-material3 --test autocomplete_motion
```

It failed for both Autocomplete and ExposedDropdown because the first open frame had popup
alpha/scale motion but no chevron rotation.

Green gate:

```powershell
cargo fmt --package fret-ui-material3
cargo nextest run -p fret-ui-material3 --test autocomplete_motion
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_autocomplete_exposes_stable_part_test_ids material3_exposed_dropdown_popup_matches_field_chrome_bounds
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_autocomplete_semantics_v1 material3_exposed_dropdown_trailing_icon_toggles_overlay_v1 material3_exposed_dropdown_reverts_query_to_committed_selection_on_blur_v1
$env:FRET_UPDATE_GOLDENS='1'; try { cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_autocomplete_suite_goldens_v1 } finally { Remove-Item Env:FRET_UPDATE_GOLDENS -ErrorAction SilentlyContinue }
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_autocomplete_suite_goldens_v1
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/WORKSTREAM.json
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json
python tools/check_workstream_catalog.py
git diff --check
```

The Autocomplete headless suite was refreshed for the intentional current signature: field chrome
active-indicator layers and selectable option row clips are now part of the component baseline.

## Matrix Impact

- `autocomplete.motion`: `covered_v2`.
- `exposed_dropdown.motion`: `covered_v2`.

## Residual Risk

- This packet proves popup alpha/scale and trailing chevron motion. It does not claim new option
  ripple timing coverage; option state styling remains covered by the selectable-item packet.
- Autocomplete without a trailing dropdown icon has no visible chevron target; its popup motion is
  still covered by the same overlay gate.
