# ImUi Table Column Width Diagnostics Gate v1 - Evidence And Gates

Status: closed
Last updated: 2026-05-01

## Smallest Repro

```text
cargo run -p fretboard -- diag run tools/diag-scripts/ui-editor/imui/imui-shadcn-adapter-table-column-width-resize.json --dir target/fret-diag/imui-table-column-width-diag-gate-v1 --session-auto --timeout-ms 300000 --exit-after-run --launch -- cargo run -p fret-demo --bin imui_shadcn_adapter_demo
```

## Gate Set

```text
python -m json.tool tools/diag-scripts/ui-editor/imui/imui-shadcn-adapter-table-column-width-resize.json
python -m json.tool tools/diag-scripts/suites/imui-table-column-width-diag-gate/suite.json
python tools/gate_imui_shadcn_adapter_table_column_width_diag_source.py
cargo run -p fretboard -- diag run tools/diag-scripts/ui-editor/imui/imui-shadcn-adapter-table-column-width-resize.json --dir target/fret-diag/imui-table-column-width-diag-gate-v1 --session-auto --timeout-ms 300000 --exit-after-run --launch -- cargo run -p fret-demo --bin imui_shadcn_adapter_demo
python tools/check_workstream_catalog.py
git diff --check
```

## Current Evidence

- Passed: script JSON shape.
- Passed: suite JSON shape.
- Passed: source-marker gate.
- Passed: launched diagnostics gate.
- Authoring correction: the width summary text is exposed as a semantics label, so the script uses
  `label_contains` rather than `value_contains`.

## Runtime Evidence

- Run ID: `1777634686501`.
- Session: `target/fret-diag/imui-table-column-width-diag-gate-v1/sessions/1777634437651-87860`.
- Last artifact: `1777634694683-imui-shadcn-adapter.table-column-width.after`.
- Script result stage: `passed`.
- Selector evidence: `imui-shadcn-demo.inspector.table.header.cell.inspector-field.resize`
  resolved to one resize handle; after `drag_pointer_until`, the width summary label was
  `Widths: field 180px, value 120px, source 72px`.

## Evidence Anchors

- `tools/diag-scripts/ui-editor/imui/imui-shadcn-adapter-table-column-width-resize.json`
- `tools/diag-scripts/suites/imui-table-column-width-diag-gate/suite.json`
- `tools/gate_imui_shadcn_adapter_table_column_width_diag_source.py`
- `apps/fret-examples-imui/src/imui_shadcn_adapter_demo.rs`
- `docs/workstreams/imui-table-column-width-demo-proof-v1/CLOSEOUT_AUDIT_2026-05-01.md`
