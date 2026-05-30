# Material3 Autocomplete / ExposedDropdown Listbox Packet v2

Date: 2026-05-28
Task: M3PV2-022
Status: Done

## Truth

- Autocomplete's implicit popup selector uses the same dotted part-id contract as explicit
  `.test_id(base)` usage: `material3-autocomplete.listbox`.
- ExposedDropdown is a composition over Autocomplete, but the rendered surface still exposes a
  first-class combobox input and a controlled listbox popup.
- The ExposedDropdown input reports `expanded=true` while open, controls the listbox, and the
  listbox is labelled by the input.
- Live Material3 Select diagnostics target the current dotted `<base>.listbox` listbox ids after
  the Select recipe changed its derived part id.

## Sources

- Fret field-family checklist:
  `.agents/skills/fret-material-source-alignment/references/material-field-family-checklist.md`.
- Base UI autocomplete harness:
  `repo-ref/base-ui/test/public-types/autocomplete.tsx`.
- Base UI combobox list role wiring:
  `repo-ref/base-ui/packages/react/src/combobox/list/ComboboxList.tsx`.
- Local `repo-ref/material-ui` was not present in this checkout; this packet used Base UI for the
  headless part contract and existing Fret Material tests for semantics proof.

## Layering

- Owner: `ecosystem/fret-ui-material3`.
- No `crates/fret-ui` or `ecosystem/fret-ui-kit` change was justified. The existing mechanism can
  express the required combobox/listbox relationship; the drift was in Material recipe selectors
  and first-party diag scripts.

## Artifacts

- `ecosystem/fret-ui-material3/src/autocomplete.rs`
  - changed the fallback listbox id from `material3-autocomplete-listbox` to
    `material3-autocomplete.listbox`;
  - added a focused unit test for the fallback contract.
- `ecosystem/fret-ui-material3/tests/radio_alignment.rs`
  - strengthened `material3_exposed_dropdown_trailing_icon_toggles_overlay_v1` to assert
    ComboBox/ListBox roles plus `controls` and `labelled_by` wiring.
- `tools/diag-scripts/ui-gallery/material3/*select*.json`
- `tools/diag-scripts/ui-gallery/overlay/ui-gallery-material3-select-*.json`
  - updated Material3 Select live diagnostics from stale `<base>-listbox` ids to
    `<base>.listbox`.

## Proof

```powershell
cargo fmt --package fret-ui-material3
python -m json.tool tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-select-a11y-parity-bundle.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-select-typeahead-delay.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-select-rich-options-screenshots.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-select-menu-width-floor-screenshots.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-select-menu-positioning-transform-screenshots.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/overlay/ui-gallery-material3-select-overlay-parity-screenshots.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/overlay/ui-gallery-material3-select-dialog-overlay-screenshots.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/overlay/ui-gallery-material3-select-dialog-bottom-collision.json | Out-Null
cargo nextest run -p fret-ui-material3 --lib autocomplete_default_listbox_test_id_uses_dotted_part_contract
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_autocomplete_exposes_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_autocomplete_semantics_v1 material3_exposed_dropdown_trailing_icon_toggles_overlay_v1 material3_exposed_dropdown_reverts_query_to_committed_selection_on_blur_v1
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_select_exposes_stable_part_test_ids
```

Results:

- `autocomplete_default_listbox_test_id_uses_dotted_part_contract`: passed, run id
  `631d55bb-3bb7-425e-b1b1-38b7eb18bc03`.
- `material3_autocomplete_exposes_stable_part_test_ids`: passed, run id
  `e31c5241-7b95-4f75-9cda-287b2ac33911`.
- Autocomplete/ExposedDropdown focused `radio_alignment` tests: 3 passed, run id
  `bce6e3dc-34b6-4854-89fa-b838d5e3ada1`.
- `material3_select_exposes_stable_part_test_ids`: passed, run id
  `f2b91219-144e-49be-b768-be6e0b9044cb`.

## Residual Risk

- This packet closes automation-surface and accessibility wiring for the listbox popup path. It
  does not claim full style, layout, or motion parity for Autocomplete or ExposedDropdown.
- Select visual/layout token parity remains open; this packet only swept live diag selectors after
  the prior Select recipe id change.
