# ImUi Table Column Resize v1 - Evidence And Gates

Status: closed
Last updated: 2026-05-01

## Primary Repro

Render a rooted IMUI table with a stable resizable column, drag the header boundary handle, and read
the returned resize drag response from `TableHeaderResponse`.

## Focused Gates

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

## Evidence Anchors

- `ecosystem/fret-ui-kit/src/imui/options/collections.rs`
- `ecosystem/fret-ui-kit/src/imui/table_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/response/widgets.rs`
- `ecosystem/fret-ui-kit/tests/imui_table_smoke.rs`
- `ecosystem/fret-imui/src/tests/label_identity.rs`
- `docs/workstreams/imui-table-column-resize-v1/CLOSEOUT_AUDIT_2026-05-01.md`

## Verified On 2026-05-01

All focused gates listed above passed on the main workspace after the resize handle hit region was
made non-zero height and the interaction test was updated to assert routing through the handle.

## Deferred Evidence

Do not use this lane to prove persistence, saved layouts, grouped headers, or declarative
table/headless sizing interop. Those need separate proof surfaces.
