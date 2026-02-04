Status: Active (workstream tracker; keep updated as parity gates land)

This document tracks executable TODOs for the TanStack Table v8 `table-core` parity workstream.

- Narrative plan: `docs/workstreams/headless-table-tanstack-parity.md`
- Related ADR: `docs/adr/0101-headless-table-engine.md`
- Fret engine: `ecosystem/fret-ui-headless/src/table/`

Tracking format:

- ID: `HTP-{area}-{nnn}`
- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked

---

## Upstream inventory (keep honest)

Option keys referenced by upstream `table-core` feature implementations (as a quick completeness
check). Source: `repo-ref/table/packages/table-core/src/features/*.ts`.

- Sorting (`RowSorting.ts`): `enableMultiRemove`, `enableMultiSort`, `enableSorting`,
  `enableSortingRemoval`, `getSortedRowModel`, `isMultiSortEvent`, `manualSorting`,
  `maxMultiSortColCount`, `onSortingChange`, `sortDescFirst`, `sortingFns`.
- Column filtering (`ColumnFiltering.ts`): `enableColumnFilters`, `enableFilters`, `filterFns`,
  `getFilteredRowModel`, `manualFiltering`, `onColumnFiltersChange`.
- Global filtering (`GlobalFiltering.ts`): `enableFilters`, `enableGlobalFilter`, `filterFns`,
  `getColumnCanGlobalFilter`, `onGlobalFilterChange`.
- Column sizing (`ColumnSizing.ts`): `columnResizeDirection`, `columnResizeMode`,
  `enableColumnResizing`, `onColumnSizingChange`, `onColumnSizingInfoChange`.
- Pagination (`RowPagination.ts`): `autoResetAll`, `autoResetPageIndex`, `getPaginationRowModel`,
  `manualPagination`, `onPaginationChange`, `pageCount`, `rowCount`.
- Expanding (`RowExpanding.ts`): `autoResetAll`, `autoResetExpanded`, `enableExpanding`,
  `getExpandedRowModel`, `getIsRowExpanded`, `getRowCanExpand`, `manualExpanding`, `onExpandedChange`.
- Grouping (`ColumnGrouping.ts`): `aggregationFns`, `enableGrouping`, `getGroupedRowModel`,
  `manualGrouping`, `onGroupingChange`, `renderFallbackValue`, `groupedColumnMode`.
- Column pinning (`ColumnPinning.ts`): `enableColumnPinning`, `enablePinning`, `onColumnPinningChange`.
- Column ordering (`ColumnOrdering.ts`): `groupedColumnMode`, `onColumnOrderChange`.
- Column visibility (`ColumnVisibility.ts`): `enableHiding`, `onColumnVisibilityChange`.
- Row selection (`RowSelection.ts`): `enableGroupingRowSelection`, `enableMultiRowSelection`,
  `enableRowSelection`, `enableSubRowSelection`, `onRowSelectionChange`.
- Row pinning (`RowPinning.ts`): `keepPinnedRows`, `onRowPinningChange`.
- Faceting (`ColumnFaceting.ts` / `GlobalFaceting.ts`): `getFacetedRowModel`,
  `getFacetedUniqueValues`, `getFacetedMinMaxValues` (+ `manualFiltering` interaction).

ColumnDef keys referenced by upstream feature implementations:

- Sorting (`RowSorting.ts`): `sortingFn`, `enableSorting`, `enableMultiSort`, `sortDescFirst`.
- Filtering (`ColumnFiltering.ts`): `filterFn`, `enableColumnFilter`.
- Global filtering (`GlobalFiltering.ts`): `enableGlobalFilter`.
- Column sizing (`ColumnSizing.ts`): `size`, `minSize`, `maxSize`, `enableResizing`.
- Grouping (`ColumnGrouping.ts`): `enableGrouping`, `getGroupingValue`, `aggregationFn`.
- Column pinning (`ColumnPinning.ts`): `enablePinning`.
- Column visibility (`ColumnVisibility.ts`): `enableHiding`.

