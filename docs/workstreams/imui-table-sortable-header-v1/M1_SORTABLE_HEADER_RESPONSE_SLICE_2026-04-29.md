# ImUi Table Sortable Header v1 - M1 Sortable Header Response Slice - 2026-04-29

Status: complete

## Summary

This slice adds a policy-layer sortable header trigger surface to IMUI tables. It keeps row sorting
and sort-state transitions app-owned: `fret-ui-kit::imui` reports which header was activated, but
does not reorder rows or mutate a table sort model.

## Adopted Surface

```rust
let columns = [
    TableColumn::fill("Name###asset-name")
        .sortable()
        .sorted(TableSortDirection::Ascending),
    TableColumn::px("Status###asset-status", Px(120.0)).sortable(),
];

let response = ui.table("assets", &columns, |table| {
    table.row("asset-a", |row| {
        row.cell_text("Asset A");
        row.cell_text("Ready");
    });
});

if response.header("asset-name").is_some_and(|header| header.clicked()) {
    // Application-owned sort model toggles here.
}
```

## Implementation Notes

- `TableColumn` now carries `sortable` and `sort_direction` metadata.
- `TableSortDirection::{Ascending, Descending}` describes current visual state only.
- `ui.table(...)` and `ui.table_with_options(...)` return `TableResponse`.
- Sortable headers render as pressable header cells and reuse the identity-derived header
  `test_id`.
- Current sort direction renders as a compact ASCII indicator (`^` / `v`).
- The existing table layout composition gate now targets identity-derived `status` / `owner`
  selectors instead of stale column-index selectors.

## Gate Evidence

- `cargo check -p fret-ui-kit --features imui --jobs 1`
- `cargo nextest run -p fret-ui-kit --features imui --test imui_table_smoke table_sortable_header_api_compiles --no-fail-fast`
- `cargo nextest run -p fret-imui table_sortable_header --no-fail-fast`
- `cargo nextest run -p fret-imui table_helper_keeps_header_and_body_columns_aligned_and_clips_long_cells --no-fail-fast`
