Status: Active (workstream tracker; not a contract)

This document is the **capability inventory** for the TanStack Table v8 `table-core` parity
workstream. The goal is **capability parity** (Fret must not be weaker than upstream), not 1:1
method-name parity.

Upstream reference (local checkout):

- `F:/SourceCodes/Rust/fret/repo-ref/table/packages/table-core`
- Baseline: `@tanstack/table-core@8.21.3` (commit `e172109fca4cc403a07236ed8fa103450ceba5e9`)

Fret implementation:

- `ecosystem/fret-ui-headless/src/table/` (`Table`, `TableState`, TanStack-shaped import/export)

Legend:

- **Aligned**: parity-gated by fixtures (or a dedicated gate that proves the observable outcome).
- **Partial**: implemented, but lacks option/edge-case parity coverage.
- **Missing**: no equivalent capability surface yet.

---

## Core APIs (table/row/column/header/cell)

Source of truth:

- Table (`CoreInstance`): `table-core/src/core/table.ts`
- Row (`CoreRow`): `table-core/src/core/row.ts`
- Column (`CoreColumn`): `table-core/src/core/column.ts`
- Headers (`HeadersInstance`, `CoreHeader`): `table-core/src/core/headers.ts`
- Cell (`CoreCell`): `table-core/src/core/cell.ts`

### Table (CoreInstance)

| Upstream API | Fret mapping | Status | Evidence |
| --- | --- | --- | --- |
| `getAllColumns/getAllFlatColumns/getAllLeafColumns/getColumn` | `Table::column_tree_snapshot` + `Table::ordered_columns` + `Table::column` + `Table::visible_leaf_columns`-style surfaces | Partial | `tanstack_v8_headers_cells_parity.rs` (core model snapshot) |
| `getCoreRowModel` | `Table::core_row_model()` | Aligned | fixtures + gates across multiple cases |
| `getRowModel` | `Table::row_model()` | Aligned | fixtures + gates across multiple cases |
| `getRow(id, searchAll?)` | `Table::row_by_id(..)` / `Table::row_key_for_id(..)` (+ `rows_by_id` parity gate) | Aligned | `tanstack_v8_row_id_lookup_parity.rs` |
| `getState` | `TableState` passed into `Table::builder().state(..)` (engine is pure) | Partial | state roundtrip gates exist, but not a full instance-style API |
| `reset/setState/setOptions` | Rust-native: build new `Table` with new `TableState`/`TableOptions` | Partial | N/A (API-shape differs by design) |

### Row (CoreRow)

| Upstream API | Fret mapping | Status | Evidence |
| --- | --- | --- | --- |
| `id/index/depth/parentId/subRows` | `RowModel::row(..)` (`RowId`, `RowKey`, `depth`, `parent`, `sub_rows`) | Partial | row model fixtures across cases |
| `getValue/getUniqueValues/renderValue` | `ColumnDef` value fns + `Table::cell_render_value` (fallback) | Partial | `render_fallback.json` parity |
| `getAllCells` | `snapshot_cells_for_row(..)` / `RowCellsSnapshot` | Partial | `tanstack_v8_headers_cells_parity.rs` |
| `getParentRow(s)/getLeafRows` | `RowModel` traversal + helpers | Partial | currently unit/fixture gated only where needed |

### Column (CoreColumn)

| Upstream API | Fret mapping | Status | Evidence |
| --- | --- | --- | --- |
| `id/parent/depth/columns` | `ColumnDef` nested columns + `column_tree` snapshot | Partial | `tanstack_v8_headers_cells_parity.rs` |
| `getFlatColumns/getLeafColumns` | `Table` core model snapshot leaf sets | Partial | `tanstack_v8_headers_cells_parity.rs` |

### Header / Cell (core)

| Upstream API | Fret mapping | Status | Evidence |
| --- | --- | --- | --- |
| `getHeaderGroups` (+ pin variants) | `build_header_groups` + `Table::header_groups_snapshot`-style surfaces | Partial | `tanstack_v8_headers_cells_parity.rs` |
| Header placeholder semantics | `HeaderSnapshot.is_placeholder/placeholder_id` | Aligned | `headers_cells.json` parity |
| Cell id `${rowId}_${columnId}` | `CellSnapshot.id` | Aligned | `headers_cells.json` parity |

---

## Pinning (RowPinning / ColumnPinning)

Source of truth:

- `table-core/src/features/RowPinning.ts`
- `table-core/src/features/ColumnPinning.ts`

