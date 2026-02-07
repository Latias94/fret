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

Last updated: 2026-02-07

---

## Capability gap snapshot (what can still make us weaker)

This section is intentionally short and action-oriented. It is the “capability parity”
definition-of-done for `HTP-cap-010` / `HTP-base-004`.

**P0 (must close; consumers will re-implement and drift):**

- **Core snapshot completeness**: `CoreModelSnapshot` is still “structure + ids” heavy.
  - Gap: lacks a first-class, versioned inventory of column/header/cell capabilities that UI consumers typically
    query on instances (e.g. `getCanResize`, `getIsPlaceholder`, pin-family splits).
  - Tracking: `HTP-core-040` (remaining scope) + future `HTP-core-*` follow-ups.
- **Memo/perf guardrails for rebuild-each-frame**:
  - Gap: we do not yet have a documented + tested integration pattern for “rebuild per frame, keep memo cache”.
  - Tracking: `HTP-memo-020` + `HTP-perf-010`.

**P1 (should close to match upstream ergonomics without copying the JS API):**

- **State JSON spec**: a written, enforceable spec for omitted-vs-explicit defaults and normalization rules.
  - Tracking: `HTP-state-010` / `HTP-state-011`.
- **Inventory completeness**: convert the raw upstream instance member dump into an explicit checklist with status + evidence.
  - Tracking: `HTP-cap-010` / `HTP-base-004`.

---

## Consumer-driven “must-have” surface (WIP)

Before we chase 1:1 instance method parity, we track what our current UI consumers *actually* need, so we can ensure
we are not weaker than upstream while keeping the Rust surface idiomatic.

- `fret-ui-kit` virtualized table: `ecosystem/fret-ui-kit/src/declarative/table.rs`
  - Uses the engine as a pure derived-model provider (build `Table`, read row models, compute rendering).
  - Currently calls into: `Table::{core_row_model,pre_pagination_row_model,grouped_row_model,top_row_keys,center_row_keys,bottom_row_keys}`
    plus standalone helpers (`order_columns`, `split_pinned_columns`, visibility/pinning/sizing helpers).
- `fret-ui-shadcn` DataTable: (TODO) identify the minimal required engine surface once it is wired to `fret-ui-headless`.
  - Note: current `fret-ui-shadcn` `DataTable*` recipes primarily mutate `TableState` directly
    (`ecosystem/fret-ui-shadcn/src/data_table_recipes.rs`) and rely on `fret-ui-kit` for rendering.

---

## Mapping strategy (TanStack → Fret)

Goal: **capability parity**, not method-name parity. The mapping rules below are the working contract for `HTP-cap-010`.

- Prefer **pure derived models** over mutable instance methods:
  - TanStack: `table.setState` + instance methods that mutate internal state.
  - Fret: consumers own `TableState`; the engine provides updaters (`Updater<T>`) and derived snapshots.
- Prefer **snapshots** over “live” instance objects:
  - TanStack exposes rich `Column/Row/Header/Cell` instances with caches and closures.
  - Fret exposes stable, JSON-serializable snapshots (IDs, structure, and computed fields) plus targeted helper methods.
- Add **targeted helper surfaces** when TanStack has “policy logic” that consumers should not re-implement:
  - Example: filtering/sorting/pinning capability gates, sizing offsets, pinned leaf splits.