---

## Baseline (scope + references)

- [x] HTP-base-001 Record the exact upstream `table-core` commit/version used for parity fixtures.
  - Evidence: a short “version stamp” section in `docs/workstreams/headless-table-tanstack-parity.md`.
- [x] HTP-base-002 Add a minimal “parity fixture schema” doc section (JSON layout for core outputs).
  - Target: stable enough to survive refactors.
- [x] HTP-base-003 Add a “feature-by-feature parity matrix” section (one row per upstream feature file).
  - Include: current status, blocking gaps, and evidence anchors (tests/fixtures).
  - Evidence: `docs/workstreams/headless-table-tanstack-parity.md`.
- [ ] HTP-base-004 Extend the upstream inventory to include non-option parity surfaces:
  - column/header/cell/row method inventories (where feasible),
  - state “reset” and “auto-reset” behavior inventory.

---

## M1 — Core types (columns/headers/rows/cells)

- [~] HTP-core-010 Add TanStack-like column tree representation (nested columns).
  - Done (scaffolding): grouped column defs via `ColumnDef::columns(...)` and leaf flattening.
    - Evidence: `ecosystem/fret-ui-headless/src/table/column.rs` (`ColumnDef.columns`)
    - Evidence: `ecosystem/fret-ui-headless/src/table/row_model.rs` (stores `column_tree` + leaf flatten)
- [~] HTP-core-020 Implement header group generation (including placeholder headers).
  - Done (parity-gated): `getHeaderGroups` + pin-family variants + placeholder headers.
    - Evidence: `ecosystem/fret-ui-headless/src/table/headers.rs`
    - Parity gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_headers_cells_parity.rs`
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/headers_cells.json`
  - Remaining: expand fixture coverage for deeper nesting and visibility interactions.
- [~] HTP-core-030 Implement cell model generation (row × leaf columns) with stable IDs.
  - Done (parity-gated): TanStack-style `${rowId}_${columnId}` ids for all/visible/left/center/right.
    - Evidence: `ecosystem/fret-ui-headless/src/table/cells.rs`
    - Parity gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_headers_cells_parity.rs`
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/headers_cells.json`
- [~] HTP-core-040 Define a stable, JSON-serializable “core model” snapshot (columns/headers/rows/cells).
  - Done (parity-gated): initial core snapshot schema (column tree + leaf sets + header groups + row ids + cell ids).
    - Evidence: `ecosystem/fret-ui-headless/src/table/core_model.rs`
    - Evidence: `ecosystem/fret-ui-headless/src/table/row_model.rs` (`Table::core_model_snapshot`)
    - Parity gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_headers_cells_parity.rs` (expects `core_model`)
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/headers_cells.json`
  - Remaining: broaden schema to include full column/header/cell inventories and cover deeper nesting + visibility edge cases.

---

## M2 — State-shape parity + reset semantics

- [ ] HTP-state-010 Define TanStack-compatible JSON schema for:
  - sorting, columnFilters, globalFilter, pagination, grouping, expanded, rowSelection,
    columnVisibility, columnOrder, columnSizing, columnSizingInfo, columnPinning, rowPinning.
- [ ] HTP-state-011 Specify normalization vs lossless rules for the JSON surface:
  - which keys may be omitted vs must be present,
  - how TanStack merges defaults (and what we must preserve to avoid semantic drift),
  - canonical ordering rules for stable fixtures (maps/arrays).
- [~] HTP-state-020 Implement round-trip conversions (Rust ↔ TanStack JSON) without loss.
  - Done (partial): JSON ↔ `TableState` conversions for a growing subset of TanStack state keys, plus a round-trip parity gate.
    - Evidence: `ecosystem/fret-ui-headless/src/table/tanstack_state.rs` (`TanStackTableState::{to_table_state,from_table_state}`)
    - Parity gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_state_roundtrip_parity.rs`
  - Covered (as of current gates): sorting, columnFilters, globalFilter (any), pagination, rowSelection,
    grouping, expanded, rowPinning, columnPinning, columnOrder, columnVisibility, columnSizing, columnSizingInfo.
  - Remaining: lossless “presence” (omitted vs explicit default) semantics, and cross-feature interactions that require
    additional option gates/behavior parity.
- [x] HTP-state-021 Add fixtures that assert state-shape parity for:
  - grouping, expanded, rowPinning, and cross-feature interactions (e.g. pinning + sizing + visibility).
  - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/state_shapes.json`
