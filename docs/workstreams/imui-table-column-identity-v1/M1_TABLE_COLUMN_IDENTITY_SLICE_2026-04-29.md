# M1 Table Column Identity Slice - 2026-04-29

Status: accepted

## Landed

- `TableColumn` now carries a stable `id`.
- `TableColumn::fill`, `TableColumn::weighted`, and `TableColumn::px` infer that id from the same
  IMUI label identity grammar used by other label-bearing controls:
  - `Name` -> `Name`
  - `Name##asset-name-column` -> `Name##asset-name-column`
  - `Status###status-column` -> `status-column`
- `TableColumn::unlabeled(...).with_id("row-actions")` gives action columns a stable identity
  without inventing a painted header.
- Rooted table semantics now derive header and body-cell `test_id`s from a column identity slug:
  - `<table>.header.cell.status-column`
  - `<table>.row.0.cell.status-column`
- Columns with no identity keep the previous index fallback.

## Proof

`ecosystem/fret-ui-kit/tests/imui_table_smoke.rs` proves the constructor/id surface, including
`##`, `###`, and explicit ids for unlabeled columns.

`ecosystem/fret-imui/src/tests/label_identity.rs` proves the rendered table surface exposes stable
header/body-cell `test_id`s and no longer exposes the old index-based header test id for the
covered identity-bearing columns.

## Gates

```text
cargo nextest run -p fret-ui-kit --features imui --test imui_table_smoke table_column_helpers_compile --no-fail-fast
cargo nextest run -p fret-imui label_identity_table_headers_hide_suffixes_from_visible_labels --no-fail-fast
cargo fmt --package fret-ui-kit --package fret-imui --check
```

## Deferred

- sortable/resizable column state
- column sizing persistence
- localization policy for labels that contain `##` / `###`
- runtime ID-stack diagnostics