- Treat underscore-prefixed upstream members as **behavioral obligations**, not API obligations:
  - If an upstream `_queue` / `_autoResetX` affects observable outcomes, we gate outcomes with fixtures rather than mirroring the internal mechanism.

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
| `getHeaderGroups/getLeftHeaderGroups/getCenterHeaderGroups/getRightHeaderGroups` | `Table::{header_groups,left_header_groups,center_header_groups,right_header_groups}` (snapshot output) | Aligned | `tanstack_v8_headers_cells_parity.rs` |
| `getFooterGroups/getLeftFooterGroups/getCenterFooterGroups/getRightFooterGroups` | `Table::{footer_groups,left_footer_groups,center_footer_groups,right_footer_groups}` | Aligned | `tanstack_v8_headers_cells_parity.rs` |
| `getFlatHeaders/getLeftFlatHeaders/getCenterFlatHeaders/getRightFlatHeaders` | `Table::{flat_headers,left_flat_headers,center_flat_headers,right_flat_headers}` | Aligned | `tanstack_v8_headers_cells_parity.rs` |
| `getLeafHeaders/getLeftLeafHeaders/getCenterLeafHeaders/getRightLeafHeaders` | `Table::{leaf_headers,left_leaf_headers,center_leaf_headers,right_leaf_headers}` | Aligned | `tanstack_v8_headers_cells_parity.rs` |
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
| `table.getLeft/Center/RightLeafColumns()` | `Table::{pinned_leaf_columns,left_leaf_columns,center_leaf_columns,right_leaf_columns}` | Aligned | `column_pinning.json` + `tanstack_v8_column_pinning_parity.rs` |
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
| `setGlobalFilter(updater)` | `Table::global_filter_updater_set_value(..)` (plus Rust-native `TableState.global_filter` updates) | Partial | `filtering_fns.json` + `tanstack_v8_filtering_fns_parity.rs` |
| `column.getCanFilter()` / `column.getFilterValue()` / `column.getIsFiltered()` | `Table::{column_can_filter,column_filter_value,column_is_filtered,column_filter_index,column_filters_updater_set_value}` | Partial | `filtering_fns.json` + `tanstack_v8_filtering_fns_parity.rs` (`filtering_helpers`) |
| `column.getCanGlobalFilter()` | `Table::column_can_global_filter(..)` + `TableBuilder::get_column_can_global_filter(..)` | Partial | `filtering_fns.json` + `tanstack_v8_filtering_fns_parity.rs` (`filtering_helpers`) |

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
  - Next concrete: close `HTP-core-050` with fixture parity for flat/leaf/footer inventories.
- Fill in missing “column/row instance method” helpers where consumers should not have to reimplement logic (e.g. `getCanFilter`-style gates).
- Global faceting instance surface (`getGlobalFaceted*`) if/when consumers need it.

---

## Appendix: upstream instance method inventory (raw, WIP)

This is a **raw** inventory of instance members assigned on upstream `table`, `column`, `row`, `header`, and `cell`
objects. It is intentionally redundant and includes some underscore-prefixed internals, because those often correspond
to observable behavior (memo caches / queues / derived model hooks).

Source (local): `F:/SourceCodes/Rust/fret/repo-ref/table/packages/table-core/src/**/*.ts`.

Extraction command (PowerShell; requires ripgrep `rg` with `--pcre2`):

```ps1
$root='F:\SourceCodes\Rust\fret\repo-ref\table\packages\table-core\src'
cd $root
rg --pcre2 -o "table\.[A-Za-z0-9_]+"  -g"*.ts" | % { $_.Split(':')[-1] } | sort -unique
rg --pcre2 -o "column\.[A-Za-z0-9_]+" -g"*.ts" | % { $_.Split(':')[-1] } | sort -unique
rg --pcre2 -o "row\.[A-Za-z0-9_]+"    -g"*.ts" | % { $_.Split(':')[-1] } | sort -unique
rg --pcre2 -o "header\.[A-Za-z0-9_]+" -g"*.ts" | % { $_.Split(':')[-1] } | sort -unique
rg --pcre2 -o "cell\.[A-Za-z0-9_]+"   -g"*.ts" | % { $_.Split(':')[-1] } | sort -unique
```

### Table instance (`table.*`)