- [ ] HTP-state-030 Implement “resetX(defaultState?)” semantics where TanStack exposes them.

---

## M3 — Filtering parity (typed values + meta)

- [x] HTP-filt-010 Generalize filter values beyond `Arc<str>` (TanStack `unknown` equivalent).
  - Constraint: keep a cheap “string fast-path” for common UI filters.
  - Evidence: `ecosystem/fret-ui-headless/src/table/filtering.rs` (`ColumnFilter.value: serde_json::Value`)
- [~] HTP-filt-020 Implement option gates: `enableFilters`, `enableColumnFilters`, `enableGlobalFilter`.
  - Done (parity-gated): `enableFilters` + `enableGlobalFilter` gate global filtering application in `filtered_row_model`.
    - Evidence: `ecosystem/fret-ui-headless/tests/tanstack_v8_filtering_fns_parity.rs`
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/filtering_fns.json`
  - Remaining: wire gates into “can filter”/UI surfaces and controlled state hooks.
- [x] HTP-filt-030 Implement filterFn registry parity (`filterFns`):
  - built-ins, custom, and `auto` selection based on first known value.
  - Evidence: `ecosystem/fret-ui-headless/tests/tanstack_v8_filtering_fns_parity.rs`
  - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/filtering_fns.json`
- [x] HTP-filt-040 Implement `resolveFilterValue` and `autoRemove` semantics.
  - Evidence: `ecosystem/fret-ui-headless/tests/tanstack_v8_filtering_fns_parity.rs`
  - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/filtering_fns.json`
- [ ] HTP-filt-050 Add `maxLeafRowFilterDepth` semantics.
- [ ] HTP-filt-060 Track per-row filter pass/fail map and optional filter meta (parity-gated).
- [ ] HTP-filt-070 Align “manual filtering” semantics:
  - `manualFiltering` (and `getFilteredRowModel` override) behavior matches upstream.
- [x] HTP-filt-080 Align “global filtering can-apply” semantics:
  - `getColumnCanGlobalFilter` default behavior and override hooks.
  - Evidence: `ecosystem/fret-ui-headless/tests/tanstack_v8_filtering_fns_parity.rs`
  - Fixture snapshot: `filtering_fns_global_filter_default_excludes_bool` in `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/filtering_fns.json`
  - Hook surface: `ecosystem/fret-ui-headless/src/table/row_model.rs` (`TableBuilder::get_column_can_global_filter`)

---

## M3 — Sorting parity (multi-sort + edge semantics)

- [x] HTP-sort-010 Implement `sortUndefined` semantics (`false | -1 | 1 | first | last`).
  - Done (parity-gated): `first | last | -1 | 1`.
    - Evidence: `ecosystem/fret-ui-headless/tests/tanstack_v8_sort_undefined_parity.rs`
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/sort_undefined.json`
  - Done (parity-gated): `false` (disables undefined pre-pass ordering; leaves behavior to sortingFn).
    - Evidence: `ecosystem/fret-ui-headless/tests/tanstack_v8_sort_undefined_parity.rs`
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/sort_undefined.json`
- [x] HTP-sort-020 Implement `invertSorting` behavior.
- [x] HTP-sort-021 Implement `sortDescFirst` behavior.
- [~] HTP-sort-030 Implement option gates and transitions:
  - `enableSorting`, `enableMultiSort`, `maxMultiSortColCount`,
  - `enableSortingRemoval`, `enableMultiRemove`.
- [~] HTP-sort-040 Implement sortingFn registry parity (`sortingFns`) and resolution (`auto` + built-ins + custom).
  - Done (parity-gated): built-in sorting fn keys + `auto` inference + named registry resolution.
    - Evidence: `ecosystem/fret-ui-headless/tests/tanstack_v8_sorting_fns_parity.rs`
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/sorting_fns.json`
  - Done (parity-gated): `getAutoSortDir`-based first-toggle direction inference (string => asc, else desc).
    - Evidence: `ecosystem/fret-ui-headless/tests/tanstack_v8_sorting_fns_parity.rs`
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/sorting_fns.json`
  - Remaining: broader edge-case coverage (mixed types / nullish behaviors).
- [ ] HTP-sort-050 Align “manual sorting” semantics:
  - `manualSorting` (and `getSortedRowModel` override) behavior matches upstream.

---

## M4 — Column sizing/resizing parity

- [ ] HTP-size-010 Implement option gates and transitions:
  - `enableColumnResizing`, `columnResizeMode`, `columnResizeDirection`.
- [~] HTP-size-020 Ensure pinned start/after offsets match TanStack (left/center/right).
  - Done (parity-gated): pinned totals + `getStart`-equivalent offsets.
    - Evidence: `ecosystem/fret-ui-headless/tests/tanstack_v8_column_sizing_parity.rs`
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/column_sizing.json`
  - Done (parity-gated): `getAfter`-equivalent offsets.
    - Evidence: `ecosystem/fret-ui-headless/tests/tanstack_v8_column_sizing_parity.rs`
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/column_sizing.json`
  - Remaining: header-group sizing offsets.
- [~] HTP-size-030 Ensure size clamp semantics match TanStack (size/min/max + reset behaviors).
  - Done (parity-gated): `columnSizing` overrides + `minSize/maxSize` clamp on totals/offsets.
    - Evidence: `ecosystem/fret-ui-headless/tests/tanstack_v8_column_sizing_parity.rs`
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/column_sizing.json`
  - Remaining: reset behaviors parity (`resetSize` / table-level resets).
