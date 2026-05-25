# IMUI Table Header Owner Split v1 - Evidence & Gates

Status: Closed
Last updated: 2026-05-25

## Evidence Anchors

- Workstream:
  - `docs/workstreams/imui-table-header-owner-split-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-table-header-owner-split-v1/DESIGN.md`
  - `docs/workstreams/imui-table-header-owner-split-v1/TODO.md`
  - `docs/workstreams/imui-table-header-owner-split-v1/MILESTONES.md`
  - `docs/workstreams/imui-table-header-owner-split-v1/EVIDENCE_AND_GATES.md`
- Implementation:
  - `ecosystem/fret-ui-kit/src/imui/table_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/table_controls/header.rs`
  - `tools/gate_imui_workstream_source.py`
- Prior table behavior lanes:
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
python -m json.tool docs/workstreams/imui-table-header-owner-split-v1/WORKSTREAM.json
git diff --check
```

## 2026-05-25 Slice Results

- `cargo fmt --check -p fret-ui-kit` passed after targeted `cargo fmt -p fret-ui-kit`.
- `cargo check -p fret-ui-kit --features imui` passed and proved the private header owner module
  compiles behind the existing IMUI feature.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_table_smoke table_sortable_header_api_compiles table_resizable_column_api_compiles --no-fail-fast`
  passed with 2 focused tests.
- `cargo nextest run -p fret-imui table_sortable_header_reports_app_owned_trigger_without_sorting_rows table_resizable_header_reports_drag_response table_plain_header_left_click_does_not_activate_or_click --no-fail-fast`
  passed with 3 focused interaction tests.
- `python tools/gate_imui_workstream_source.py` passed and now freezes the private owner split:
  `table_controls.rs` delegates header behavior to `table_controls/header.rs`, while the header
  owner carries trigger, sort, label, and resize implementation.
- `python tools/check_workstream_catalog.py` passed and validated 440 dedicated directories plus 47
  standalone markdown files.
- `python -m json.tool docs/workstreams/imui-table-header-owner-split-v1/WORKSTREAM.json` passed.
- `git diff --check` passed with Git CRLF/LF working-copy warnings for `Cargo.lock` and
  `apps/fret-examples/src/lib.rs`, but no whitespace errors.

Notes:

- Compile/test gates emitted existing `crates/fret-ui` warnings for `unexpected cfg:
  unstable-retained-bridge` and `current_effective_opacity`; they are outside this slice.