```
table._autoResetExpanded
table._autoResetPageIndex
table._features
table._getAllFlatColumnsById
table._getColumnDefs
table._getCoreRowModel
table._getDefaultColumnDef
table._getExpandedRowModel
table._getFilteredRowModel
table._getGlobalFacetedMinMaxValues
table._getGlobalFacetedRowModel
table._getGlobalFacetedUniqueValues
table._getGroupedRowModel
table._getOrderColumnsFn
table._getPaginationRowModel
table._getPinnedRows
table._getRowId
table._getSortedRowModel
table._queue
table.firstPage
table.getAllColumns
table.getAllFlatColumns
table.getAllLeafColumns
table.getBottomRows
table.getCanNextPage
table.getCanPreviousPage
table.getCanSomeRowsExpand
table.getCenterFlatHeaders
table.getCenterFooterGroups
table.getCenterHeaderGroups
table.getCenterLeafColumns
table.getCenterLeafHeaders
table.getCenterRows
table.getCenterTotalSize
table.getCenterVisibleLeafColumns
table.getColumn
table.getCoreRowModel
table.getExpandedDepth
table.getExpandedRowModel
table.getFilteredRowModel
table.getFilteredSelectedRowModel
table.getFlatHeaders
table.getFooterGroups
table.getGlobalAutoFilterFn
table.getGlobalFacetedMinMaxValues
table.getGlobalFacetedRowModel
table.getGlobalFacetedUniqueValues
table.getGlobalFilterFn
table.getGroupedRowModel
table.getGroupedSelectedRowModel
table.getHeaderGroups
table.getIsAllColumnsVisible
table.getIsAllPageRowsSelected
table.getIsAllRowsExpanded
table.getIsAllRowsSelected
table.getIsSomeColumnsPinned
table.getIsSomeColumnsVisible
table.getIsSomePageRowsSelected
table.getIsSomeRowsExpanded
table.getIsSomeRowsPinned
table.getIsSomeRowsSelected
table.getLeafHeaders
table.getLeftFlatHeaders
table.getLeftFooterGroups
table.getLeftHeaderGroups
table.getLeftLeafColumns
table.getLeftLeafHeaders
table.getLeftTotalSize
table.getLeftVisibleLeafColumns
table.getPageCount
table.getPageOptions
table.getPaginationRowModel
table.getPreExpandedRowModel
table.getPreFilteredRowModel
table.getPreGroupedRowModel
table.getPrePaginationRowModel
table.getPreSelectedRowModel
table.getPreSortedRowModel
table.getRightFlatHeaders
table.getRightFooterGroups
table.getRightHeaderGroups
table.getRightLeafColumns
table.getRightLeafHeaders
table.getRightTotalSize
table.getRightVisibleLeafColumns
table.getRow
table.getRowCount
table.getRowModel
table.getSelectedRowModel
table.getSortedRowModel
table.getState
table.getToggleAllColumnsVisibilityHandler
table.getToggleAllPageRowsSelectedHandler
table.getToggleAllRowsExpandedHandler
table.getToggleAllRowsSelectedHandler
table.getTopRows
table.getTotalSize
table.getVisibleFlatColumns
table.getVisibleLeafColumns
table.initialState
table.lastPage
table.nextPage
table.options
table.previousPage
table.resetColumnFilters
table.resetColumnOrder
table.resetColumnPinning
table.resetColumnSizing
table.resetColumnVisibility
table.resetExpanded
table.resetGlobalFilter
table.resetGrouping
table.resetHeaderSizeInfo
table.resetPageIndex
table.resetPageSize
table.resetPagination
table.resetRowPinning
table.resetRowSelection
table.resetSorting
table.rows
table.setColumnFilters
table.setColumnOrder
table.setColumnPinning
table.setColumnSizing
table.setColumnSizingInfo
table.setColumnVisibility
table.setExpanded
table.setGlobalFilter
table.setGrouping
table.setPageCount
table.setPageIndex
table.setPageSize
table.setPagination
table.setRowPinning
table.setRowSelection
table.setRowType
table.setSorting
table.setState
table.toggleAllColumnsVisible
table.toggleAllPageRowsSelected
table.toggleAllRowsExpanded
table.toggleAllRowsSelected
table.toggleColumnSorting
```

### Column instance (`column.*`)

