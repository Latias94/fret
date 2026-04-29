# ImUi Table Sortable Diagnostics Gate v1 - M2 Launched Diag Gate - 2026-04-29

Status: landed

## Result

The promoted diagnostics script passed against `imui_shadcn_adapter_demo`.

## Commands

```text
cargo nextest run -p fret-examples imui_shadcn_adapter_demo_keeps_sortable_table_diag_gate --no-fail-fast
cargo build -p fret-demo --bin imui_shadcn_adapter_demo --jobs 1
cargo run -p fretboard -- diag run tools/diag-scripts/ui-editor/imui/imui-shadcn-adapter-sortable-table-gate.json --dir target/fret-diag/imui-table-sortable-diag-gate-v1 --session-auto --timeout-ms 300000 --exit-after-run --launch -- cargo run -p fret-demo --bin imui_shadcn_adapter_demo
```

## Evidence

- Source-marker test: passed.
- Launched diagnostics run: passed.
- Run ID: `1777454218396`.
- Session: `target/fret-diag/imui-table-sortable-diag-gate-v1/sessions/1777453912985-52424`.
- Last artifact: `1777454223920-imui-shadcn-adapter.sortable-table.after`.

## Notes

The earlier timeouts were cold test/demo build timeouts, not script assertion failures. The final
script result reached `stage: passed`.
