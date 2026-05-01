# ImUi Table Column Width Diagnostics Gate v1 - M2 Launched Diagnostics Gate - 2026-05-01

Status: landed

## Gate

```text
cargo run -p fretboard -- diag run tools/diag-scripts/ui-editor/imui/imui-shadcn-adapter-table-column-width-resize.json --dir target/fret-diag/imui-table-column-width-diag-gate-v1 --session-auto --timeout-ms 300000 --exit-after-run --launch -- cargo run -p fret-demo --bin imui_shadcn_adapter_demo
```

## Result

- Passed.
- Run ID: `1777634686501`.
- Session: `target/fret-diag/imui-table-column-width-diag-gate-v1/sessions/1777634437651-87860`.
- Last artifact: `1777634694683-imui-shadcn-adapter.table-column-width.after`.

## Evidence Read

Bounded `diag slice` evidence for `imui-shadcn-demo.inspector.widths` reported:

```text
Widths: field 180px, value 120px, source 72px
```

The resize handle selector
`imui-shadcn-demo.inspector.table.header.cell.inspector-field.resize` resolved to one node in the
same after bundle.