### Row pinning (capability)

| Upstream API | Fret mapping | Status | Evidence |
| --- | --- | --- | --- |
| `row.getCanPin()` | `Table::row_can_pin(RowKey)` | Aligned | `pinning.json`, `grouping.json` |
| `row.getIsPinned()` | `Table::row_is_pinned(RowKey)` | Aligned | `pinning.json` |
| `row.getPinnedIndex()` | `Table::row_pinned_index(RowKey)` | Aligned | `pinning.json` |
| `row.pin(position, includeLeafRows, includeParentRows)` | `Table::row_pinning_updater(..)` / `row_pinning_updater_by_id(..)` | Aligned | `pinning_tree.json`, `pinning_grouped_rows.json` |
| `table.getTopRows/getCenterRows/getBottomRows` | `Table::top_row_ids/center_row_ids/bottom_row_ids` (and `*_row_keys`) | Aligned | `pinning.json`, `grouping.json`, `pinning_grouped_rows.json` |
| `table.getIsSomeRowsPinned(position?)` | `Table::is_some_rows_pinned(..)` | Aligned | `pinning.json` |
| `setRowPinning/resetRowPinning` | `Table::reset_row_pinning(..)` + updater surfaces | Aligned | `pinning.json` |

### Row pinning (grouped-row id pinning)

| Upstream capability | Fret mapping | Status | Evidence |
| --- | --- | --- | --- |
| Pin grouped root rows by id (e.g. `role:1`) | `row_pinning_updater_by_id("role:1", searchAll=true, ..)` | Aligned | `pinning_grouped_rows.json` + `tanstack_v8_pinning_grouped_rows_parity.rs` |

### Column pinning (capability)

| Upstream API | Fret mapping | Status | Evidence |
| --- | --- | --- | --- |
| `column.getCanPin()` | `Table::column_can_pin(column_id)` | Aligned | `column_pinning.json` |
| `column.getIsPinned()` | `Table::column_pin_position(column_id)` | Aligned | `column_pinning.json` |
| `column.getPinnedIndex()` | `Table::column_pinned_index(column_id)` | Aligned | `column_pinning.json` |
| `column.pin(position)` | `Table::toggled_column_pinning(..)` / `Table::column_pinning_updater(..)` | Aligned | `column_pinning.json` |
| `row.getLeft/Center/RightVisibleCells()` | `snapshot_cells_for_row(..)` pinned splits | Aligned | `column_pinning.json` |
| `table.getLeft/Center/RightLeafColumns()` | `split_pinned_columns(..)` over visible ordered leaf columns | Partial | planned consumer-facing helpers + direct gate (`HTP-colpin-030`) |
| `setColumnPinning/resetColumnPinning` | `Table::reset_column_pinning(..)` + updater surfaces | Aligned | `column_pinning.json` |

---

## Sorting (RowSorting)

Source of truth:

- `table-core/src/features/RowSorting.ts`

| Upstream API / capability | Fret mapping | Status | Evidence |
| --- | --- | --- | --- |
| `getPreSortedRowModel/getSortedRowModel` | `Table::pre_sorted_row_model()` / `Table::sorted_row_model()` | Aligned | `sorting_fns.json`, `sort_undefined.json` |
| `resetSorting(defaultState?)` | `Table::reset_sorting(default_state)` | Aligned | `resets.json`, sorting fixtures |
| `setSorting(updater)` | Rust-native: update `TableState.sorting` outside the engine (`Updater<Vec<SortSpec>>`) | Partial | N/A (API-shape differs) |
| `column.toggleSorting(desc?, isMulti?)` | `toggle_sorting_tanstack(&mut SortingState, &ColumnDef, options, multi, auto_sort_dir_desc)` | Aligned | `tanstack_v8_parity.rs`, `tanstack_v8_sort_undefined_parity.rs`, `tanstack_v8_sorting_manual_parity.rs` |
| `column.getToggleSortingHandler()` gating | `toggle_sorting_handler_tanstack(..)` (models TanStack “can sort?” gating + multi-sort event) | Partial | covered indirectly by fixtures; expand explicit gate as needed |

---

## Filtering (ColumnFiltering / GlobalFiltering)

Source of truth:

- `table-core/src/features/ColumnFiltering.ts`
- `table-core/src/features/GlobalFiltering.ts`

