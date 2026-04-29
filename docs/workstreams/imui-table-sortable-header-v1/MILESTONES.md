# ImUi Table Sortable Header v1 - Milestones

Status: closed

## M1 - Sortable Header Trigger Slice

Exit criteria:

- `TableColumn` can declare a sortable header and current sort direction.
- `ui.table(...)` and `ui.table_with_options(...)` return `TableResponse`.
- The response includes stable per-header column identity, index, sort direction, and trigger
  response.
- Header clicks report through `ResponseExt` without changing row ordering inside IMUI.
- Current sort direction renders as a compact indicator without painting label identity suffixes.

Result:

- Complete in `M1_SORTABLE_HEADER_RESPONSE_SLICE_2026-04-29.md`.

## M2 - Closeout

Exit criteria:

- The lane is explicitly closed after the trigger/response slice.
- Future row sorting engines, multi-sort rules, resize, persistence, localization, and runtime
  semantics are routed to separate follow-ons.

Result:

- Complete in `CLOSEOUT_AUDIT_2026-04-29.md`.
