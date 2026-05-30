# Exposed Dropdown Diagnostics Packet v1

## Truth

- Material3 `ExposedDropdown` composes Autocomplete and inherits stable root, field chrome,
  trailing icon, listbox, option, and option chrome selectors.
- Typing a filter query while the popup is open keeps matching options mounted and removes
  non-matching options from the diagnostics tree.
- The popup remains within the window and can be closed through the trailing dropdown icon.
- Committed selection and editable query ownership remain Material recipe policy; no shared kit or
  runtime mechanism change is required.

## Artifacts

- `ecosystem/fret-ui-material3/src/exposed_dropdown.rs`
- `ecosystem/fret-ui-material3/src/autocomplete.rs`
- `ecosystem/fret-ui-material3/src/text_field.rs`
- `apps/fret-ui-gallery/src/ui/snippets/material3/autocomplete.rs`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-exposed-dropdown-filtering.json`
- `tools/diag-scripts/suites/ui-gallery-material3-exposed-dropdown-filtering/suite.json`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`

## Wiring

The filtering script opens the Material3 Autocomplete gallery page from the UI Gallery navigation,
scrolls the `ui-gallery-material3-exposed-dropdown` field into view, clicks the input surface,
types `ga`, waits for `ui-gallery-material3-exposed-dropdown.option.gamma`, asserts
`ui-gallery-material3-exposed-dropdown.option.alpha` is absent, verifies the listbox is within the
window, captures open evidence, closes through the trailing icon, and captures closed evidence.

## Proof

```powershell
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-exposed-dropdown-filtering.json --dir target/fret-diag/material3-exposed-dropdown-filtering-20260528-pass --session-auto --pack --ai-packet --exit-after-run --timeout-ms 900000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_exposed_dropdown_reverts_query_to_committed_selection_on_blur_v1
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_exposed_dropdown_trailing_icon_toggles_overlay_v1
```

Results:

- filtering diagnostics: `PASS`, run id `1779945219795`
- blur synchronization Rust gate: `PASS`
- trailing icon overlay toggle Rust gate: `PASS`

Bounded evidence:

- `target/fret-diag/material3-exposed-dropdown-filtering-20260528-pass/sessions/1779944866704-59440/1779945219795/ai.packet`
- `target/fret-diag/material3-exposed-dropdown-filtering-20260528-pass/sessions/1779944866704-59440/share/1779945219795.zip`
- Open filtering query evidence:
  `target/fret-diag/material3-exposed-dropdown-filtering-20260528-pass/sessions/1779944866704-59440/1779945241904-ui-gallery-material3-exposed-dropdown-filtering`

## Residual Risk

No current ExposedDropdown recipe, foundation, kit-policy, or mechanism residual remains from this
packet. Broader Autocomplete option chrome and dialog placement diagnostics remain owned by the
Autocomplete packet/gates.
