# ImUi Table Column Identity v1 - Milestones

Status: closed

## M1 - Column Identity Slice

Exit criteria:

- `TableColumn` stores a stable `id`.
- Labeled constructors infer `id` from `##` / `###` grammar.
- Unlabeled columns can opt into stable identity via `with_id(...)`.
- Header and body cell default `test_id`s use column identity when available.
- The old index form remains only as the fallback for no-id columns.

Result:

- Complete in `M1_TABLE_COLUMN_IDENTITY_SLICE_2026-04-29.md`.

## M2 - Closeout

Exit criteria:

- The prior table-header label policy lane stays closed.
- Future sortable/resizable, persistence, localization, and runtime diagnostics work is explicitly
  routed to separate follow-ons.

Result:

- Complete in `CLOSEOUT_AUDIT_2026-04-29.md`.