```
column._getFacetedMinMaxValues
column._getFacetedRowModel
column._getFacetedUniqueValues
column.accessorFn
column.clearSorting
column.columnDef
column.columns
column.depth
column.filterFn
column.getAfter
column.getAggregationFn
column.getAutoAggregationFn
column.getAutoFilterFn
column.getAutoSortDir
column.getAutoSortingFn
column.getCanFilter
column.getCanGlobalFilter
column.getCanGroup
column.getCanHide
column.getCanMultiSort
column.getCanPin
column.getCanResize
column.getCanSort
column.getFacetedMinMaxValues
column.getFacetedRowModel
column.getFacetedUniqueValues
column.getFilterFn
column.getFilterIndex
column.getFilterValue
column.getFirstSortDir
column.getFlatColumns
column.getGroupedIndex
column.getIndex
column.getIsFiltered
column.getIsFirstColumn
column.getIsGrouped
column.getIsLastColumn
column.getIsPinned
column.getIsResizing
column.getIsSorted
column.getIsVisible
column.getLeafColumns
column.getNextSortingOrder
column.getPinnedIndex
column.getSize
column.getSortIndex
column.getSortingFn
column.getStart
column.getToggleGroupingHandler
column.getToggleSortingHandler
column.getToggleVisibilityHandler
column.id
column.parent
column.pin
column.resetSize
column.setFilterValue
column.toggleGrouping
column.toggleSorting
column.toggleVisibility
```

### Row instance (`row.*`)

```
row._getAllVisibleCells
row._groupingValuesCache
row._uniqueValuesCache
row._valuesCache
row.columnFilters
row.columnFiltersMeta
row.depth
row.getAllCells
row.getCanExpand
row.getCanMultiSelect
row.getCanPin
row.getCanSelect
row.getCanSelectSubRows
row.getCenterVisibleCells
row.getGroupingValue
row.getIsAllParentsExpanded
row.getIsAllSubRowsSelected
row.getIsExpanded
row.getIsGrouped
row.getIsPinned
row.getIsSelected
row.getIsSomeSelected
row.getLeafRows
row.getLeftVisibleCells
row.getParentRows
row.getPinnedIndex
row.getRightVisibleCells
row.getToggleExpandedHandler
row.getToggleSelectedHandler
row.getValue
row.getVisibleCells
row.groupingColumnId
row.id
row.index
row.original
row.originalSubRows
row.parentId
row.pin
row.subRows
row.toggleExpanded
row.toggleSelected
row.userId
```

### Header instance (`header.*`)

```
header.colSpan
header.column
header.getLeafHeaders
header.getResizeHandler
header.getSize
header.getStart
header.headerGroup
header.index
header.isPlaceholder
header.rowSpan
header.subHeaders
```

### Cell instance (`cell.*`)

```
cell.column
cell.getContext
cell.getIsAggregated
cell.getIsGrouped
cell.getIsPlaceholder
cell.getValue
cell.renderValue
```

---

## Checklist: public instance surfaces (WIP)

This is the actionable inventory for `HTP-cap-010` / `HTP-base-004`.

Rules:

- Prefer **capability mapping** over method-name mapping.
- “Aligned” means **parity-gated by fixtures** (or a dedicated unit gate for a non-JSONable invariant).
- For handler-returning APIs, we map to **pure state transitions** (`Updater<T>` or `toggled_*` helpers).

### Table instance (public, non-underscore)

**Row model pipeline**

- **Aligned**: `getCoreRowModel` → `Table::core_row_model()` (multiple fixtures, e.g. `demo_process.json`).
- **Aligned**: `getPreFilteredRowModel` / `getFilteredRowModel` → `Table::{pre_filtered_row_model,filtered_row_model}` (`filtering_fns.json`).
- **Aligned**: `getPreSortedRowModel` / `getSortedRowModel` → `Table::{pre_sorted_row_model,sorted_row_model}` (`sorting_fns.json`, `sort_undefined.json`).
- **Aligned**: `getPreExpandedRowModel` / `getExpandedRowModel` → `Table::{pre_expanded_row_model,expanded_row_model}` (`expanding.json`).
- **Aligned**: `getPrePaginationRowModel` / `getPaginationRowModel` / `getRowModel` → `Table::{pre_pagination_row_model,row_model}` (`pagination.json`).
- **Aligned**: `getPreGroupedRowModel` / `getGroupedRowModel` → `Table::{pre_grouped_row_model,grouped_row_model}` (`grouping.json`).

**Columns**

- **Partial**: `getAllColumns/getAllFlatColumns/getAllLeafColumns/getColumn`
  → `Table::{column_tree,ordered_columns,column}` + `CoreModelSnapshot.column_tree/leaf_columns` (`headers_cells.json`).
