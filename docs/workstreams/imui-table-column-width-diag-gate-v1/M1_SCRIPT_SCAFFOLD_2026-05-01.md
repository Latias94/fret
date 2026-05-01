# ImUi Table Column Width Diagnostics Gate v1 - M1 Script Scaffold - 2026-05-01

Status: landed

## Landed

- Added a schema v2 diagnostics script for the resizable inspector table width proof.
- Added a suite manifest for the script.
- Added source-marker coverage tying the script to stable demo selectors and app-owned resize
  markers.

## Design Decision

The script uses `drag_pointer_until` rather than `drag_pointer`. The table resize proof consumes
`TableColumnResizeResponse::drag_delta_x()` during render while the drag is active; a burst
down/move/up drag does not leave a frame for that immediate response loop.

The width summary assertions use `label_contains`, because `ui::text(...).test_id(...)` exposes the
visible summary as a semantics label.

## Validation

Passed:

```text
python -m json.tool tools/diag-scripts/ui-editor/imui/imui-shadcn-adapter-table-column-width-resize.json
python -m json.tool tools/diag-scripts/suites/imui-table-column-width-diag-gate/suite.json
python tools/gate_imui_shadcn_adapter_table_column_width_diag_source.py
```

Attempted but not counted as a gate:

```text
cargo run -p fretboard -- diag config doctor --mode launch --print-launch-policy
```

The public `fretboard` CLI in this workspace does not expose `diag config doctor`; the launched
`diag run` command still prints launch policy hygiene details.
