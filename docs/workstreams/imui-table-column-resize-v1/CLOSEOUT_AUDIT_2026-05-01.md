# ImUi Table Column Resize v1 - Closeout Audit

Status: closed
Date: 2026-05-01

## Verdict

The bounded IMUI table column resize response surface landed and this lane is closed.

The shipped surface lets a `TableColumn` opt into a resizable header boundary, exposes the
per-column resize drag response through `TableHeaderResponse`, and keeps width state, persistence,
row sorting, multi-sort policy, declarative/headless table sizing, and runtime table semantics out
of `fret-ui-kit::imui`.

## Shipped Evidence

- `TableColumn::resizable()` and `TableColumn::resizable_with_limits(...)` record opt-in resize
  metadata without adding helper-owned width state.
- `TableHeaderResponse::resize` reports the stable column id/index, width limits, drag lifecycle,
  drag delta, and total drag offset.
- Header resize handles render with stable diagnostics ids:
  `<table>.header.cell.<column>.resize`.
- The handle has a stable non-zero hit region, so layout does not collapse the affordance to a
  zero-height pointer target in auto-sized header rows.
- Existing sortable header trigger reporting remains app-owned and unchanged.

## Gates

```text
cargo nextest run -p fret-ui-kit --features imui --test imui_table_smoke table_resizable_column_api_compiles --no-fail-fast
cargo nextest run -p fret-imui table_resizable_header_reports_drag_response --no-fail-fast
cargo nextest run -p fret-imui table_sortable_header_reports_app_owned_trigger_without_sorting_rows --no-fail-fast
cargo check -p fret-ui-kit --features imui --jobs 1
cargo fmt --package fret-ui-kit --package fret-imui --check
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-table-column-resize-v1/WORKSTREAM.json
git diff --check
```

All gates passed on 2026-05-01.

## Follow-On Boundary

Do not reopen this lane for larger table policy. Start a narrower follow-on for any of:

- app-owned width persistence and layout save/load,
- declarative table/headless sizing interop examples,
- grouped header or multi-column resize policy,
- localization-aware column ids,
- runtime table semantics or ID-stack changes.
