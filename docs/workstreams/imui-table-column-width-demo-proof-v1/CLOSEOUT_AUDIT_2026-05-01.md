# ImUi Table Column Width Demo Proof v1 - Closeout Audit

Status: closed
Date: 2026-05-01

## Verdict

The first-party IMUI demo now proves app-owned table column resizing and this lane is closed.

`imui_shadcn_adapter_demo` owns inspector table column widths, replays those widths through
`TableColumn::px(...)`, opts columns into resizable headers, and applies
`TableHeaderResponse::resize.drag_delta_x()` back to the demo-owned state. The helper remains a
response surface; no width map, persistence, or table semantics were added to `fret-ui-kit::imui`.

## Shipped Evidence

- `InspectorColumnWidths` is local app state in `imui_shadcn_adapter_demo`.
- Compact and regular inspector tables use stable column ids and app-owned pixel widths.
- Header resize handles are enabled with explicit local min/max limits.
- `apply_inspector_width_delta(...)` consumes resize drag deltas and clamps them in the demo layer.
- `imui-shadcn-demo.inspector.widths` exposes the current width summary for diagnostics.
- Existing sortable header behavior remains routed through the app-owned sort state.

## Gates

```text
cargo nextest run -p fret-examples-imui imui_shadcn_adapter_demo_owns_resizable_table_width_state --no-fail-fast
cargo check -p fret-examples-imui --bin imui_shadcn_adapter_demo --jobs 1
cargo fmt --package fret-examples-imui --check
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-table-column-width-demo-proof-v1/WORKSTREAM.json
git diff --check
```

All gates passed on 2026-05-01.

## Follow-On Boundary

Do not reopen this lane for larger table policy. Start a narrower follow-on for any of:

- width persistence and saved layouts,
- declarative/headless table sizing interop examples,
- grouped header or multi-column resize policy,
- localization-aware column ids,
- runtime table semantics or ID-stack changes.