- **Aligned**: `getVisibleLeafColumns` (+ left/center/right variants)
  → `CoreModelSnapshot.leaf_columns.{visible,left_visible,center_visible,right_visible}` (`headers_cells.json`).
- **Partial**: `getVisibleFlatColumns` (may need a dedicated “flat column” snapshot helper).

**Headers**

- **Aligned**: `getHeaderGroups` (+ left/center/right) → `Table::{header_groups,left_header_groups,center_header_groups,right_header_groups}` (`headers_cells.json`).
- **Aligned**: `getFooterGroups` (+ left/center/right) → `Table::{footer_groups,left_footer_groups,center_footer_groups,right_footer_groups}` (`headers_cells.json`).
- **Aligned**: `getFlatHeaders` (+ left/center/right) → `Table::{flat_headers,left_flat_headers,center_flat_headers,right_flat_headers}` (`headers_cells.json`).
- **Aligned**: `getLeafHeaders` (+ left/center/right) → `Table::{leaf_headers,left_leaf_headers,center_leaf_headers,right_leaf_headers}` (`headers_cells.json`).
- **Aligned**: deeper nesting + hide-branch edge cases are gated by `headers_inventory_deep.json`.

**Sizing**

- **Aligned**: `getTotalSize` (+ left/center/right) → `Table::{total_size,left_total_size,center_total_size,right_total_size}` (`column_sizing.json`).
- **Aligned**: `column.getStart/getAfter` equivalents → `Table::{column_start,column_after}` (`column_sizing.json`).
- **Aligned**: `resetHeaderSizeInfo` → `Table::reset_header_size_info(default_state)` (`column_sizing.json`).

**Visibility/ordering**

- **Aligned**: `getIsAllColumnsVisible/getIsSomeColumnsVisible` → `Table::{is_all_columns_visible,is_some_columns_visible}` (unit gates exist; fixture gating TBD).
- **Partial**: `getToggleAllColumnsVisibilityHandler/toggleAllColumnsVisible`
  → `Table::toggled_all_columns_visible(visible)` (unit gate; fixture gating TBD).
- **Aligned**: `resetColumnVisibility` / `resetColumnOrder` → `Table::{reset_column_visibility,reset_column_order}` (`resets.json`).

**Selection/expanding**

- **Aligned**: `getIsAllRowsSelected/getIsSomeRowsSelected/getIsAllPageRowsSelected/getIsSomePageRowsSelected`
  → `Table::{is_all_rows_selected,is_some_rows_selected,is_all_page_rows_selected,is_some_page_rows_selected}` (`selection.json`, `selection_tree.json`).
- **Partial**: `getToggleAllRowsSelectedHandler/toggleAllRowsSelected`
  → `Table::toggled_all_rows_selected(value)` (selection fixtures cover outcomes; handler mapping is Rust-native).
- **Partial**: `getToggleAllRowsExpandedHandler/toggleAllRowsExpanded`
  → `Table::toggled_all_rows_expanded(value)` (expanding fixtures cover outcomes; handler mapping is Rust-native).

**Pinning**

- **Aligned**: `getTopRows/getCenterRows/getBottomRows`
  → `Table::{top_row_keys,center_row_keys,bottom_row_keys}` + row lookup (`pinning.json`).
- **Aligned**: `getIsSomeRowsPinned` (+ top/bottom) → `Table::is_some_rows_pinned(..)` (`pinning.json`).
- **Aligned**: `resetRowPinning/resetColumnPinning` → `Table::{reset_row_pinning,reset_column_pinning}` (`pinning.json`, `column_pinning.json`).

**State**

- **Partial**: `getState`/`setState`/`setOptions` (JS instance-style API)
  → Rust-native “pure table”: rebuild `Table` from `TableState` + `TableOptions` (gated indirectly by fixtures).
- **Aligned**: reset surfaces (`resetSorting`, `resetColumnFilters`, `resetGlobalFilter`, `resetGrouping`, `resetPagination`, `resetRowSelection`, ...)
  → `Table::reset_*` (feature fixtures + `resets.json`).

