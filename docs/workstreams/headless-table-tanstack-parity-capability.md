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
| `row.pin(position, includeLeafRows, includeParentRows)` | `Table::row_pinning_updater(..)` / `row_pinning_updater_by_id(..)` | Partial | tree include-leaf/include-parent is gated by `pinning_tree.json`; grouped coverage remains TODO |
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
| `table.getLeft/Center/RightLeafColumns()` | `split_pinned_columns(..)` over visible ordered leaf columns | Partial | derived via headless state; needs explicit inventory/gate beyond current fixtures |
| `setColumnPinning/resetColumnPinning` | `Table::reset_column_pinning(..)` + updater surfaces | Aligned | `column_pinning.json` |

---

## Open inventory work (next)

This inventory is intentionally incomplete. Next expansions (tracked in `HTP-cap-010` / `HTP-base-004`):

- Sorting/filtering/grouping/expanding/selection/pagination feature-specific instance/row/column APIs.
- Header/footer/flat/leaf header inventories under visibility + pinning + nested columns.
- Column sizing instance APIs (start/after offsets, resize handlers, `columnSizingInfo` fields).

