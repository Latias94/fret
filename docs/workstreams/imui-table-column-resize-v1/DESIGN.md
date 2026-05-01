# ImUi Table Column Resize v1

Status: active execution lane
Last updated: 2026-05-01

## Problem

The IMUI table helper has stable column ids and sortable header responses, but it still cannot expose
a Dear ImGui-style column boundary drag from the immediate table surface. The declarative table stack
already has richer headless column sizing, but the IMUI facade should not force authors to abandon the
lightweight table helper just to make an editor proof feel adjustable.

This lane adds the smallest useful policy-layer bridge:

- a column can opt into a resize handle,
- the table response reports that handle's drag state by stable column identity,
- and the application remains responsible for storing, clamping, persisting, and replaying column
  widths through `TableColumnWidth`.

## Owner Split

- `fret-ui-kit::imui`: renders the resize affordance and reports pointer drag response.
- app / recipe / `fret-ui-headless::table`: owns column sizing state, persistence, resize mode, and
  any width application policy.
- `crates/fret-ui`: stays mechanism-only; no runtime table semantic growth is admitted here.

## Non-Goals

- no helper-owned column sizing map,
- no persisted table layout format,
- no row sorting engine,
- no multi-sort policy,
- no grouped header resize policy,
- no runtime ID-stack or table contract changes,
- no Linux/Wayland or multi-window dependency.

## Target Shape

The first slice should allow this shape:

```rust
let columns = [
    TableColumn::px("Name###asset-name", Px(180.0)).resizable(),
    TableColumn::px("Status###asset-status", Px(96.0)),
];

let response = ui.table_with_options("assets", &columns, options, |table| {
    // rows remain app-owned
});

if let Some(name) = response.header("asset-name") {
    if name.resize.dragging() {
        // App decides how to turn drag_total_x into a new TableColumnWidth.
    }
}
```

The handle should have a stable diagnostics id derived from the existing header cell id:

- `table.test_id.header.cell.<column-id>.resize`

## Why This Is A Follow-On

The closed table identity and sortable lanes explicitly rejected column sizing and persistence from
their scope. Reopening them would blur ownership. This lane is narrow enough to land with local unit
and interaction tests, then close without committing to a larger table engine.