| Upstream API / capability | Fret mapping | Status | Evidence |
| --- | --- | --- | --- |
| `getPreFilteredRowModel/getFilteredRowModel` | `Table::pre_filtered_row_model()` / `Table::filtered_row_model()` | Aligned | `filtering_fns.json` |
| `column.setFilterValue(updater)` | `set_column_filter_value_tanstack(&mut ColumnFiltersState, column_id, value)` | Aligned | `filtering_fns.json` |
| `row.columnFilters` / `row.columnFiltersMeta` | `Table::row_filter_state_snapshot()` | Aligned | `filtering_meta.json` |
| `setGlobalFilter(updater)` | `Table::global_filter_updater_set_value(..)` (plus Rust-native `TableState.global_filter` updates) | Partial | helper is smoke-gated; controlled hook parity still open (`HTP-filt-100`) |
| `column.getCanFilter()` / `column.getFilterValue()` / `column.getIsFiltered()` | `Table::{column_can_filter,column_filter_value,column_is_filtered,column_filter_index,column_filters_updater_set_value}` | Partial | helper surfaces are smoke-gated; expand fixture parity as needed (`HTP-filt-090`) |
| `column.getCanGlobalFilter()` | `Table::column_can_global_filter(..)` + `TableBuilder::get_column_can_global_filter(..)` | Partial | helper surface is smoke-gated; expand fixture parity as needed (`HTP-filt-090`) |

---

## Pagination (RowPagination)

Source of truth:

- `table-core/src/features/RowPagination.ts`

| Upstream API / capability | Fret mapping | Status | Evidence |
| --- | --- | --- | --- |
| `getPrePaginationRowModel/getPaginationRowModel/getRowModel` | `Table::pre_pagination_row_model()` / `Table::row_model()` | Aligned | `pagination.json` |
| `getRowCount/getPageCount/getCanNextPage/getCanPreviousPage/getPageOptions` | `Table::{row_count,page_count,can_next_page,can_previous_page,page_options}` | Aligned | `pagination.json` |
| `setPageIndex/setPageSize` | `Table::{set_page_index,set_page_size}` (+ updater variants) | Aligned | `pagination.json` |
| `nextPage/previousPage/firstPage/lastPage` | `Table::{next_page,previous_page,first_page,last_page}` | Aligned | `pagination.json` |
| `resetPageIndex/resetPageSize/resetPagination` | `Table::{reset_page_index,reset_page_size,reset_pagination}` | Aligned | `pagination.json` |
| Auto-reset `_queue` behavior | modeled via state-transition parity gates (not a first-class runtime queue) | Partial | `auto_reset.json` |

---

## Row selection (RowSelection)

Source of truth:

- `table-core/src/features/RowSelection.ts`

| Upstream API / capability | Fret mapping | Status | Evidence |
| --- | --- | --- | --- |
| `row.getIsSelected/getIsSomeSelected/getIsAllSubRowsSelected` | `Table::{row_is_selected,row_is_some_selected,row_is_all_sub_rows_selected}` | Aligned | `selection.json`, `selection_tree.json` |
| `row.toggleSelected(value?, { selectChildren })` | `Table::row_selection_updater(..)` (+ by-id variants) | Aligned | selection fixtures + `row_id_state_ops.json` |
| `table.getIsAllRowsSelected/getIsSomeRowsSelected/getIsAllPageRowsSelected/getIsSomePageRowsSelected` | `Table::{is_all_rows_selected,is_some_rows_selected,is_all_page_rows_selected,is_some_page_rows_selected}` | Aligned | selection fixtures |
| `toggleAllRowsSelected/toggleAllPageRowsSelected` | `Table::{toggled_all_rows_selected,toggled_all_page_rows_selected}` | Aligned | selection fixtures |
| `getSelectedRowModel` (+ filtered/grouped) | `Table::{selected_row_model,filtered_selected_row_model,grouped_selected_row_model,page_selected_row_model}` | Aligned | selection fixtures |
| `resetRowSelection` | `Table::reset_row_selection(default_state)` | Aligned | `resets.json` + selection fixtures |

---

## Expanding (RowExpanding)

Source of truth:

- `table-core/src/features/RowExpanding.ts`

