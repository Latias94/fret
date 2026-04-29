# ImUi Table Sortable Diagnostics Gate v1 - Evidence And Gates

Status: closed

## Smallest Repro

```text
cargo run -p fretboard -- diag run tools/diag-scripts/ui-editor/imui/imui-shadcn-adapter-sortable-table-gate.json --dir target/fret-diag/imui-table-sortable-diag-gate-v1 --session-auto --timeout-ms 300000 --exit-after-run --launch -- cargo run -p fret-demo --bin imui_shadcn_adapter_demo
```

## Gate Set

```text
python -m json.tool tools/diag-scripts/ui-editor/imui/imui-shadcn-adapter-sortable-table-gate.json
python -m json.tool tools/diag-scripts/suites/imui-table-sortable-diag-gate/suite.json
python tools/gate_imui_shadcn_adapter_sortable_table_source.py
cargo check -p fret-examples --lib --jobs 1
cargo run -p fretboard -- diag run tools/diag-scripts/ui-editor/imui/imui-shadcn-adapter-sortable-table-gate.json --dir target/fret-diag/imui-table-sortable-diag-gate-v1 --session-auto --timeout-ms 300000 --exit-after-run --launch -- cargo run -p fret-demo --bin imui_shadcn_adapter_demo
python tools/check_workstream_catalog.py
git diff --check
```

## Current Evidence

- Passed: script JSON shape.
- Passed: suite JSON shape.
- Passed: lightweight sortable source-marker gate.
- Passed: `cargo check -p fret-examples --lib --jobs 1`.
- Historical closeout evidence: `cargo nextest run -p fret-examples imui_shadcn_adapter_demo_keeps_sortable_table_diag_gate --no-fail-fast` passed before the pure source-marker check moved to `tools/gate_imui_shadcn_adapter_sortable_table_source.py` under `docs/workstreams/fret-examples-build-latency-v1/`.
- Passed: launched diagnostics gate.

## Runtime Evidence

- Run ID: `1777454218396`.
- Session: `target/fret-diag/imui-table-sortable-diag-gate-v1/sessions/1777453912985-52424`.
- Last artifact: `1777454223920-imui-shadcn-adapter.sortable-table.after`.
- Script result stage: `passed`.
- Selector evidence: `imui-shadcn-demo.inspector.table.header.cell.inspector-field` resolved to
  one button named `Field, sorted ascending`; after `click_stable`, the script observed
  `sorted descending`.

## Evidence Anchors

- `tools/diag-scripts/ui-editor/imui/imui-shadcn-adapter-sortable-table-gate.json`
- `tools/diag-scripts/suites/imui-table-sortable-diag-gate/suite.json`
- `tools/gate_imui_shadcn_adapter_sortable_table_source.py`
- `apps/fret-examples/src/lib.rs`
- `docs/workstreams/imui-table-sortable-demo-proof-v1/CLOSEOUT_AUDIT_2026-04-29.md`
