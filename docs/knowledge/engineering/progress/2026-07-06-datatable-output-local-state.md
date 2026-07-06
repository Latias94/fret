---
type: Work Progress
title: DataTable demo output LocalState migration
timestamp: 2026-07-06T00:00:00Z
git_branch: refactor/raw-surface-contracts
tags: fret,ui-framework,public-surface,datatable,local-state,raw-model
---

# Summary

`apps/fret-examples/src/datatable_demo.rs` now keeps its shadcn `DataTableViewOutput` handle on the
app-facing `LocalState` surface instead of allocating a raw runtime `Model` through
`app.models_mut().insert(...)`.

The demo still shares the output with `DataTablePagination::new(...)` and
`DataTable::output_model(...)`; the conversion happens through the existing
`IntoTableViewOutputModel for LocalState<TableViewOutput>` adapter. The render loop reads
`table_output.layout_value(cx)` to keep the layout invalidation subscription explicit without
exposing raw model plumbing to first-contact example code.

# Decision

The earlier raw-model audit treated `datatable_demo.rs` as part of the retained table/data-grid
exception bucket. That was too broad for the shadcn `DataTable` output handle because the public
adapter already exists and the cookbook example already proves the app-facing shape.

Keep lower-level retained table/data-grid raw model seams only where the component contract still
requires shared retained state. Do not use `datatable_demo.rs` as precedent for raw
`Model<DataTableViewOutput>` in default examples.

# Verification

- Red first:
  `cargo nextest run -p fret-examples --test datatable_demo_surface datatable_demo_uses_local_state_table_output --no-fail-fast`
- Green after implementation:
  `cargo nextest run -p fret-examples --test datatable_demo_surface datatable_demo_uses_local_state_table_output --no-fail-fast`
- `python3 tools/gate_table_source_policy.py`

# Next

Keep `table_demo.rs`, `table_stress_demo.rs`, and `canvas_datagrid_stress_demo.rs` classified by
their retained/stress contracts. Migrate only after a named table/data-grid binding or owner
contract exists.
