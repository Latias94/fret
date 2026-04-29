# ImUi Table Column Identity v1 - Evidence And Gates

Status: closed

## Smallest Repro

Use a rooted IMUI table with three column identity shapes:

- `Name##asset-name-column`
- `Status###status-column`
- an unlabeled action column with `.with_id("row-actions")`

The table should expose stable header and row-cell `test_id`s using the column identity slug rather
than the column index.

## Gate Set

```text
cargo nextest run -p fret-ui-kit --features imui --test imui_table_smoke table_column_helpers_compile --no-fail-fast
cargo nextest run -p fret-imui label_identity_table_headers_hide_suffixes_from_visible_labels --no-fail-fast
cargo fmt --package fret-ui-kit --package fret-imui --check
python -m json.tool docs/workstreams/imui-table-column-identity-v1/WORKSTREAM.json
python tools/check_workstream_catalog.py
git diff --check
```

## Evidence Anchors

- `ecosystem/fret-ui-kit/src/imui/options/collections.rs`
- `ecosystem/fret-ui-kit/src/imui/table_controls.rs`
- `ecosystem/fret-ui-kit/tests/imui_table_smoke.rs`
- `ecosystem/fret-imui/src/tests/label_identity.rs`
- `docs/workstreams/imui-table-header-label-policy-v1/CLOSEOUT_AUDIT_2026-04-28.md`
- `docs/workstreams/imui-label-identity-ergonomics-v1/CLOSEOUT_AUDIT_2026-04-28.md`