- [~] HTP-size-040 Align `columnSizingInfo` fields (isResizing, deltaOffset, etc.) and transitions.
  - Done (parity-gated): basic resize lifecycle + `onChange` vs `onEnd` write timing (LTR).
    - Evidence: `ecosystem/fret-ui-headless/tests/tanstack_v8_column_sizing_parity.rs`
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/column_sizing.json`
    - Snapshots: `colsize_resize_on_change_move_updates`, `colsize_resize_on_change_end_resets`, `colsize_resize_on_end_move_no_sizing`, `colsize_resize_on_end_end_writes`
  - Done (parity-gated): RTL direction (`columnResizeDirection=rtl`) flips delta sign and keeps sizing consistent.
    - Evidence: `ecosystem/fret-ui-headless/tests/tanstack_v8_column_sizing_parity.rs`
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/column_sizing.json`
    - Snapshot: `colsize_resize_rtl_move_flips`
  - Done (parity-gated): group-header `columnSizingStart` fan-out (multiple leaf headers + group entry) matches TanStack.
    - Evidence: `ecosystem/fret-ui-headless/tests/tanstack_v8_column_resizing_group_headers_parity.rs`
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/column_resizing_group_headers.json`
- [ ] HTP-size-050 Align “controlled state” hooks:
  - `onColumnSizingChange` and `onColumnSizingInfoChange` equivalence.

---

## M5 — Grouping + aggregation parity

- [~] HTP-grp-010 Implement grouped row model parity (including placeholder/aggregated cell flags).
  - Parity-gated (grouped row model structure + flat rows ordering): `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/grouping.json` +
    `ecosystem/fret-ui-headless/tests/tanstack_v8_grouping_parity.rs`.
- [~] HTP-grp-020 Implement grouped aggregation parity (built-in and custom aggregation fns).
  - Done (parity-gated, u64 built-ins + TanStack `auto`→`sum`): `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/grouping.json` +
    `ecosystem/fret-ui-headless/tests/tanstack_v8_grouping_parity.rs`.
  - Remaining: custom aggregation fn registry + non-u64 aggregations (`extent`/`median`/`unique`/etc).
- [~] HTP-grp-030 Implement grouped sorting parity (group rows ordering + child ordering).
  - Done (parity-gated for 1-column and 2-column grouping, single sort spec): `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/grouping.json` +
    `ecosystem/fret-ui-headless/tests/tanstack_v8_grouping_parity.rs`.
  - Evidence: `ecosystem/fret-ui-headless/src/table/grouped_sorting.rs`.
  - Remaining: multi-column sort precedence + non-u64 sort keys + fully integrating sorted grouped output into the main row model pipeline.
- [x] HTP-grp-040 Align option gates and hooks:
  - Parity-gated: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/grouping.json` +
    `ecosystem/fret-ui-headless/tests/tanstack_v8_grouping_parity.rs`.
  - Covered surfaces:
    - `enableGrouping` affects `getCanGroup`/`getToggleGroupingHandler` gating, but does not prevent `toggleGrouping()` (matches TanStack).
    - `manualGrouping` bypasses grouped row model computation.
    - `onGroupingChange(updater)` controlled-state semantics (fixtures assert `next_state.grouping`).
    - `getGroupedRowModel` override (fixture-only marker `__getGroupedRowModel=pre_grouped`).
