# Material3 Select Popup RTL v1 Closeout Audit

Status: Closed
Date: 2026-05-30

## Summary

This follow-on closes the Material3 Select popup/listbox RTL slice. Select popup placement now uses
the resolved Material layout direction, and listbox option rows apply that direction directly to the
actual flex/text elements that are built inside delayed child closures.

## Shipped

- Replaced Select popup `use_direction_in_scope` fallback with the resolved Material layout
  direction, so `SelectMenuAlign::Start` is logical rather than physical.
- Passed the resolved direction into Select listbox rendering and option row construction.
- Added explicit `with_layout_direction(layout_direction)` calls on listbox row/text elements
  created inside delayed child builders.
- Added an icon slot container for menu item icons so stable item part ids describe the same node
  that participates in row layout.
- Added focused RTL geometry tests for popup start alignment and listbox leading/trailing icon
  placement.

## Evidence

- `ecosystem/fret-ui-material3/src/select.rs`
- `ecosystem/fret-ui-material3/tests/select_behavior.rs`

## Gates

- Passed: `cargo fmt -p fret-ui-material3`
- Passed: `cargo nextest run -p fret-ui-material3 --features diagnostics --test select_behavior select_rtl_start_aligned_popup_anchors_to_trigger_inline_start`
- Passed: `cargo nextest run -p fret-ui-material3 --features diagnostics --test select_behavior select_rtl_listbox_items_place_logical_leading_slot_on_right`
- Passed: `cargo nextest run -p fret-ui-material3 --features diagnostics --test select_behavior select_rtl_label_and_supporting_text_use_logical_inline_insets`
- Passed: `cargo nextest run -p fret-ui-material3 --lib select::item_text_tests`
- Passed: `cargo check -p fret-ui-material3 --features diagnostics --tests`
- Passed: `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`

Final documentation gates are recorded in `EVIDENCE_AND_GATES.md` after the closeout catalog is
updated.

## Important Finding

`ElementContext::provide(LayoutDirection, ...)` captures direction for elements built while the
provider is active, but it does not automatically cover child builder closures that execute later.
For listbox rows built through nested closures, the direction must be applied to the actual
`AnyElement` via `with_layout_direction(layout_direction)` or provided inside the closure body that
constructs that element.

## Layering Verdict

No core or shadcn surface widened. The fix stays inside the Material3 recipe and uses existing
`LayoutDirection`/popper mechanisms.

## Residual Follow-Ons

- Select trigger input-row leading/trailing icon visual order.
- ExposedDropdown and Autocomplete popup/listbox RTL adoption.
- A neutral `fret-ui-kit` logical edge helper only if shadcn audits show duplicated need.
