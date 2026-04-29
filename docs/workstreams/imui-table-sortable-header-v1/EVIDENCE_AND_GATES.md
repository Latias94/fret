# ImUi Table Sortable Header v1 - Evidence And Gates

Status: closed

## Smallest Repro

Render a rooted IMUI table with:

- a sortable `Name###asset-name` header marked as currently ascending,
- a sortable unsorted `Status###asset-status` header,
- at least one row,
- a root `TableOptions::test_id`.

Click the name header and confirm `TableResponse::header("asset-name")` reports a clicked trigger.
The rendered header should expose the stable header `test_id` and should not paint `###asset-name`.

## Gate Set

```text
cargo nextest run -p fret-ui-kit --features imui --test imui_table_smoke table_sortable_header_api_compiles --no-fail-fast
cargo nextest run -p fret-imui table_sortable_header --no-fail-fast
cargo nextest run -p fret-imui table_helper_keeps_header_and_body_columns_aligned_and_clips_long_cells --no-fail-fast
cargo check -p fret-ui-kit --features imui --jobs 1
cargo fmt --package fret-ui-kit --package fret-imui --check
python -m json.tool docs/workstreams/imui-table-sortable-header-v1/WORKSTREAM.json
python tools/check_workstream_catalog.py
git diff --check
```

## Evidence Anchors

- `ecosystem/fret-ui-kit/src/imui/options/collections.rs`
- `ecosystem/fret-ui-kit/src/imui/table_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`
- `ecosystem/fret-ui-kit/src/imui/response/widgets.rs`
- `ecosystem/fret-ui-kit/tests/imui_table_smoke.rs`
- `ecosystem/fret-imui/src/tests/label_identity.rs`
- `ecosystem/fret-imui/src/tests/composition.rs`
- `docs/workstreams/imui-table-column-identity-v1/CLOSEOUT_AUDIT_2026-04-29.md`
