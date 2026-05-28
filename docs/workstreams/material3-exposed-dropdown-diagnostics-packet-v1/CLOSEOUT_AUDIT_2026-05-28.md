# Material 3 Exposed Dropdown Diagnostics Packet v1 - Closeout Audit

Date: 2026-05-28
Status: Closed

## Scope

Closed the ExposedDropdown matrix residual with a promoted diagnostics suite and focused recipe
state-model gates.

## Result

- Material3 ExposedDropdown filtering diagnostics passed.
- Blur synchronization and trailing icon overlay toggle Rust gates passed.
- No component, foundation, kit-policy, or mechanism change was needed.

## Gates

- `python -m json.tool tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-exposed-dropdown-filtering.json`
- `python -m json.tool tools/diag-scripts/suites/ui-gallery-material3-exposed-dropdown-filtering/suite.json`
- `python tools/check_diag_scripts_registry.py`
- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-exposed-dropdown-filtering.json --dir target/fret-diag/material3-exposed-dropdown-filtering-20260528-pass --session-auto --pack --ai-packet --exit-after-run --timeout-ms 900000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3`
- `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_exposed_dropdown_reverts_query_to_committed_selection_on_blur_v1`
- `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_exposed_dropdown_trailing_icon_toggles_overlay_v1`

## Layering

- `material_recipe`: committed selection, editable query, trailing dropdown icon, blur-time display
  synchronization, and exposed-dropdown recipe composition.
- `material_foundation`: inherited Autocomplete/TextField field chrome, active-indicator, and
  dotted part-id helpers.
- `diagnostics`: promoted the Material3 gallery filtering popup gate and recorded bundle evidence.
- `kit_policy`: no new shared policy gap was found.
- `mechanism`: no core mechanism gap was found.
