# Component recipe: DataGrid (canvas-backed)

Goal: spreadsheet-scale density with a ~constant UI node count, by rendering dense cells via canvas ops.

## Upstream references

There is no 1:1 shadcn/Radix primitive for this surface; treat it as an **editor-grade** building block.

## Fret building blocks

- Public surface:
  - `fret-ui-shadcn::DataGrid` (type alias to the canvas-backed `DataGridCanvas`)
  - `fret-ui-shadcn::DataGridCanvasAxis` (row/col axis descriptors)
- Inputs:
  - `rows: DataGridCanvasAxis` and `cols: DataGridCanvasAxis`:
    - `keys` (stable keys)
    - `revision` (bump when the axis changes)
    - measure mode (`fixed` / `measured`), estimate, gap, min/max
  - `cell_text_at: Fn(row_key, col_key) -> Arc<str>` (should be cheap)

## Checklist (what to verify)

- Identity + revisions:
  - axis keys are stable; revisions bump on structural changes
  - measurement reset policy is explicit (`reset_measurements_on_revision_change`)
- Performance:
  - `cell_text_at` does not allocate excessively; prefer cached `Arc<str>` where possible
  - overscan is tuned (too high wastes work; too low causes “popping”)
- Editing UX (usually overlays, not per-cell widgets):
  - selection rectangles, editor popovers, and context menus live in overlay layers
  - focus/keyboard flows remain consistent while scrolling
- Scrollbars:
  - thumb sizing and clamp behavior are stable under resize and data changes

## `test_id` suggestions

- `data-grid-root`
- `data-grid-scroll`

## See also

- `fret-scroll-and-virtualization` (windowing + stable keys mindset)
- `references/mind-models/mm-overlays-and-focus.md` (edit overlays and focus restore)
- `references/recipes/apps/app-data-table.md` (table-centric, headless-state approach)
