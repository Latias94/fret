# IMUI Table Body Owner Split v1 - Evidence & Gates

Status: Closed
Last updated: 2026-05-25

## Evidence Anchors

- Workstream:
  - `docs/workstreams/imui-table-body-owner-split-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-table-body-owner-split-v1/DESIGN.md`
  - `docs/workstreams/imui-table-body-owner-split-v1/TODO.md`
  - `docs/workstreams/imui-table-body-owner-split-v1/MILESTONES.md`
  - `docs/workstreams/imui-table-body-owner-split-v1/EVIDENCE_AND_GATES.md`
- Implementation:
  - `ecosystem/fret-ui-kit/src/imui/table_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/table_controls/body.rs`
  - `ecosystem/fret-ui-kit/src/imui/table_controls/header.rs`
  - `tools/gate_imui_workstream_source.py`
- Prior table behavior lanes:
  - `docs/workstreams/imui-table-header-owner-split-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-table-sortable-header-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-table-column-resize-v1/WORKSTREAM.json`

## Repro

```powershell
cargo nextest run -p fret-imui table_sortable_header_reports_app_owned_trigger_without_sorting_rows table_resizable_header_reports_drag_response table_plain_header_left_click_does_not_activate_or_click --no-fail-fast
```

## Focused Gates

```powershell
cargo fmt --check -p fret-ui-kit
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui --test imui_table_smoke table_sortable_header_api_compiles table_resizable_column_api_compiles --no-fail-fast
cargo nextest run -p fret-imui table_sortable_header_reports_app_owned_trigger_without_sorting_rows table_resizable_header_reports_drag_response table_plain_header_left_click_does_not_activate_or_click --no-fail-fast
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-table-body-owner-split-v1/WORKSTREAM.json
git diff --check
```

## 2026-05-25 Slice Results

- PASS: `cargo fmt -p fret-ui-kit`
- PASS: `cargo fmt --check -p fret-ui-kit`
- PASS: `cargo check -p fret-ui-kit --features imui`
  - Existing warnings only from `crates/fret-ui`: `unexpected cfg` for
    `unstable-retained-bridge` and unused `current_effective_opacity`.
- PASS: `cargo nextest run -p fret-ui-kit --features imui --test imui_table_smoke table_sortable_header_api_compiles table_resizable_column_api_compiles --no-fail-fast`
  - 2 passed, 7 skipped.
- PASS: `cargo nextest run -p fret-imui table_sortable_header_reports_app_owned_trigger_without_sorting_rows table_resizable_header_reports_drag_response table_plain_header_left_click_does_not_activate_or_click --no-fail-fast`
  - 3 passed, 176 skipped.
- PASS: `python tools/gate_imui_workstream_source.py`
- PASS: `python tools/check_workstream_catalog.py`
  - Validated 441 dedicated directories and 47 standalone markdown files.
- PASS: `python -m json.tool docs/workstreams/imui-table-body-owner-split-v1/WORKSTREAM.json`
- PASS_WITH_WARNINGS: `git diff --check`
  - No whitespace errors.
  - Existing line-ending warnings remain for `Cargo.lock` and `apps/fret-examples/src/lib.rs`.
