# ImUi Table Sortable Diagnostics Gate v1 - Closeout Audit - 2026-04-29

Status: closed

## Verdict

This lane is closed. The app-owned sortable inspector table proof now has a promoted diagnostics
script, a suite manifest, source-marker coverage, and a passing launched run.

## Shipped Surface

- `tools/diag-scripts/ui-editor/imui/imui-shadcn-adapter-sortable-table-gate.json`
- `tools/diag-scripts/suites/imui-table-sortable-diag-gate/suite.json`
- Source-marker coverage in `tools/gate_imui_shadcn_adapter_sortable_table_source.py` after the
  build-latency follow-on moved the pure text check out of `apps/fret-examples/src/lib.rs`.

## Gate Evidence

- `cargo check -p fret-examples --lib --jobs 1`
- `python tools/gate_imui_shadcn_adapter_sortable_table_source.py`
- `cargo build -p fret-demo --bin imui_shadcn_adapter_demo --jobs 1`
- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-editor/imui/imui-shadcn-adapter-sortable-table-gate.json --dir target/fret-diag/imui-table-sortable-diag-gate-v1 --session-auto --timeout-ms 300000 --exit-after-run --launch -- cargo run -p fret-demo --bin imui_shadcn_adapter_demo`

Historical note: the Rust nextest source-marker gate passed during closeout. The current source
marker lives in the Python gate above so it does not compile the monolithic examples crate for a
pure source scan.

Runtime evidence:

- Run ID: `1777454218396`.
- Session: `target/fret-diag/imui-table-sortable-diag-gate-v1/sessions/1777453912985-52424`.
- Last artifact: `1777454223920-imui-shadcn-adapter.sortable-table.after`.
- Script result stage: `passed`.

## Future Work

Start separate follow-ons for row sorting engines, multi-sort policy, column resize handles, column
sizing persistence, or cookbook prose. Do not reopen this diagnostics gate lane for those scopes.
