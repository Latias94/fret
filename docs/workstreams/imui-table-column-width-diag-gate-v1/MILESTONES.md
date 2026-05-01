# ImUi Table Column Width Diagnostics Gate v1 - Milestones

Status: closed
Last updated: 2026-05-01

## M1 - Script Scaffold

Status: landed

- Add `tools/diag-scripts/ui-editor/imui/imui-shadcn-adapter-table-column-width-resize.json`.
- Add `tools/diag-scripts/suites/imui-table-column-width-diag-gate/suite.json`.
- Add a source-marker gate for the script and stable resize selectors.
- Validate JSON shape and source-marker coverage.

Evidence:

- Script JSON, suite JSON, and source-marker gates passed.
- The script uses `label_contains` for the width summary because `ui::text(...).test_id(...)`
  exposes its visible copy as a semantics label.

## M2 - Launched Gate Evidence

Status: met

- Run the launched diagnostics command.
- Record the passing run id, session path, and final artifact label.
- Close the lane when runtime evidence exists.

Evidence:

- Run ID: `1777634686501`.
- Session: `target/fret-diag/imui-table-column-width-diag-gate-v1/sessions/1777634437651-87860`.
- Last artifact: `1777634694683-imui-shadcn-adapter.table-column-width.after`.
- Final width summary label: `Widths: field 180px, value 120px, source 72px`.
