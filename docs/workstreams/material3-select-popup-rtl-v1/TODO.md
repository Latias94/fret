# Material3 Select Popup RTL v1 TODO

Status: Closed
Last updated: 2026-05-30

## Tasks

- [x] M3SELRTL-010: Wire Select popup placement to the resolved Material layout direction.
  - Gate: `cargo nextest run -p fret-ui-material3 --features diagnostics --test select_behavior select_rtl_start_aligned_popup_anchors_to_trigger_inline_start`.

- [x] M3SELRTL-020: Wire Select listbox option rows to logical leading/trailing visual slots.
  - Gate: `cargo nextest run -p fret-ui-material3 --features diagnostics --test select_behavior select_rtl_listbox_items_place_logical_leading_slot_on_right`.

- [x] M3SELRTL-030: Run quality gates and close the lane.
