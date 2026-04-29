# ImUi Table Sortable Demo Proof v1 - M1 App-Owned Sortable Demo Slice - 2026-04-29

Status: complete

## Summary

`imui_shadcn_adapter_demo` now demonstrates the sortable table header response surface in a
runnable product-style panel. The demo owns `InspectorSort`, sorts its own row snapshot, and uses
`TableResponse::header(sort_column_id).clicked()` only as the trigger.

## Adopted Pattern

```rust
let table_response = ui.table_with_options(...);
if table_response
    .header(sort_column_id)
    .is_some_and(|header| header.clicked())
{
    let _ = inspector_sort_state.update_in(ui.cx_mut().app.models_mut(), |sort| {
        *sort = sort.toggled();
    });
}
```

## Gate Evidence

- `cargo check -p fret-examples --jobs 1`
- `python tools/gate_imui_shadcn_adapter_sortable_table_source.py`
