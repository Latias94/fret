# ImUi Table Column Width Diagnostics Gate v1 - Closeout Audit - 2026-05-01

Status: closed

## Verdict

Closed. The app-owned resizable inspector table proof now has a launched diagnostics gate.

## What Shipped

- `tools/diag-scripts/ui-editor/imui/imui-shadcn-adapter-table-column-width-resize.json`
- `tools/diag-scripts/suites/imui-table-column-width-diag-gate/suite.json`
- `tools/gate_imui_shadcn_adapter_table_column_width_diag_source.py`
- Workstream docs and catalog entries for this narrow follow-on.

## Runtime Evidence

```text
cargo run -p fretboard -- diag run tools/diag-scripts/ui-editor/imui/imui-shadcn-adapter-table-column-width-resize.json --dir target/fret-diag/imui-table-column-width-diag-gate-v1 --session-auto --timeout-ms 300000 --exit-after-run --launch -- cargo run -p fret-demo --bin imui_shadcn_adapter_demo
```

- Result: passed.
- Run ID: `1777634686501`.
- Session: `target/fret-diag/imui-table-column-width-diag-gate-v1/sessions/1777634437651-87860`.
- Last artifact: `1777634694683-imui-shadcn-adapter.table-column-width.after`.
- Final width summary label: `Widths: field 180px, value 120px, source 72px`.

## Boundary Check

- `fret-ui-kit::imui` remains a mechanism layer exposing resize response state.
- The demo owns width state and resize policy.
- No persistence, grouped-header resize policy, declarative/headless interop, localization policy,
  or runtime table semantics were added in this lane.

## Continue Policy

Do not reopen this lane for broader table sizing work. Start a narrow follow-on for:

- column width persistence,
- grouped resize policy,
- declarative/headless table interop,
- localization-aware column ids,
- runtime table semantics.