- [ ] HTP-grp-050 Align `groupedColumnMode` behavior and column ordering interactions.
- [x] HTP-grp-050 Align `groupedColumnMode` behavior and column ordering interactions.
  - Parity-gated via header/cell + core-model snapshots:
    - Evidence: `ecosystem/fret-ui-headless/tests/tanstack_v8_headers_cells_parity.rs`
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/headers_cells.json`
    - Snapshots: `headers_cells_grouped_column_mode_*`
- [x] HTP-grp-060 Align `aggregationFns` registry and `renderFallbackValue` behavior.
  - Parity-gated:
    - `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/grouping_aggregation_fns.json` +
      `ecosystem/fret-ui-headless/tests/tanstack_v8_grouping_aggregation_fns_parity.rs`
    - `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/render_fallback.json` +
      `ecosystem/fret-ui-headless/tests/tanstack_v8_render_fallback_parity.rs`
  - Evidence: `ecosystem/fret-ui-headless/src/table/aggregation_fns.rs`
  - Evidence: `ecosystem/fret-ui-headless/src/table/row_model.rs` (`Table::grouped_aggregations_any`, `Table::cell_render_value`)

---

## M6 — Pinning/expanding/selection/pagination interactions parity

- [x] HTP-rowpin-010 Align `keepPinnedRows` behavior and its interactions with filtering/sorting/pagination.
  - Parity-gated: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/pinning.json` +
    `ecosystem/fret-ui-headless/tests/tanstack_v8_pinning_parity.rs`.
- [x] HTP-rowpin-020 Align `onRowPinningChange` (controlled state hook) behavior.
  - Parity-gated (state transition outcomes): `pinRow` action snapshots in
    `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/pinning.json`,
    `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/pinning_tree.json`.
- [~] HTP-expand-010 Align expanded state shape (`true | Record<RowId, boolean>`) and behaviors.
  - In progress (parity gate added): expanded state transitions and row model outputs under `paginateExpandedRows` true/false.
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/expanding.json`
    - Parity gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_expanding_parity.rs` (currently failing; see HTP-page-030).
- [ ] HTP-expand-020 Align option gates and hooks:
  - `enableExpanding`, `manualExpanding`, `onExpandedChange`, `getExpandedRowModel` override.
- [ ] HTP-expand-030 Align `autoResetExpanded` / `autoResetAll` behaviors.
- [ ] HTP-expand-040 Align row capability hooks:
  - `getRowCanExpand`, `getIsRowExpanded` default behavior + overrides.
