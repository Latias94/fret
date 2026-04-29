# ImUi Table Column Identity v1 - TODO

Status: closed

## M0 - Tracking

- [x] Start a narrow follow-on instead of reopening `imui-table-header-label-policy-v1`.
- [x] Keep sortable/resizable column state and localization out of this slice.

## M1 - Implementation

- [x] Add `TableColumn::id` as a policy-layer stable column identity.
- [x] Infer column ids from the existing IMUI label identity parser.
- [x] Add explicit `TableColumn::with_id(...)` for unlabeled/action columns.
- [x] Derive rooted table header/body-cell `test_id`s from column identity.
- [x] Keep index fallback for columns without identity.

## M2 - Closeout

- [x] Add focused API and rendered diagnostics tests.
- [x] Record the shipped scope and future follow-on boundaries.
