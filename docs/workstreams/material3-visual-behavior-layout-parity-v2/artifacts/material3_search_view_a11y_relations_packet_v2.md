# Material3 SearchView Accessibility Relations Packet v2

Date: 2026-05-28
Task: M3PV2-033

## Truth

- SearchView overlay ownership is Material recipe accessibility wiring, not caller-owned layout.
- Docked SearchView should expose an expanded input that controls the overlay panel.
- Full-screen SearchView should expose an expanded overlay header input that controls the dialog.
- The overlay panel/dialog should be labelled by the input that controls it.

## Sources

- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/SearchBar.kt`
  - `SearchBarDefaults.InputField` publishes expanded search semantics.
  - Full-screen SearchBar requests focus for the overlay-local input when expanded.
  - Expanded SearchBar handles Down-arrow traversal into search results.
- Fret-side component exemplars:
  - `ecosystem/fret-ui-material3/src/autocomplete.rs` wires the input to the listbox with
    `controls_element` and labels the listbox from the input.
  - `ecosystem/fret-ui-material3/tests/select_behavior.rs` proves Select combobox/listbox
    `controls` and `labelled_by` relations.

## Artifacts

- `ecosystem/fret-ui-material3/src/search_bar.rs`
- `ecosystem/fret-ui-material3/src/search_view.rs`
- `ecosystem/fret-ui-material3/tests/search_view_behavior.rs`

## Wiring

- `SearchBar` now accepts a crate-private controlled-element cell for SearchView composition.
- SearchView stores the current overlay surface id in per-slot state after the surface is rendered
  through `semantics_with_id`.
- Docked SearchView wraps the overlay surface as a `Panel`, labels it from the underlay input, and
  feeds the panel id back to the underlay SearchBar as `controls_element`.
- Full-screen SearchView wraps the overlay surface as a `Dialog`, labels it from the overlay header
  input, and feeds the dialog id back to that header SearchBar as `controls_element`.

## Proof

Red before fix:

```powershell
cargo nextest run -p fret-ui-material3 --test search_view_behavior search_view_inputs_control_overlay_semantics
```

The new gate failed because the docked SearchView input did not control the overlay panel.

Green after fix:

```powershell
cargo fmt --package fret-ui-material3
cargo nextest run -p fret-ui-material3 --test search_view_behavior
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_search_view_exposes_stable_part_test_ids material3_search_bar_exposes_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_search_view_suite_goldens_v1
cargo nextest run -p fret-ui-material3 --lib search_view search_bar
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
```

## Residual Risk

- SearchView motion remains open: predictive back and fixed-timestep open/close transitions still
  need a dedicated packet.
- Active-descendant or result-collection semantics remain open because SearchView currently accepts
  arbitrary overlay content rather than a typed results/listbox API.
- Down-arrow traversal into suggestions is not proven by this packet; it should be scoped with a
  future SearchView results API or kit-level focus policy.