- [ ] HTP-page-010 Align pagination option gates and hooks:
  - `manualPagination`, `pageCount`, `rowCount`, `onPaginationChange`, `getPaginationRowModel` override.
- [ ] HTP-page-020 Align `autoResetPageIndex` / `autoResetAll` behaviors.
- [ ] HTP-page-030 Align `paginateExpandedRows` interactions with expansion and page bounds.
  - Note: TanStack `getPaginationRowModel` can yield duplicated `flatRows` entries when `paginateExpandedRows=false`.
- [~] HTP-sel-010 Align selection state shape and semantics (including sub-row selection defaults).
  - Done (parity-gated): `getSelectedRowModel` / `getFilteredSelectedRowModel` / `getGroupedSelectedRowModel` equivalents,
    plus basic toggle behaviors for flat rows (including `enableMultiRowSelection=false` clearing semantics).
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/selection.json`
    - Parity gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_selection_parity.rs`
  - Remaining: nested sub-row selection defaults and `getIsSomeSelected`/`getIsAllSubRowsSelected` can-select semantics parity.
- [~] HTP-sel-020 Align option gates and hooks:
  - `enableRowSelection`, `enableMultiRowSelection`, `enableSubRowSelection`, `enableGroupingRowSelection`,
  - `onRowSelectionChange`.
  - Done (partial): table-level boolean gates (`enableRowSelection`, `enableMultiRowSelection`, `enableSubRowSelection`) are parsed from TanStack options and applied by state transitions.
    - Evidence: `ecosystem/fret-ui-headless/src/table/tanstack_options.rs`
    - Evidence: `ecosystem/fret-ui-headless/src/table/row_selection.rs`
  - Remaining: per-row function options + controlled `onRowSelectionChange` parity.
- [x] HTP-colpin-010 Align column pinning option gates and hooks:
  - `enablePinning`, `enableColumnPinning`, `onColumnPinningChange`.
  - Parity-gated (option gates + state transition outcomes): `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/column_pinning.json` +
    `ecosystem/fret-ui-headless/tests/tanstack_v8_column_pinning_parity.rs`.
- [~] HTP-colvis-010 Align column visibility option gates and hooks:
  - `enableHiding`, `onColumnVisibilityChange`.
  - Parity-gated (state transition outcomes + derived visible leaf order): `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/visibility_ordering.json` +
    `ecosystem/fret-ui-headless/tests/tanstack_v8_visibility_ordering_parity.rs`.
