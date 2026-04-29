# ImUi Table Sortable Diagnostics Gate v1 - Milestones

Status: closed

## M1 - Script Scaffold

Status: landed

- Added `tools/diag-scripts/ui-editor/imui/imui-shadcn-adapter-sortable-table-gate.json`.
- Added `tools/diag-scripts/suites/imui-table-sortable-diag-gate/suite.json`.
- Added source-marker coverage in `apps/fret-examples/src/lib.rs`.
- JSON validation passed for the script and suite.

## M2 - Launched Gate Evidence

Status: met

- Run the launched diagnostics command.
- Record the passing bundle path.
- Close the lane when the runtime evidence exists.

Evidence:

- Run ID: `1777454218396`.
- Session: `target/fret-diag/imui-table-sortable-diag-gate-v1/sessions/1777453912985-52424`.
- Last artifact: `1777454223920-imui-shadcn-adapter.sortable-table.after`.