### Column instance (public, non-underscore)

Fret does not expose a first-class `Column` instance object yet. Capabilities map to:

- `ColumnDef` static config (`enable_*`, sizing bounds, fn specs), plus
- `Table::*` helper surfaces (can-* gates, pin state, sizing, filtering/sorting helper outputs),
- and JSON fixtures that lock the observable outcomes.

Initial mapping (WIP):

- **Aligned**: `getCanFilter/getFilterValue/getIsFiltered/getFilterIndex/getCanGlobalFilter`
  → `Table::{column_can_filter,column_filter_value,column_is_filtered,column_filter_index,column_can_global_filter}` (`filtering_fns.json`).
- **Aligned**: `getCanHide/getIsVisible` → `Table::{column_can_hide,is_column_visible}` (`visibility_ordering.json`).
- **Aligned**: `getCanPin/getIsPinned/getPinnedIndex` → `Table::{column_can_pin,column_pin_position,column_pinned_index}` (`column_pinning.json`).
- **Aligned**: `getCanResize/getIsResizing/getSize/getStart/getAfter`
  → `Table::{column_can_resize,is_column_resizing,column_size,column_start,column_after}` (`column_sizing.json`).
- **Partial**: sorting instance surfaces (`getCanSort/getIsSorted/getSortIndex/getNextSortingOrder/getToggleSortingHandler/toggleSorting`)
  → `sorting.rs` helpers (`toggle_sorting_tanstack`, `toggle_sorting_handler_tanstack`) + `TableState.sorting` (needs an explicit engine-owned helper surface and fixture gate for the “handler/policy” path).
- **Partial**: grouping instance surfaces (`getCanGroup/getIsGrouped/getGroupedIndex/getToggleGroupingHandler/toggleGrouping`)
  → `grouping.rs` helpers + `TableState.grouping` (fixture-gated for outcomes, but no consolidated “column instance” helper surface yet).
- **Partial**: faceting instance surfaces (`getFaceted*`) are supported via `Table::{faceted_*}` (fixture-gated), but we do not expose them as `Column` instance methods.

### Row instance (public, non-underscore)

Fret does not expose a first-class `Row` instance object yet. Capabilities map to:

- `RowModel` snapshots + lookup (`row_by_key`, `row_by_id`), plus
- `Table::*` row helpers (selection/expanding/pinning) and cell snapshots.

Initial mapping (WIP):

- **Aligned**: `getAllCells/getVisibleCells/getLeft/Center/RightVisibleCells`
  → `Table::row_cells(row_key)` (`headers_cells.json`).
- **Aligned**: selection/expanding/pinning booleans + transitions (`getIsSelected/toggleSelected`, `getIsExpanded/toggleExpanded`, `getIsPinned/pin`)
  → `Table::{row_is_selected,row_selection_updater,row_is_expanded_for_row,row_expanding_updater,row_is_pinned,row_pinning_updater}` (feature fixtures).
- **Aligned**: `row.columnFilters` / `row.columnFiltersMeta` → `Table::row_filter_state_snapshot()` (`filtering_meta.json`).
- **Partial**: `getValue/renderValue/getUniqueValues` instance-style value accessors (we expose the behaviors via column defs + aggregation/fallback gates).

### Header instance (public, non-underscore)

- **Aligned**: `colSpan/rowSpan/isPlaceholder/placeholderId/subHeaders` equivalents
  → `HeaderSnapshot` fields (`headers_cells.json`, `headers_inventory_deep.json`).
- **Partial**: `getResizeHandler` instance surface
  → modeled via `Table::{started_column_resize,dragged_column_resize,ended_column_resize}` + `header_size/header_start` snapshots (policy helper surface may be needed).

### Cell instance (public, non-underscore)

- **Aligned**: `getIsGrouped/getIsPlaceholder/getIsAggregated`
  → `CellSnapshot.{is_grouped,is_placeholder,is_aggregated}` (`grouping_cells.json` parity).
- **Partial**: `getValue/renderValue` (value extraction is Rust-native via column defs; fallback behavior is parity-gated by `render_fallback.json`).
