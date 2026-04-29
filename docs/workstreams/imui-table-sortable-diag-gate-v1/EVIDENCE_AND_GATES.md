# ImUi Table Sortable Diagnostics Gate v1 - Evidence And Gates

Status: active

## Smallest Repro

```text
cargo run -p fretboard -- diag run tools/diag-scripts/ui-editor/imui/imui-shadcn-adapter-sortable-table-gate.json --dir target/fret-diag/imui-table-sortable-diag-gate-v1 --session-auto --timeout-ms 240000 --exit-after-run --launch -- cargo run -p fret-demo --bin imui_shadcn_adapter_demo
```

## Gate Set

```text
python -m json.tool tools/diag-scripts/ui-editor/imui/imui-shadcn-adapter-sortable-table-gate.json
python -m json.tool tools/diag-scripts/suites/imui-table-sortable-diag-gate/suite.json
cargo check -p fret-examples --lib --jobs 1
cargo nextest run -p fret-examples imui_shadcn_adapter_demo_keeps_sortable_table_diag_gate --no-fail-fast
cargo run -p fretboard -- diag run tools/diag-scripts/ui-editor/imui/imui-shadcn-adapter-sortable-table-gate.json --dir target/fret-diag/imui-table-sortable-diag-gate-v1 --session-auto --timeout-ms 240000 --exit-after-run --launch -- cargo run -p fret-demo --bin imui_shadcn_adapter_demo
python tools/check_workstream_catalog.py
git diff --check
```

## Current Evidence

- Passed: script JSON shape.
- Passed: suite JSON shape.
- Passed: `cargo check -p fret-examples --lib --jobs 1`.
- Pending: source-marker nextest after local Rust compile/link timeout clears.
- Pending: launched diagnostics gate after local `fret-demo` build timeout clears.

## Evidence Anchors

- `tools/diag-scripts/ui-editor/imui/imui-shadcn-adapter-sortable-table-gate.json`
- `tools/diag-scripts/suites/imui-table-sortable-diag-gate/suite.json`
- `apps/fret-examples/src/lib.rs`
- `docs/workstreams/imui-table-sortable-demo-proof-v1/CLOSEOUT_AUDIT_2026-04-29.md`
