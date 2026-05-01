# ImUi Table Column Width Demo Proof v1

Status: active execution lane
Last updated: 2026-05-01

## Problem

`imui-table-column-resize-v1` shipped the mechanism for table header resize responses, but it
intentionally stopped before proving a real app can turn those responses into visible column width
changes. Without a first-party proof, the feature reads like a response API rather than an editor
workflow.

This lane wires the response into `imui_shadcn_adapter_demo`:

- the demo owns inspector column widths in local app state,
- table columns replay those widths through `TableColumn::px(...)`,
- resizable headers report drag deltas,
- and the demo applies those deltas back to its own width state with local clamping.

## Owner Split

- `apps/fret-examples-imui`: owns the visible proof, state, clamping, and teaching surface.
- `fret-ui-kit::imui`: stays limited to resize affordance and response reporting.
- Future app/project layers: own persistence, saved layouts, and policy beyond this demo.

## Non-Goals

- no helper-owned column width map,
- no persisted layout format,
- no headless/declarative table sizing interop,
- no grouped header resize policy,
- no runtime table semantic changes,
- no multi-window or Linux/Wayland dependency.

## Target Shape

The inspector table should show the intended app-owned loop:

```rust
let widths = inspector_widths_state.layout_value(cx);
let columns = [
    TableColumn::px("Field###inspector-field", widths.field).resizable(),
    TableColumn::px("Value###inspector-value", widths.value).resizable(),
];

let response = ui.table_with_options("...", &columns, options, |table| {
    // rows remain app-owned
});

if let Some(field) = response.header("inspector-field") {
    if field.resize.dragging() {
        // demo updates its own width state from `drag_delta_x`
    }
}
```

The proof should leave stable selectors for diagnostics:

- `imui-shadcn-demo.inspector.table`
- `imui-shadcn-demo.inspector.table.header.cell.<column>.resize`
- `imui-shadcn-demo.inspector.widths`

## Why This Is A Follow-On

The resize response lane is already closed and explicitly deferred app-owned persistence and demo
proof. Reopening it would blur the API surface slice with first-party example wiring. This lane is
smaller and should close once the demo visibly consumes the response.
