# Material3 Select Popup RTL v1 Evidence And Gates

Status: Closed
Last updated: 2026-05-30

## Repro

- `cargo nextest run -p fret-ui-material3 --features diagnostics --test select_behavior select_rtl_start_aligned_popup_anchors_to_trigger_inline_start`
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test select_behavior select_rtl_listbox_items_place_logical_leading_slot_on_right`

## Gates

- Passed: `cargo fmt -p fret-ui-material3`
- Passed: `cargo nextest run -p fret-ui-material3 --features diagnostics --test select_behavior select_rtl_start_aligned_popup_anchors_to_trigger_inline_start`
- Passed: `cargo nextest run -p fret-ui-material3 --features diagnostics --test select_behavior select_rtl_listbox_items_place_logical_leading_slot_on_right`
- Passed: `cargo nextest run -p fret-ui-material3 --features diagnostics --test select_behavior select_rtl_label_and_supporting_text_use_logical_inline_insets`
- Passed: `cargo nextest run -p fret-ui-material3 --lib select::item_text_tests`
- Passed: `cargo check -p fret-ui-material3 --features diagnostics --tests`
- Passed: `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
- Passed: `python -m json.tool docs/workstreams/material3-select-popup-rtl-v1/WORKSTREAM.json | Out-Null`
- Passed: `python tools/check_workstream_catalog.py`
- Passed: `python tools/check_layering.py`
- Passed: `git diff --check`

## Evidence Anchors

- `ecosystem/fret-ui-material3/src/select.rs`
- `ecosystem/fret-ui-material3/tests/select_behavior.rs`
- `docs/workstreams/material3-select-popup-rtl-v1/CLOSEOUT_AUDIT_2026-05-30.md`
