# ImUi Table Sortable Header v1 - Closeout Audit - 2026-04-29

Status: closed

## Verdict

This lane is closed. IMUI tables now have a narrow sortable header response surface that lets an
application toggle its own sort model from stable column identities.

The lane intentionally does not add row sorting, multi-sort policy, resizable column handles,
column sizing persistence, localization policy, or runtime table semantics.

## Shipped Surface

- `TableSortDirection::{Ascending, Descending}`
- `TableColumn::sortable()`
- `TableColumn::sorted(...)`
- `TableColumn::with_sort_direction(...)`
- `TableResponse`
- `TableHeaderResponse`

## Gate Evidence

- `cargo check -p fret-ui-kit --features imui --jobs 1`
- `cargo nextest run -p fret-ui-kit --features imui --test imui_table_smoke table_sortable_header_api_compiles --no-fail-fast`
- `cargo nextest run -p fret-imui table_sortable_header --no-fail-fast`
- `cargo nextest run -p fret-imui table_helper_keeps_header_and_body_columns_aligned_and_clips_long_cells --no-fail-fast`

## Future Work

Start separate follow-ons for any of:

- app recipe or headless row sorting integration,
- multi-sort toggle policy,
- resizable column handles,
- column sizing persistence,
- localization-aware column ids,
- richer runtime table semantics.
