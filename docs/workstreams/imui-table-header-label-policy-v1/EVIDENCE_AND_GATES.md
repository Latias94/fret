# ImUi Table Header Label Policy v1 - Evidence and Gates

Status: closed
Last updated: 2026-04-28

## Smallest Repro

- `cargo nextest run -p fret-imui label_identity --no-fail-fast`

## Current Evidence

- `docs/workstreams/imui-label-identity-ergonomics-v1/CLOSEOUT_AUDIT_2026-04-28.md`
- `docs/workstreams/imui-table-header-label-policy-v1/DESIGN.md`
- `docs/workstreams/imui-table-header-label-policy-v1/M1_TABLE_HEADER_VISIBLE_LABEL_SLICE_2026-04-28.md`
- `docs/workstreams/imui-table-header-label-policy-v1/CLOSEOUT_AUDIT_2026-04-28.md`
- `ecosystem/fret-ui-kit/src/imui/table_controls.rs`
- `ecosystem/fret-imui/src/tests/label_identity.rs`

## Gate Set

- `cargo nextest run -p fret-imui label_identity --no-fail-fast`
- `cargo check -p fret-ui-kit --features imui --jobs 1`
- `cargo fmt --package fret-ui-kit --package fret-imui --check`
- `python tools/check_workstream_catalog.py`
- `python -m json.tool docs/workstreams/imui-table-header-label-policy-v1/WORKSTREAM.json`
- `git diff --check`

## Non-Gates

- No sortable/resizable column identity.
- No runtime ID-stack diagnostics.
- No `test_id` inference from column labels.