- [~] HTP-colord-010 Align column ordering hook:
  - `onColumnOrderChange` (state transition outcomes + derived leaf order).
  - Parity-gated: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/visibility_ordering.json` +
    `ecosystem/fret-ui-headless/tests/tanstack_v8_visibility_ordering_parity.rs`.
  - Remaining: `groupedColumnMode` interactions (covered by future `grouping` gates).

---

## M7 — Engine memoization parity + perf gates

- [ ] HTP-memo-010 Introduce dependency-driven memoization for derived models (TanStack-style).
- [ ] HTP-memo-020 Provide an integration pattern for “rebuild each frame” while retaining memo cache.
  - Candidate designs:
    - external cache passed into a pure “compute” API, or
    - persistent `TableInstance` with `update_state`/`update_data_revision`.
- [ ] HTP-perf-010 Add a minimal perf regression gate for large datasets (engine-only).

---

## M8 — Parity harness (fixtures)

- [x] HTP-fixt-010 Add a Node-based fixture generator that runs upstream `table-core` and emits JSON.
  - Input: deterministic datasets + deterministic option/state transitions.
  - Output: fixtures committed under `ecosystem/fret-ui-headless/tests/fixtures/` (or equivalent).
- [~] HTP-fixt-020 Add Rust tests that load fixtures and assert parity on:
  - core row model output,
  - filtered/sorted/grouped/expanded/paginated models,
  - selection/pinning interactions.

### Fixture coverage matrix (keep this in sync)

Each row is a “parity gate”: a committed TanStack fixture + at least one Rust test that asserts the
fixture outcomes.

| Fixture (JSON) | `case_id` | Upstream feature(s) covered | Rust parity gate | Status |
| --- | --- | --- | --- | --- |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/demo_process.json` | `demo_process` | `ColumnFiltering`, `RowSorting`, `RowPagination` (basic), option/state transition scaffolding | `ecosystem/fret-ui-headless/tests/tanstack_v8_parity.rs` | Partial |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/sort_undefined.json` | `sort_undefined` | `RowSorting` (`sortUndefined`: `first/last/-1/1`) | `ecosystem/fret-ui-headless/tests/tanstack_v8_sort_undefined_parity.rs` | Partial |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/column_sizing.json` | `column_sizing` | `ColumnSizing` (totals, start/after offsets, clamp, resize lifecycle + `columnSizingInfo`, RTL delta sign flip) | `ecosystem/fret-ui-headless/tests/tanstack_v8_column_sizing_parity.rs` | Partial |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/column_resizing_group_headers.json` | `column_resizing_group_headers` | `ColumnSizing` (group header resize fan-out + group entry in `columnSizingStart`) | `ecosystem/fret-ui-headless/tests/tanstack_v8_column_resizing_group_headers_parity.rs` | Partial |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/sorting_fns.json` | `sorting_fns` | `RowSorting` (sortingFn resolution: `auto` + built-ins + registry/custom) | `ecosystem/fret-ui-headless/tests/tanstack_v8_sorting_fns_parity.rs` | Partial |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/filtering_fns.json` | `filtering_fns` | `ColumnFiltering` / `GlobalFiltering` (`filterFns`, `resolveFilterValue`, `autoRemove`, gates) | `ecosystem/fret-ui-headless/tests/tanstack_v8_filtering_fns_parity.rs` | Partial |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/headers_cells.json` | `headers_cells` | `core/*` (header groups + cell ids, including pinning split families) | `ecosystem/fret-ui-headless/tests/tanstack_v8_headers_cells_parity.rs` | Partial |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/pinning.json` | `pinning` | `RowPinning` (`keepPinnedRows` vs sorting/pagination/filtering) | `ecosystem/fret-ui-headless/tests/tanstack_v8_pinning_parity.rs` | Partial |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/pinning_tree.json` | `pinning_tree` | `RowPinning` (includeLeaf/includeParent + expanded gating) | `ecosystem/fret-ui-headless/tests/tanstack_v8_pinning_tree_parity.rs` | Partial |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/column_pinning.json` | `column_pinning` | `ColumnPinning` (option gates + `pin()` transitions) | `ecosystem/fret-ui-headless/tests/tanstack_v8_column_pinning_parity.rs` | Partial |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/selection.json` | `selection` | `RowSelection` (selected models + toggle semantics for flat rows) | `ecosystem/fret-ui-headless/tests/tanstack_v8_selection_parity.rs` | Partial |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/expanding.json` | `expanding` | `RowExpanding` (expanded row model + pagination interactions) | `ecosystem/fret-ui-headless/tests/tanstack_v8_expanding_parity.rs` | Partial |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/grouping.json` | `grouping` | `ColumnGrouping` (grouped model + flat row ordering) | `ecosystem/fret-ui-headless/tests/tanstack_v8_grouping_parity.rs` | Partial |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/visibility_ordering.json` | `visibility_ordering` | `ColumnVisibility` + `ColumnOrdering` (state transitions + derived leaf column order) | `ecosystem/fret-ui-headless/tests/tanstack_v8_visibility_ordering_parity.rs` | Partial |
| (todo) `.../faceting.json` | `faceting` | `ColumnFaceting` / `GlobalFaceting` | (todo) `.../tanstack_v8_faceting_parity.rs` | Open |
| (todo) `.../auto_reset.json` | `auto_reset` | auto-reset semantics across features (`autoResetAll`, `autoResetPageIndex`, etc.) | (todo) `.../tanstack_v8_auto_reset_parity.rs` | Open |
