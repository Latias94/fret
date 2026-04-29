# ImUi Table Sortable Header v1 - TODO

Status: closed

## M0 - Tracking

- [x] Start a narrow follow-on instead of reopening table header label or column identity lanes.
- [x] Keep row sorting, multi-sort, resize, persistence, and localization out of this slice.

## M1 - Implementation

- [x] Add `TableSortDirection`.
- [x] Add sortable/current-sort builder surface to `TableColumn`.
- [x] Return `TableResponse` from `table(...)` / `table_with_options(...)`.
- [x] Render sortable headers as pressable header cells.
- [x] Preserve identity-derived `test_id`s and suffix-free visible labels.

## M2 - Closeout

- [x] Add focused API and rendered interaction tests.
- [x] Record shipped scope and future follow-on boundaries.