| Upstream API / capability | Fret mapping | Status | Evidence |
| --- | --- | --- | --- |
| `row.toggleExpanded/getIsExpanded/getIsAllParentsExpanded/getCanExpand` | `Table::{toggled_row_expanded,row_is_expanded_for_row,row_is_all_parents_expanded,row_can_expand_for_row}` (plus by-id toggles) | Aligned | `expanding.json`, `grouping.json` |
| `getPreExpandedRowModel/getExpandedRowModel` | `Table::{pre_expanded_row_model,expanded_row_model}` | Aligned | `expanding.json` |
| `getIsAllRowsExpanded/getIsSomeRowsExpanded/toggleAllRowsExpanded` | `Table::{is_all_rows_expanded,is_some_rows_expanded,toggled_all_rows_expanded}` | Aligned | `expanding.json` |
| `resetExpanded` | `Table::reset_expanded(default_state)` | Aligned | `expanding.json` |
| `paginateExpandedRows` behavior | `TableOptions.paginate_expanded_rows` | Aligned | `expanding.json` |

---

## Grouping (ColumnGrouping)

Source of truth:

- `table-core/src/features/ColumnGrouping.ts`

| Upstream API / capability | Fret mapping | Status | Evidence |
| --- | --- | --- | --- |
| `getPreGroupedRowModel/getGroupedRowModel` | `Table::pre_grouped_row_model()` + `Table::grouped_row_model()` | Aligned | `grouping.json` |
| `setGrouping/resetGrouping` | `Table::{grouping_updater,reset_grouping}` + controlled hook parity via updater snapshots | Aligned | `grouping.json`, `resets.json` |
| `column.getCanGroup/getIsGrouped/getGroupedIndex/toggleGrouping/handler` | `Table::{column_can_group,is_column_grouped,column_grouped_index,toggled_column_grouping,grouping_handler_updater}` | Aligned | `grouping.json` |
| `groupedColumnMode` interactions | `TableOptions.grouped_column_mode` + `Table::ordered_columns()` | Aligned | `headers_cells.json` |
| Aggregation registry + fallback value | `aggregation_fns.rs` + `Table::cell_render_value` | Aligned | `grouping_aggregation_fns.json`, `render_fallback.json` |

---

## Column ordering / visibility / sizing (core UI affordances)

Source of truth:

- `table-core/src/features/ColumnOrdering.ts`
- `table-core/src/features/ColumnVisibility.ts`
- `table-core/src/features/ColumnSizing.ts`

| Upstream capability | Fret mapping | Status | Evidence |
| --- | --- | --- | --- |
| Column ordering state + reset | `Table::{column_order,reset_column_order,toggled_column_order_move}` | Aligned | `visibility_ordering.json`, `resets.json` |
| Column visibility state + toggle + reset | `Table::{column_visibility,is_column_visible,toggled_column_visibility,toggled_all_columns_visible,reset_column_visibility}` | Aligned | `visibility_ordering.json`, `resets.json` |
| Visible ordered columns | `Table::{ordered_columns,visible_columns,pinned_visible_columns}` | Partial | visibility+ordering+pinning are fixture-gated; expand API inventory as needed |
| Column sizing totals + start/after offsets | `Table::{total_size,left_total_size,center_total_size,right_total_size,column_start,column_after}` | Aligned | `column_sizing.json`, `visibility_ordering.json` |
| Resize lifecycle (`onChange`/`onEnd`, RTL) | `Table::{started_column_resize,dragged_column_resize,ended_column_resize}` + `columnSizingInfo` state | Partial | `column_sizing.json`, `column_resizing_group_headers.json` |

---

## Faceting (ColumnFaceting / GlobalFaceting)

Source of truth:

- `table-core/src/features/ColumnFaceting.ts`
- `table-core/src/features/GlobalFaceting.ts`

| Upstream capability | Fret mapping | Status | Evidence |
| --- | --- | --- | --- |
| `column.getFacetedRowModel/getFacetedUniqueValues/getFacetedMinMaxValues` | `Table::{faceted_row_model,faceted_unique_values,faceted_min_max_u64}` | Aligned | `faceting.json` |
| “Global faceting” (`__global__`) surface | not first-class; current fixtures only assert empty/null globals (TanStack built-ins warn) | Partial | `faceting.json` |

---

## Open inventory work (next)

This inventory is intentionally incomplete. Next expansions (tracked in `HTP-cap-010` / `HTP-base-004`):

- Header/footer/flat/leaf header inventories under visibility + pinning + nested columns (deep nesting + edge cases).
- Fill in missing “column/row instance method” helpers where consumers should not have to reimplement logic (e.g. `getCanFilter`-style gates).
- Global faceting instance surface (`getGlobalFaceted*`) if/when consumers need it.
