# ImUi Table Sortable Diagnostics Gate v1 - M1 Script Scaffold - 2026-04-29

Status: active

## Landed

- Added a schema v2 diagnostics script for the sortable inspector table proof.
- Added a suite manifest for the script.
- Added source-marker coverage tying the script to the demo selectors and app-owned sort markers.

## Validation

Passed:

```text
python -m json.tool tools/diag-scripts/ui-editor/imui/imui-shadcn-adapter-sortable-table-gate.json
python -m json.tool tools/diag-scripts/suites/imui-table-sortable-diag-gate/suite.json
cargo check -p fret-examples --lib --jobs 1
```

Attempted but not counted as passed:

```text
cargo nextest run -p fret-examples imui_shadcn_adapter_demo_keeps_sortable_table_diag_gate --no-fail-fast
cargo run -p fretboard -- diag run tools/diag-scripts/ui-editor/imui/imui-shadcn-adapter-sortable-table-gate.json --dir target/fret-diag/imui-table-sortable-diag-gate-v1-local --session-auto --timeout-ms 240000 --exit-after-run --launch -- cargo run -p fret-demo --bin imui_shadcn_adapter_demo
```

Both local attempts timed out while compiling/linking `fret-examples` or `fret-demo`; no script
assertion failure was observed. Keep this lane active until the launched diagnostics command passes.
