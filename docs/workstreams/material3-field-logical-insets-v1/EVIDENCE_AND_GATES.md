# Material3 Field Logical Insets v1 Evidence And Gates

Status: Closed
Last updated: 2026-05-30

## Repro

- `cargo nextest run -p fret-ui-material3 --lib foundation::logical_edges`
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test text_field_hover text_field_rtl_label_and_supporting_text_use_logical_inline_insets`
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test select_behavior select_rtl_label_and_supporting_text_use_logical_inline_insets`

## Gates

- Passed: `cargo fmt -p fret-ui-material3`
- Passed: `cargo nextest run -p fret-ui-material3 --lib foundation::logical_edges`
- Passed: `cargo nextest run -p fret-ui-material3 --features diagnostics --test text_field_hover text_field_rtl_label_and_supporting_text_use_logical_inline_insets`
- Passed: `cargo nextest run -p fret-ui-material3 --features diagnostics --test select_behavior select_rtl_label_and_supporting_text_use_logical_inline_insets`
- Passed: `cargo check -p fret-ui-material3 --features diagnostics --tests`
- Passed: `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
- Passed: `python -m json.tool docs/workstreams/material3-field-logical-insets-v1/WORKSTREAM.json | Out-Null`
- Passed: `python tools/check_workstream_catalog.py`
- Passed: `python tools/check_layering.py`
- Passed: `git diff --check`

## Evidence Anchors

- `ecosystem/fret-ui-material3/src/foundation/logical_edges.rs`
- `ecosystem/fret-ui-material3/src/text_field.rs`
- `ecosystem/fret-ui-material3/src/select.rs`
- `ecosystem/fret-ui-material3/tests/text_field_hover.rs`
- `ecosystem/fret-ui-material3/tests/select_behavior.rs`
- `docs/workstreams/material3-field-logical-insets-v1/CLOSEOUT_AUDIT_2026-05-30.md`
