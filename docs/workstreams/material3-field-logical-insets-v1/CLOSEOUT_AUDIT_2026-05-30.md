# Material3 Field Logical Insets v1 Closeout Audit

Status: Closed
Date: 2026-05-30

## Summary

This follow-on closes the TextField and Select slice of Material3 logical inline field geometry.
The shipped behavior keeps Material direction policy in `ecosystem/fret-ui-material3`, extends the
shared Material logical edge helper, and wires TextField/Select floating label plus supporting text
geometry through the resolved Material layout direction.

## Shipped

- Added inline-start absolute inset plus inline-start/end margin helpers in
  `foundation::logical_edges`.
- Migrated TextField label and supporting text from physical left/right geometry to logical inline
  edge helpers.
- Migrated Select trigger label and supporting text from physical left/right geometry to logical
  inline edge helpers.
- Added `select-trigger.supporting-text` as a stable Select part `test_id` so diagnostics can prove
  supporting text geometry.
- Added RTL geometry tests for TextField and Select with leading icons, where the larger
  inline-start field text inset must appear on the physical right edge.

## Evidence

- `ecosystem/fret-ui-material3/src/foundation/logical_edges.rs`
- `ecosystem/fret-ui-material3/src/text_field.rs`
- `ecosystem/fret-ui-material3/src/select.rs`
- `ecosystem/fret-ui-material3/tests/text_field_hover.rs`
- `ecosystem/fret-ui-material3/tests/select_behavior.rs`

## Gates

- Passed: `cargo fmt -p fret-ui-material3`
- Passed: `cargo nextest run -p fret-ui-material3 --lib foundation::logical_edges`
- Passed: `cargo nextest run -p fret-ui-material3 --features diagnostics --test text_field_hover text_field_rtl_label_and_supporting_text_use_logical_inline_insets`
- Passed: `cargo nextest run -p fret-ui-material3 --features diagnostics --test select_behavior select_rtl_label_and_supporting_text_use_logical_inline_insets`
- Passed: `cargo check -p fret-ui-material3 --features diagnostics --tests`
- Passed: `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`

Final documentation gates are recorded in `EVIDENCE_AND_GATES.md` after the closeout catalog is
updated.

## Layering Verdict

No core runtime contract widened. Direction is read through the Material theme/context bridge and
converted to physical layout edges inside the Material3 recipe/foundation layer.

## Residual Follow-Ons

- Field input-row icon order and trailing affordance hit targets.
- Select popup/listbox RTL placement and option row logical insets.
- Autocomplete and ExposedDropdown adoption of the same field-family logical inset path.
