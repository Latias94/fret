# App recipe: DataTable (TanStack-aligned)

Goal: a dense, editor-friendly data table surface (fixed header + virtualized body) with reliable sorting/filtering/selection and strong regression gates.

## References

- shadcn docs: https://ui.shadcn.com/docs/components/data-table
- Gallery docs snippet: `apps/fret-ui-gallery/src/docs.rs` (DataTable usage)
- Demo app: `apps/fret-examples/src/datatable_demo.rs`
- Diag scripts:
  - `tools/diag-scripts/ui-gallery-data-table-smoke.json`
  - `tools/diag-scripts/ui-gallery-data-table-torture-scroll-refresh.json`

## Building blocks

- Headless state: `fret_ui_headless::table::TableState` stored in a `Model<_>`.
- UI: `fret-ui-shadcn::DataTable` + optional recipes (`DataTableToolbar`, `DataTablePagination`).

## Checklist

- **Row identity**: stable row keys (required for virtualization correctness).
- **Revisions**: update `data_revision` / `rows_revision` when underlying data changes.
- **Selection**: selection state is in the headless model; UI reflects it deterministically.
- **Performance**:
  - avoid per-frame allocations in `cell_at`
  - keep cell rendering cheap; prefer simple elements and caching where appropriate
- **UX**:
  - fixed header stays aligned while scrolling
  - column widths feel consistent and don’t jitter

## Regression gates (recommended)

- Keep a minimal scripted repro (scroll, sort, filter) that captures a bundle at each step.
- Add one invariant test for the most fragile contract (e.g., “header/body column widths stay aligned”).

## See also

- `fret-diag-workflow` (scripted repro + packaging)
