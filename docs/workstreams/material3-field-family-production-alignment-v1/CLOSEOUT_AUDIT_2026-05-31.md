# Material3 Field Family Production Alignment v1 Closeout Audit

Status: Closed
Date: 2026-05-31

## Summary

This fearless-refactor lane audited the Material3 field family and landed the smallest production
alignment fix that was clearly shared foundation work: field icon slots now respect logical
inline edges. Autocomplete and ExposedDropdown benefit through TextField composition, while Select
keeps its separate non-editable combobox trigger policy.

## Source-Backed Conclusions

- Compose Material3 uses shared text field decoration/chrome for editable fields and exposed
  dropdown anchoring, so Fret's `TextField` composition for Autocomplete/ExposedDropdown is the
  right direction.
- Base UI keeps Combobox input semantics and Select trigger/listbox semantics separate. Fret should
  preserve that split: Autocomplete uses input focus plus active descendant, Select uses trigger
  focus plus a roving listbox.
- The field foundation gap was not another component rewrite. It was logical icon-slot geometry:
  label/supporting text had moved to logical insets, but TextField icons and padding still used
  physical edges.

## Shipped

- Added `material_field_icon_adjusted_padding` in `foundation::field`.
- Migrated TextField icon padding and absolute icon hit-target insets to logical inline
  start/end.
- Added Select trigger `m3-select.leading-icon` part `test_id`.
- Applied resolved Material layout direction to Select trigger row/text layout.
- Added regression coverage:
  - `text_field_rtl_icon_slots_use_logical_inline_edges`
  - `material3_select_rtl_trigger_icons_use_logical_inline_edges`
  - `material3_select_exposes_stable_part_test_ids` now covers trigger leading icon.

## Evidence

- `ecosystem/fret-ui-material3/src/foundation/field.rs`
- `ecosystem/fret-ui-material3/src/text_field.rs`
- `ecosystem/fret-ui-material3/src/select.rs`
- `ecosystem/fret-ui-material3/tests/text_field_hover.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`

## Gates

- Passed: `cargo fmt --package fret-ui-material3 --check`
- Passed: `cargo nextest run -p fret-ui-material3 --features diagnostics --test text_field_hover --test automation_surface`
- Passed: `cargo check -p fret-ui-material3 --features diagnostics --tests`
- Passed: `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
- Passed: `python -m json.tool docs/workstreams/material3-field-family-production-alignment-v1/WORKSTREAM.json | Out-Null`
- Passed: `python tools/check_workstream_catalog.py`
- Passed: `python tools/check_layering.py`
- Passed: `git diff --check`

## Residual Follow-Ons

- Multiline field icon padding may still need an API-level answer because `TextAreaStyle` stores
  symmetric `padding_x`.
- Autocomplete and Select menu rows share token fallback, but a shared renderer should wait until
  duplicated behavior grows beyond simple list item geometry.
- Optional TextField prop forwarding can become public builder surface later; this lane did not
  widen that API.
