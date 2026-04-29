# ImUi Table Column Identity v1 - Closeout Audit - 2026-04-29

Status: closed

## Verdict

This lane is closed. IMUI table columns now have a narrow stable identity surface that feeds default
header and body-cell diagnostics `test_id`s when a table root `test_id` is present.

The slice intentionally does not add sortable/resizable table state, column sizing persistence,
runtime ID-stack diagnostics, or localization policy.

## Adopted Surface

```rust
TableColumn::fill("Name##asset-name-column")
TableColumn::px("Status###status-column", Px(120.0))
TableColumn::unlabeled(TableColumnWidth::px(Px(64.0))).with_id("row-actions")
```

For identity-bearing columns, default table diagnostics now use identity-derived suffixes:

```text
<table>.header.cell.<column-id-slug>
<table>.row.<row-index>.cell.<column-id-slug>
```

Columns without identity continue to use index fallback.

## Gate Evidence

- `cargo nextest run -p fret-ui-kit --features imui --test imui_table_smoke table_column_helpers_compile --no-fail-fast`
- `cargo nextest run -p fret-imui label_identity_table_headers_hide_suffixes_from_visible_labels --no-fail-fast`
- `cargo fmt --package fret-ui-kit --package fret-imui --check`

## Future Work

Start a separate narrow follow-on for any of:

- sortable/resizable column state,
- column sizing persistence,
- localization-aware column ids,
- table column runtime ID-stack diagnostics,
- public table column state APIs.
