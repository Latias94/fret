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
  - Evidence: a short 鈥渧ersion stamp鈥?section in `docs/workstreams/headless-table-tanstack-parity.md`.
- [x] HTP-base-002 Add a minimal 鈥減arity fixture schema鈥?doc section (JSON layout for core outputs).
  - Target: stable enough to survive refactors.
- [x] HTP-base-003 Add a 鈥渇eature-by-feature parity matrix鈥?section (one row per upstream feature file).
  - Include: current status, blocking gaps, and evidence anchors (tests/fixtures).
  - Evidence: `docs/workstreams/headless-table-tanstack-parity.md`.
- [~] HTP-base-004 Extend the upstream inventory to include non-option parity surfaces:
  - column/header/cell/row method inventories (where feasible),
  - state 鈥渞eset鈥?and 鈥渁uto-reset鈥?behavior inventory.
  - In progress: core + pinning inventories are tracked in
    `docs/workstreams/headless-table-tanstack-parity-capability.md`.
  - Update: added a raw upstream instance member dump (table/column/row/header/cell) as an appendix in
    `docs/workstreams/headless-table-tanstack-parity-capability.md`.
  - Next: mark public surfaces vs internals, and record which internals are observable obligations
    (memo/queue/auto-reset) even if we do not mirror the internal mechanism.

---

## Next execution plan (functional parity first)

- Step 1: Close the non-option capability inventory (`HTP-cap-010`, `HTP-base-004`) so consumers don't drift.
- Step 2: Extend core snapshot introspection so UI consumers stop re-implementing traversal/policy logic
  (`HTP-core-040` remaining scope).
- Step 3: Lift the memo strategy across the full derived model pipeline (TanStack-style deps) (`HTP-memo-010` remaining).

## Functional parity gap snapshot (must not be weaker than TanStack)

P0 (core behavior parity, highest user-visible risk):

- Cleared for current grouping scope: `HTP-grp-010` + `HTP-grp-030` are fixture-gated.
- Remaining (still Partial in matrix): sorting edge-case coverage, pagination auto-reset trigger breadth,
  and `columnSizingInfo` inventory + cross-feature interactions.

P1 (capability breadth parity):

- Cleared: `HTP-id-014/015`, `HTP-filt-090/100`, `HTP-state-020` are fixture-gated.
- Remaining: `HTP-core-040` core snapshot introspection breadth (so UI consumers do not drift by
  re-implementing traversal / inventory helpers).

P2 (engineering guardrails for sustained parity):

- HTP-cap-010 + HTP-base-004: full public API inventory and non-option surface tracking.
- HTP-memo-010 (remaining): lift the memo strategy across the full derived model pipeline (with guardrail tests as we expand).

---

## Next milestone plan (functional parity first)

- Milestone A (UI pinning correctness, done): HTP-ui-colpin-010 closed with retained split alignment + dedicated parity gate.
- Milestone B (grouped pinning semantics, done): `HTP-ui-rowpin-020` + `HTP-rowpin-015` closed with fixture-backed assertions.
- Milestone C (manual pipeline parity, done): `HTP-sort-050` closed with fixture-backed manualSorting/getSortedRowModel override assertions.
- Milestone D (filter depth/meta parity, done): `HTP-filt-050` + `HTP-filt-060` + `HTP-filt-070` are parity-gated.
- Milestone E (grouped row-model pipeline parity, done): `HTP-grp-030` closed with fixture-backed pipeline assertions.
- Milestone F (ID/state hardening, done): close HTP-id-* remaining items and HTP-state-020 lossless semantics.
- Milestone G (guardrails): close HTP-cap-010 and HTP-base-004 (memo/perf rebuild-each-frame guardrails are done).

---

## M0.5 鈥?Capability parity contract (API inventory)

Goal: ensure we are 鈥渘ot weaker than TanStack鈥?by explicitly tracking upstream public API surfaces
(table/row/column/header/cell) and mapping them to Fret equivalents.

- [x] HTP-cap-010 Inventory upstream public APIs and decide the Fret mapping strategy.
  - Evidence target: expand 鈥淐apability Inventory鈥?in `docs/workstreams/headless-table-tanstack-parity.md`
    with an explicit Table/Row/Column/Header/Cell checklist and per-item status.
  - In progress: initial core + pinning mapping is tracked in
    `docs/workstreams/headless-table-tanstack-parity-capability.md`.
  - Update: added a raw upstream instance member inventory appendix (table/column/row/header/cell) in
    `docs/workstreams/headless-table-tanstack-parity-capability.md` to keep the mapping work honest.
  - Next: turn the raw upstream instance-member dump into an explicit checklist (public surfaces first),
    with status + evidence anchors for each capability surface.
    - [x] HTP-cap-011 Table instance public surface checklist (`table.*`, non-underscore).
    - [x] HTP-cap-012 Column instance capability checklist (`column.*`, non-underscore).
    - [x] HTP-cap-013 Row instance capability checklist (`row.*`, non-underscore).
    - [x] HTP-cap-014 Header + cell instance capability checklist (`header.*`, `cell.*`).
    - [x] HTP-cap-015 Identify “policy helpers” that should be engine-owned (sorting/filtering/grouping/resize handlers)
      so UI consumers do not re-implement TanStack policies and drift.
      - [x] Sorting policy helpers are engine-owned and fixture-gated:
        `Table::{sorting_updater_tanstack,sorting_handler_updater_tanstack,toggled_column_sorting_tanstack,toggled_column_sorting_handler_tanstack}`.
        - Evidence: `ecosystem/fret-ui-headless/tests/tanstack_v8_parity.rs`,
          `ecosystem/fret-ui-headless/tests/tanstack_v8_sorting_fns_parity.rs`.
        - UI integration: `ecosystem/fret-ui-kit/src/declarative/table.rs` now uses the headless TanStack policy
          (`toggle_sorting_state_handler_tanstack`) to avoid drift.
      - [x] Filtering helpers are engine-owned:
        - `Table::{column_filters_updater_set_value,global_filter_updater_set_value}`
          (autoRemove semantics are in-engine).
        - Evidence: `ecosystem/fret-ui-headless/tests/tanstack_v8_filtering_fns_parity.rs`.
      - [x] Grouping toggle handler helpers are engine-owned:
        - `Table::{grouping_updater,grouping_handler_updater}`.
        - Evidence: `ecosystem/fret-ui-headless/src/table/row_model.rs` (`grouping_handler_updater`).
      - [x] Column resize interaction helpers are engine-owned:
        - `Table::{started_column_resize,dragged_column_resize,ended_column_resize}`.
        - Evidence: `ecosystem/fret-ui-headless/tests/tanstack_v8_column_sizing_parity.rs`.
  - Source of truth:
    - Core: `repo-ref/table/packages/table-core/src/core/*`
    - Features: `repo-ref/table/packages/table-core/src/features/*.ts`
    - Glue/types: `repo-ref/table/packages/table-core/src/types.ts`
  - Deliverables:
    - A minimal list of 鈥渕ust-have鈥?APIs required by `DataTable` (`fret-ui-shadcn`) and `table_virtualized` (`fret-ui-kit`).
    - A second list of 鈥渃apability parity鈥?APIs that must exist to avoid being weaker than upstream.
  - Update: started a consumer-driven must-have surface section in
    `docs/workstreams/headless-table-tanstack-parity-capability.md` (references `fret-ui-kit` virtualized table usage).
  - Update: documented the current TanStack → Fret mapping strategy in
    `docs/workstreams/headless-table-tanstack-parity-capability.md` (pure models + snapshots + helper surfaces).
  - Update: closed a core “row value” capability gap to reduce consumer drift:
    - `row.getUniqueValues(columnId)` now has an engine-owned surface (`Table::row_unique_values`)
      backed by an optional `ColumnDef::unique_values_by` hook.
    - `ColumnHelper::{accessor,accessor_str}` now stamps a `sort_value_by` “getValue” source so
      core sorting/filtering helpers don’t depend on ad-hoc value wiring.
    - Filtering now resolves its “value source” via a shared getter across `sort_value/value_u64/facet_*`
      to avoid being weaker when consumers only configured faceting or numeric extraction.

- [ ] HTP-cap-030 Refresh the `fret-ui-shadcn` DataTable must-have surface once it is fully wired to the headless engine.
  - Rationale: today, DataTable recipes primarily mutate `TableState` and rely on `fret-ui-kit` for rendering; once that
    integration changes, we should update the consumer-driven must-have list and add any missing helper surfaces.
  - Evidence target: `docs/workstreams/headless-table-tanstack-parity-capability.md` must-have section + a focused smoke gate.
- [x] HTP-cap-020 Add 鈥渃apability smoke鈥?gates (compile-time + runtime).
  - Done (compile-time, smoke): a minimal API-call coverage gate exists.
    - Evidence: `ecosystem/fret-ui-headless/tests/tanstack_v8_capability_smoke.rs`
  - Done (runtime, smoke): RowId-based state resolution and pinning-by-id helpers are covered.
    - Evidence: `ecosystem/fret-ui-headless/src/table/tanstack_state.rs` (`to_table_state_with_row_model`)
    - Evidence: `ecosystem/fret-ui-headless/src/table/row_model.rs` (`row_pinning_updater_by_id`)
  - Note: deeper runtime semantics are validated by dedicated fixture parity gates (not the smoke gate),
    e.g. `tanstack_v8_row_id_lookup_parity.rs`, `tanstack_v8_row_id_state_ops_parity.rs`,
    and `tanstack_v8_pinning_grouped_rows_parity.rs`.
- [x] HTP-id-010 Promote TanStack-style `RowId` to a first-class concept (capability parity).
  - Rationale: TanStack features operate on string row ids (including grouped row ids like `role:1`), and consumers
    can pin/select/expand by those ids. We must be able to express the same, even if we keep `RowKey(u64)` for hot paths.
  - Planned (staged):
    - [x] HTP-id-011 Introduce `RowId` type (likely `Arc<str>`) and plumb through state shapes where required.
      - Done (engine scaffolding): `RowId` and `RowModel.rows_by_id`.
        - Evidence: `ecosystem/fret-ui-headless/src/table/row_model.rs`
    - [x] HTP-id-012 Add `TableBuilder::get_row_id` (TanStack `_getRowId` equivalent) and default behavior.
      - Done: `TableBuilder::get_row_id` is plumbed and the default RowId strategy matches TanStack (index-path).
        - Evidence: `ecosystem/fret-ui-headless/src/table/row_model.rs`
    - [x] HTP-id-013 Store and gate `rows_by_id` (TanStack `rowsById`) for core/pre-pagination/final models.
      - Done (engine scaffolding): `rows_by_id` is carried through the main row model pipeline for leaf rows.
        - Evidence: `ecosystem/fret-ui-headless/src/table/row_model.rs`
      - Done (fixture parity gate): core/pre-pagination/final `rowsById` semantics (custom RowId + grouped ids + lookup parity).
        - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/row_id_lookup.json`
        - Gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_row_id_lookup_parity.rs`
    - [x] HTP-id-014 Make grouped row ids first-class (deterministic string ids matching upstream).
      - Done (partial): grouped rows now carry TanStack-style ids (`col:value` with `>` parent chain),
        and id 鈫?rowKey lookup can resolve grouped ids.
        - Evidence: `ecosystem/fret-ui-headless/src/table/grouping.rs` (`GroupedRow.id`, `GroupedRowModel::row_by_id`)
        - Evidence: `ecosystem/fret-ui-headless/src/table/row_model.rs` (`Table::row_key_for_id` grouped fallback)
        - Gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_capability_smoke.rs` (`*_grouped_row_ids_exist_*`)
      - Done: row partition APIs now have RowId surfaces (`top_row_ids` / `center_row_ids` / `bottom_row_ids`)
        that resolve grouped ids via the grouped row model.
      - Remaining: continue the non-option capability inventory workstream (`HTP-base-004`, `HTP-cap-010`).
    - [x] HTP-id-015 Support pin/select/expand by `RowId` without losing existing `RowKey` fast paths.
      - Done (initial): RowId-aware TanStack JSON import path and pinning-by-id helper.
        - Evidence: `ecosystem/fret-ui-headless/src/table/tanstack_state.rs` (`to_table_state_with_row_model`)
        - Evidence: `ecosystem/fret-ui-headless/src/table/row_model.rs` (`row_pinning_updater_by_id`)
      - Done (leaf rows): selection/expanding by-id updaters are available alongside existing key paths.
        - Evidence: `ecosystem/fret-ui-headless/src/table/row_model.rs` (`row_selection_updater_by_id`, `row_expanding_updater_by_id`)
        - Gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_capability_smoke.rs` (`*_row_id_updaters_cover_selection_and_expanding`)
      - Done (grouped edges): grouped-row selection propagation is parity-gated for select-children/clear/two-level-group cases.
        - Snapshots: `row_id_state_ops_group_selection_select_children_false`, `row_id_state_ops_group_selection_toggle_off`, `row_id_state_ops_nested_group_selection`
      - Done (state export): `TableState` -> TanStack JSON now has row-model-aware RowId export surfaces.
        - Evidence: `ecosystem/fret-ui-headless/src/table/tanstack_state.rs`
          (`TanStackTableState::{from_table_state_with_row_model,from_table_state_with_row_model_and_shape}`)
      - Done (presence + RowId gate): explicit empty/default key presence and string RowId maps are parity-gated.
        - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/state_presence_rowid.json`
        - Gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_state_presence_rowid_parity.rs`
    - [x] HTP-id-016 Extend fixtures to cover id-based lookup and group row operations.
      - Done (smoke): grouped id lookup + pinning-by-id gate exists.
      - Done (smoke): id-keyed selection/expanding updater coverage exists for leaf rows.
      - Done (fixture parity, rowsById + getRow lookup): grouped ids + custom RowId lookup semantics are gated.
        - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/row_id_lookup.json`
        - Gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_row_id_lookup_parity.rs`
      - Done (fixture parity): added dedicated fixture + parity gate for id-keyed selection/expanding/pinning,
        including grouped row ids and nested grouped ids.
        - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/row_id_state_ops.json`
        - Gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_row_id_state_ops_parity.rs`
        - Fixture generator: `tools/tanstack-table-fixtures/extract-fixtures.mts` (`--case row_id_state_ops`)
      - Done (fixture parity, grouped selection edges): `select_children=false`, toggle-off clearing, and nested grouped selection.
      - Done (fixture parity, controlled hooks): grouped RowId no-op hook paths are covered for selection/expanding/pinning.
        - Snapshots: `row_id_state_ops_group_selection_on_row_selection_change_noop`, `row_id_state_ops_group_expanding_on_expanded_change_noop`, `row_id_state_ops_group_pinning_on_row_pinning_change_noop`
      - Done (fixture parity, mixed action sequences): grouped/nested grouped selection+expanding+pinning cross-feature ordering is gated.
        - Snapshots: `row_id_state_ops_group_mixed_select_expand_pin`, `row_id_state_ops_nested_group_mixed_select_expand_pin`, `row_id_state_ops_group_mixed_selection_noop_expand_pin`
  - Note: plain `TanStackTableState::to_table_state` still assumes numeric ids for row-keyed maps
    (`rowSelection` / `expanded` / `rowPinning`). Use the row-model-aware conversion path for string RowId:
    `TanStackTableState::to_table_state_with_row_model` (parity-gated by `tanstack_v8_state_presence_rowid_parity.rs`).

---

## M1 鈥?Core types (columns/headers/rows/cells)

- [x] HTP-core-010 Add TanStack-like column tree representation (nested columns).
  - Done (scaffolding): grouped column defs via `ColumnDef::columns(...)` and leaf flattening.
    - Evidence: `ecosystem/fret-ui-headless/src/table/column.rs` (`ColumnDef.columns`)
    - Evidence: `ecosystem/fret-ui-headless/src/table/row_model.rs` (stores `column_tree` + leaf flatten)
  - Done (parity-gated): column tree / flat column inventory helpers match upstream outcomes for deep nesting + visibility.
    - Fixtures: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/headers_inventory_deep.json`,
      `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/visibility_ordering.json`
    - Gates: `ecosystem/fret-ui-headless/tests/tanstack_v8_headers_inventory_deep_parity.rs`,
      `ecosystem/fret-ui-headless/tests/tanstack_v8_visibility_ordering_parity.rs`
- [x] HTP-core-020 Implement header group generation (including placeholder headers).
  - Done (parity-gated): `getHeaderGroups` + pin-family variants + placeholder headers.
    - Evidence: `ecosystem/fret-ui-headless/src/table/headers.rs`
    - Parity gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_headers_cells_parity.rs`
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/headers_cells.json`
  - Done (deeper nesting + visibility): additional deep header inventory fixture extends coverage for
    deep placeholder trees, leaf/flat/footer inventories, and branch hiding.
    - Parity gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_headers_inventory_deep_parity.rs`
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/headers_inventory_deep.json`
- [x] HTP-core-030 Implement cell model generation (row 脳 leaf columns) with stable IDs.
  - Parity-gated: TanStack-style `${rowId}_${columnId}` ids for all/visible/left/center/right.
    - Evidence: `ecosystem/fret-ui-headless/src/table/cells.rs`
    - Parity gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_headers_cells_parity.rs`
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/headers_cells.json`
- [~] HTP-core-040 Define a stable, JSON-serializable 鈥渃ore model鈥?snapshot (columns/headers/rows/cells).
  - Done (parity-gated): initial core snapshot schema (column tree + leaf sets + header groups + row ids + cell ids).
    - Evidence: `ecosystem/fret-ui-headless/src/table/core_model.rs`
    - Evidence: `ecosystem/fret-ui-headless/src/table/row_model.rs` (`Table::core_model_snapshot`)
  - Update (parity-gated): bump core snapshot to `schema_version: 3` and add `column_capabilities`
    inventory for leaf columns:
    `getCanHide/getCanPin/getIsPinned/getPinnedIndex/getCanResize/getIsVisible`.
    - Gates: `ecosystem/fret-ui-headless/tests/tanstack_v8_headers_cells_parity.rs`,
      `ecosystem/fret-ui-headless/tests/tanstack_v8_headers_inventory_deep_parity.rs`
    - Fixtures: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/headers_cells.json`,
      `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/headers_inventory_deep.json`
  - Update (parity-gated): core snapshot cell inventory now includes grouping flags:
    `cell.getIsGrouped/getIsPlaceholder/getIsAggregated` across `all/visible/left/center/right` splits.
    - Gates: `ecosystem/fret-ui-headless/tests/tanstack_v8_headers_cells_parity.rs`,
      `ecosystem/fret-ui-headless/tests/tanstack_v8_headers_inventory_deep_parity.rs`
    - Fixtures: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/headers_cells.json`,
      `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/headers_inventory_deep.json`
  - Update (parity-gated): add `header_sizing` inventory keyed by header id:
    `header.getSize/getStart` (including pin-family header groups) to avoid UI-side recompute drift.
    - Gates: `ecosystem/fret-ui-headless/tests/tanstack_v8_headers_cells_parity.rs`,
      `ecosystem/fret-ui-headless/tests/tanstack_v8_headers_inventory_deep_parity.rs`
    - Fixtures: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/headers_cells.json`,
      `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/headers_inventory_deep.json`
  - Update (parity-gated): flat-column inventories (`table.getAllFlatColumns/getVisibleFlatColumns`)
    are fixture-asserted so consumers can rely on TanStack’s pre-order DFS flattening semantics.
    - Gates: `ecosystem/fret-ui-headless/tests/tanstack_v8_headers_cells_parity.rs`,
      `ecosystem/fret-ui-headless/tests/tanstack_v8_headers_inventory_deep_parity.rs`,
      `ecosystem/fret-ui-headless/tests/tanstack_v8_visibility_ordering_parity.rs`
    - Fixtures: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/headers_cells.json`,
      `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/headers_inventory_deep.json`,
      `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/visibility_ordering.json`
  - Update (parity-gated): `core_model.flat_columns` is now part of the core snapshot schema (`schema_version: 3`)
    so consumers can rely on stable inventories without recomputing.
  - Update: core snapshot `cells` is keyed by TanStack-style `RowId` strings (not numeric row keys) to match upstream
    `rowsById`/cell inventory semantics and avoid drift when custom `getRowId` is used.
  - Remaining: broaden the schema with more column/header/cell inventories (keep versioning strict) as additional
    UI consumers require them.
- [x] HTP-core-050 Expose header inventories (flat/leaf/footer) with pin-family variants.
  - Covered (TanStack): `getFlatHeaders`, `getLeafHeaders`, `getFooterGroups` and left/center/right variants.
  - Fret mapping: snapshot-friendly header lists + footer groups as reversed header groups.
  - Parity-gated by fixtures:
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/headers_cells.json`
    - Gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_headers_cells_parity.rs`
  - Evidence (engine surfaces): `ecosystem/fret-ui-headless/src/table/row_model.rs`
    (`Table::{flat_headers,leaf_headers,footer_groups}` + pin-family variants).

- [x] HTP-rowtrav-010 Gate row traversal helpers (`row.getParentRows` / `row.getLeafRows`) for nested `subRows`.
  - Parity-gated (ordering semantics): parent chain order + DFS preorder flattening of descendants.
  - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/selection_tree.json` (`row_traversal_detail`).
  - Gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_selection_tree_parity.rs`.
  - Evidence: `ecosystem/fret-ui-headless/src/table/row_model.rs` (`RowModel::{parent_row_ids,leaf_row_ids}`).

- [x] HTP-rowstruct-010 Gate core row structure fields (`id/index/depth/parentId/subRows`).
  - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/selection_tree.json` (`row_structure_detail`).
  - Gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_selection_tree_parity.rs`.
  - Evidence: `ecosystem/fret-ui-headless/src/table/row_model.rs` (`Row` arena fields).

---

## M2 鈥?State-shape parity + reset semantics

- [x] HTP-state-010 Define TanStack-compatible JSON schema for:
  - sorting, columnFilters, globalFilter, pagination, grouping, expanded, rowSelection,
    columnVisibility, columnOrder, columnSizing, columnSizingInfo, columnPinning, rowPinning.
- [x] HTP-state-011 Specify normalization vs lossless rules for the JSON surface:
  - which keys may be omitted vs must be present,
  - how TanStack merges defaults (and what we must preserve to avoid semantic drift),
  - canonical ordering rules for stable fixtures (maps/arrays).
  - Evidence: `docs/workstreams/headless-table-tanstack-parity.md` (state JSON schema + rules)
  - Gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_state_json_semantics_parity.rs`
- [x] HTP-state-020 Implement round-trip conversions (Rust 鈫?TanStack JSON) without loss.
  - Done (partial): JSON 鈫?`TableState` conversions for a growing subset of TanStack state keys, plus a round-trip parity gate.
    - Evidence: `ecosystem/fret-ui-headless/src/table/tanstack_state.rs` (`TanStackTableState::{to_table_state,from_table_state}`)
    - Parity gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_state_roundtrip_parity.rs`
  - Update (RowId + presence): row-model-aware conversions cover string RowId for `rowSelection` / `expanded` / `rowPinning`
    in both directions, and preserve omitted-vs-explicit-default JSON key presence on the covered surfaces.
    - Evidence: `ecosystem/fret-ui-headless/src/table/tanstack_state.rs`
      (`TanStackTableState::{to_table_state_with_row_model,from_table_state_with_shape,from_table_state_with_row_model,from_table_state_with_row_model_and_shape}`)
    - Gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_state_presence_rowid_parity.rs`
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/state_presence_rowid.json`
  - Covered (as of current gates): sorting, columnFilters, globalFilter (any), pagination, rowSelection,
    grouping, expanded, rowPinning, columnPinning, columnOrder, columnVisibility, columnSizing, columnSizingInfo.
  - Legacy limitation note: plain `to_table_state` still assumes numeric row ids for row-keyed maps; string RowId parity is
    now covered by row-model-aware conversions.
  - Remaining: cross-feature interactions that require additional option gates/behavior parity.
- [x] HTP-state-021 Add fixtures that assert state-shape parity for:
  - grouping, expanded, rowPinning, and cross-feature interactions (e.g. pinning + sizing + visibility).
  - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/state_shapes.json`
- [x] HTP-state-030 Implement 鈥渞esetX(defaultState?)鈥?semantics where TanStack exposes them.
  - Done (parity-gated): a dedicated reset parity fixture covering core reset surfaces:
    `resetSorting`, `resetColumnFilters`, `resetGlobalFilter`, `resetGrouping`,
    `resetColumnVisibility`, `resetColumnOrder`, `resetRowSelection`.
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/resets.json`
    - Parity gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_resets_parity.rs`
    - Evidence: `ecosystem/fret-ui-headless/src/table/row_model.rs` (`Table::reset_*`)
  - Covered elsewhere (feature-specific gates):
    - Pagination: `resetPageIndex` / `resetPageSize` / `resetPagination` (`pagination.json`)
    - Expanding: `resetExpanded` (`grouping.json` auto-reset gates + `Table::reset_expanded`)
    - Column sizing: `resetColumnSizing` / `resetHeaderSizeInfo` / `column.resetSize()` (`column_sizing.json`)
    - Pinning: `resetRowPinning` / `resetColumnPinning` (`pinning.json`, `column_pinning.json`)

---

## M3 鈥?Filtering parity (typed values + meta)

- [x] HTP-filt-010 Generalize filter values beyond `Arc<str>` (TanStack `unknown` equivalent).
  - Constraint: keep a cheap 鈥渟tring fast-path鈥?for common UI filters.
  - Evidence: `ecosystem/fret-ui-headless/src/table/filtering.rs` (`ColumnFilter.value: serde_json::Value`)
- [x] HTP-filt-020 Implement option gates: `enableFilters`, `enableColumnFilters`, `enableGlobalFilter`.
  - Done (parity-gated): `enableFilters` + `enableGlobalFilter` gate global filtering application in `filtered_row_model`.
    - Evidence: `ecosystem/fret-ui-headless/tests/tanstack_v8_filtering_fns_parity.rs`
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/filtering_fns.json`
  - Done (parity-gated): `enableColumnFilters=false` disables `getCanFilter`, but does **not** disable
    `getFilteredRowModel` in upstream (it still uses `state.columnFilters` when present); fixtures assert we match.
    - Snapshot: `filtering_fns_column_filters_disabled_when_enable_column_filters_false`
- [x] HTP-filt-030 Implement filterFn registry parity (`filterFns`):
  - built-ins, custom, and `auto` selection based on first known value.
  - Evidence: `ecosystem/fret-ui-headless/tests/tanstack_v8_filtering_fns_parity.rs`
  - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/filtering_fns.json`
- [x] HTP-filt-040 Implement `resolveFilterValue` and `autoRemove` semantics.
  - Evidence: `ecosystem/fret-ui-headless/tests/tanstack_v8_filtering_fns_parity.rs`
  - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/filtering_fns.json`
- [x] HTP-filt-050 Add `maxLeafRowFilterDepth` semantics.
  - Done (parity-gated): root/leaf recursion both honor `maxLeafRowFilterDepth` + `filterFromLeafRows`.
  - Evidence: `ecosystem/fret-ui-headless/src/table/filtering.rs` (`filter_row_model` recursion branches)
  - Evidence: `ecosystem/fret-ui-headless/src/table/filtering.rs` (`root_filter_depth_zero_preserves_unfiltered_subtree`, `leaf_filter_depth_gate_controls_descendant_bubbling`)
- [x] HTP-filt-060 Track per-row filter pass/fail map and optional filter meta (parity-gated).
  - Done: `RowFilterStateSnapshot` + `evaluate_row_filter_state` + table-level `row_filter_state_snapshot()` now track per-row pass/fail and meta containers.
  - Done (parity-gated): custom filter meta callback (`addMeta`-like path) via named filterFns + snapshot assertions.
  - Evidence: `ecosystem/fret-ui-headless/src/table/filtering.rs` (`RowFilterStateSnapshot`, `evaluate_row_filter_state`)
  - Evidence: `ecosystem/fret-ui-headless/src/table/row_model.rs` (`Table::row_filter_state_snapshot`, `TableBuilder::filter_fn_value_with_meta`)
  - Evidence: `ecosystem/fret-ui-headless/tests/tanstack_v8_filtering_meta_parity.rs`
  - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/filtering_meta.json`
- [x] HTP-filt-070 Align manual filtering semantics:
  - `manualFiltering` (and `getFilteredRowModel` override) behavior matches upstream.
  - Evidence: `ecosystem/fret-ui-headless/src/table/row_model.rs` (`Table::filtered_row_model`, `Table::faceted_row_model`)
  - Evidence: `ecosystem/fret-ui-headless/src/table/row_model.rs` (`manual_filtering_skips_filtered_row_model`, `filtered_row_model_override_skips_filtered_and_faceted_row_models`)
- [x] HTP-filt-080 Align 鈥済lobal filtering can-apply鈥?semantics:
  - `getColumnCanGlobalFilter` default behavior and override hooks.
  - Evidence: `ecosystem/fret-ui-headless/tests/tanstack_v8_filtering_fns_parity.rs`
  - Fixture snapshot: `filtering_fns_global_filter_default_excludes_bool` in `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/filtering_fns.json`
  - Hook surface: `ecosystem/fret-ui-headless/src/table/row_model.rs` (`TableBuilder::get_column_can_global_filter`)
- [x] HTP-filt-090 Add TanStack-like filtering helper surfaces (capability parity, consumer-facing).
  - Target: consumers should not re-implement TanStack logic for:
    - `column.getCanFilter()`
    - `column.getFilterValue()`
    - `column.getIsFiltered()`
    - `column.getFilterIndex()`
    - `column.setFilterValue(updater)` (table-scoped helper that returns an updater for `TableState.column_filters`)
    - `column.getCanGlobalFilter()`
  - Done (helper surfaces): `Table::{column_can_filter,column_filter_value,column_is_filtered,column_filter_index,column_can_global_filter}`
    and `Table::column_filters_updater_set_value(..)` exist.
    - Evidence: `ecosystem/fret-ui-headless/src/table/row_model.rs`
    - Gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_capability_smoke.rs`
  - Done (parity-gated): helper surface outcomes are asserted against upstream via `filtering_helpers` snapshots.
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/filtering_fns.json`
    - Gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_filtering_fns_parity.rs`
- [x] HTP-filt-100 Add global filtering helper surface + controlled hook parity.
  - Done (helper surface): `Table::global_filter_updater_set_value(..)` exists and is smoke-gated.
    - Evidence: `ecosystem/fret-ui-headless/src/table/row_model.rs`
    - Gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_capability_smoke.rs`
  - Done (controlled hook parity): fixture marker-driven noop semantics are gated for both
    `onColumnFiltersChange` and `onGlobalFilterChange`.
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/filtering_fns.json`
      (snapshots: `filtering_fns_action_set_column_filter_noop_hook_ignores`,
      `filtering_fns_action_set_global_filter_noop_hook_ignores`)
    - Gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_filtering_fns_parity.rs`
    - Fixture generator: `tools/tanstack-table-fixtures/extract-fixtures.mts`
      (`__onColumnFiltersChange`, `__onGlobalFilterChange`)

---

## M3 鈥?Sorting parity (multi-sort + edge semantics)

- [x] HTP-sort-010 Implement `sortUndefined` semantics (`false | -1 | 1 | first | last`).
  - Done (parity-gated): `first | last | -1 | 1`.
    - Evidence: `ecosystem/fret-ui-headless/tests/tanstack_v8_sort_undefined_parity.rs`
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/sort_undefined.json`
  - Done (parity-gated): `false` (disables undefined pre-pass ordering; leaves behavior to sortingFn).
    - Evidence: `ecosystem/fret-ui-headless/tests/tanstack_v8_sort_undefined_parity.rs`
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/sort_undefined.json`
- [x] HTP-sort-020 Implement `invertSorting` behavior.
- [x] HTP-sort-021 Implement `sortDescFirst` behavior.
- [x] HTP-sort-022 Gate sorting query helper surfaces:
  - `getCanMultiSort`, `getAutoSortDir`, `getFirstSortDir`, `getNextSortingOrder`.
  - Parity-gated via `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/demo_process.json`
    (`sorting_helpers`) + `ecosystem/fret-ui-headless/tests/tanstack_v8_parity.rs`.
  - Evidence: `Table::{column_can_multi_sort,column_auto_sort_dir_desc_tanstack,column_first_sort_dir_desc_tanstack,column_next_sorting_order_desc_tanstack}`.
- [x] HTP-sort-023 Align `column.clearSorting()` semantics.
  - Parity-gated via `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/demo_process.json` +
    `ecosystem/fret-ui-headless/tests/tanstack_v8_parity.rs`.
  - Evidence: `Table::cleared_column_sorting(..)`.
- [x] HTP-sort-030 Implement option gates and transitions:
  - `enableSorting`, `enableMultiSort`, `maxMultiSortColCount`,
  - `enableSortingRemoval`, `enableMultiRemove`.
  - Done (parity-gated): option-driven toggle transitions for direct calls (`toggleSorting`) and UI handler semantics
    (`toggleSortingHandler`) including:
    - `enableSortingRemoval=false` (no “remove” state on 3rd toggle),
    - `maxMultiSortColCount` dropping oldest sorts,
    - `enableMultiSort=false` and column-level `enableMultiSort=false` behavior,
    - handler gating when column/table sorting is disabled.
    - Parity gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_parity.rs`
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/demo_process.json`
- [x] HTP-sort-040 Implement sortingFn registry parity (`sortingFns`) and resolution (`auto` + built-ins + custom).
  - Parity-gated: built-in sorting fn keys + `auto` inference + named registry resolution.
    - Gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_sorting_fns_parity.rs`
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/sorting_fns.json`
  - Done (parity-gated): `getAutoSortDir`-based first-toggle direction inference (string => asc, else desc).
    - Evidence: `ecosystem/fret-ui-headless/tests/tanstack_v8_sorting_fns_parity.rs`
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/sorting_fns.json`
  - Update: added explicit descending and datetime toggle coverage.
    - Snapshots: `sorting_fns_builtin_basic_desc`, `sorting_fns_builtin_datetime_desc`, `sorting_fns_toggle_dt_auto_first`
  - Remaining: broader edge-case coverage (mixed types / nullish behaviors).
- [ ] HTP-sort-060 Expand sorting edge-case fixture coverage (capability parity hardening).
  - Target: ensure we are not weaker than TanStack across mixed-type/nullish datasets and grouped sorting interactions.
  - Evidence target: a dedicated fixture + parity gate (e.g. `sorting_edge_cases.json` +
    `ecosystem/fret-ui-headless/tests/tanstack_v8_sorting_edge_cases_parity.rs`).
- [x] HTP-sort-050 Align manual sorting semantics:
  - Done (parity-gated): `manualSorting=true` returns `pre_sorted` row model.
  - Done (parity-gated): `getSortedRowModel` override to `pre_sorted` matches upstream behavior.
  - Evidence: `ecosystem/fret-ui-headless/tests/tanstack_v8_sorting_manual_parity.rs`
  - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/sorting_manual.json`
  - Hook surface: `ecosystem/fret-ui-headless/src/table/row_model.rs` (`TableBuilder::override_sorted_row_model_pre_sorted`)

---

## M4 鈥?Column sizing/resizing parity

- [x] HTP-size-010 Implement option gates and transitions:
  - `enableColumnResizing`, `columnResizeMode`, `columnResizeDirection`.
  - Parity-gated (resize lifecycle + mode/direction): `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/column_sizing.json` +
    `ecosystem/fret-ui-headless/tests/tanstack_v8_column_sizing_parity.rs`.
  - Covered: `enableColumnResizing=false` yields no-op resize handler effects (snapshot `colsize_enable_column_resizing_false_noops`).
- [x] HTP-size-020 Ensure pinned start/after offsets match TanStack (left/center/right).
  - Done (parity-gated): pinned totals + `getStart`-equivalent offsets.
    - Evidence: `ecosystem/fret-ui-headless/tests/tanstack_v8_column_sizing_parity.rs`
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/column_sizing.json`
  - Done (parity-gated): `getAfter`-equivalent offsets.
    - Evidence: `ecosystem/fret-ui-headless/tests/tanstack_v8_column_sizing_parity.rs`
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/column_sizing.json`
  - Done (parity-gated): header-group `header.getSize()` and `header.getStart()` semantics (including placeholder headers).
    - Evidence: `ecosystem/fret-ui-headless/tests/tanstack_v8_column_resizing_group_headers_parity.rs`
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/column_resizing_group_headers.json`
- [x] HTP-size-030 Ensure size clamp semantics match TanStack (size/min/max + reset behaviors).
  - Done (parity-gated): `columnSizing` overrides + `minSize/maxSize` clamp on totals/offsets.
    - Evidence: `ecosystem/fret-ui-headless/tests/tanstack_v8_column_sizing_parity.rs`
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/column_sizing.json`
  - Done (parity-gated): reset behaviors (`column.resetSize()`, `table.resetColumnSizing()`, `table.resetHeaderSizeInfo()`).
    - Snapshots: `colsize_reset_column_size_removes_override`, `colsize_reset_column_sizing_default_true_clears`,
      `colsize_reset_column_sizing_restores_initial`, `colsize_reset_header_size_info_default_true_clears`.
- [x] HTP-size-040 Align `columnSizingInfo` fields (isResizing, deltaOffset, etc.) and transitions.
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
- [x] HTP-size-050 Align 鈥渃ontrolled state鈥?hooks:
  - `onColumnSizingChange` and `onColumnSizingInfoChange` equivalence.
  - Done (parity-gated): controlled no-op semantics using fixture-only markers:
    - `__onColumnSizingChange="noop"` (sizing stays unchanged) and `__onColumnSizingInfoChange="noop"`
      (info+sizing stay unchanged due to upstream `newColumnSizing` computation placement).
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/column_sizing.json`
    - Parity gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_column_sizing_parity.rs`
- [ ] HTP-size-060 Expand column sizing capability coverage across feature interactions.
  - Target: ensure sizing offsets + `columnSizingInfo` remain correct under:
    - pinning/visibility/ordering changes, and
    - grouped header resize fan-out + manualSizing overrides.
  - Evidence target: extend `column_sizing.json` (or add a new interaction fixture + parity gate).

---

## M5 鈥?Grouping + aggregation parity

- [x] HTP-grp-010 Implement grouped row model parity (including placeholder/aggregated cell flags).
  - Done (parity-gated, grouped row model structure + flat rows ordering): `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/grouping.json` +
    `ecosystem/fret-ui-headless/tests/tanstack_v8_grouping_parity.rs`.
  - Done (parity-gated, grouped/placeholder/aggregated cell inventories in two-level grouping):
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/grouping_cells.json`
    - Gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_grouping_cells_parity.rs`
- [x] HTP-grp-020 Implement grouped aggregation parity (built-in and custom aggregation fns).
  - Done (parity-gated, u64 built-ins + TanStack `auto` -> `sum`): `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/grouping.json` +
    `ecosystem/fret-ui-headless/tests/tanstack_v8_grouping_parity.rs`.
  - Done (parity-gated, non-u64 built-ins + custom registry): `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/grouping_aggregation_fns.json` +
    `ecosystem/fret-ui-headless/tests/tanstack_v8_grouping_aggregation_fns_parity.rs`.
  - Done (sorting integration over aggregation-any values): `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/grouping_sorting_precedence.json` +
    `ecosystem/fret-ui-headless/tests/tanstack_v8_grouping_sorting_precedence_parity.rs`.
- [x] HTP-grp-030 Implement grouped sorting parity (group rows ordering + child ordering).
  - Done (parity-gated for 1-column and 2-column grouping, single sort spec): `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/grouping.json` +
    `ecosystem/fret-ui-headless/tests/tanstack_v8_grouping_parity.rs`.
  - Done (parity-gated): multi-sort precedence + non-u64 aggregated sort keys (`mean` + secondary tie-break) via
    `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/grouping_sorting_precedence.json` +
    `ecosystem/fret-ui-headless/tests/tanstack_v8_grouping_sorting_precedence_parity.rs`.
  - Done (parity-gated): grouped row-model pipeline handoff (`pre_sorted` -> `sorted` -> `expanded`) with sorted grouped output integration.
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/grouping_row_model_pipeline.json`
    - Gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_grouping_row_model_pipeline_parity.rs`
  - Evidence: `ecosystem/fret-ui-headless/src/table/row_model.rs`,
    `ecosystem/fret-ui-headless/src/table/grouped_sorting.rs`.
- [x] HTP-grp-040 Align option gates and hooks:
  - Parity-gated: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/grouping.json` +
    `ecosystem/fret-ui-headless/tests/tanstack_v8_grouping_parity.rs`.
  - Covered surfaces:
    - `enableGrouping` affects `getCanGroup`/`getToggleGroupingHandler` gating, but does not prevent `toggleGrouping()` (matches TanStack).
    - `manualGrouping` bypasses grouped row model computation.
    - `onGroupingChange(updater)` controlled-state semantics (fixtures assert `next_state.grouping`).
    - `getGroupedRowModel` override (fixture-only marker `__getGroupedRowModel=pre_grouped`).
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

## M6 鈥?Pinning/expanding/selection/pagination interactions parity

- [x] HTP-rowpin-010 Align `keepPinnedRows` behavior and its interactions with filtering/sorting/pagination.
  - Parity-gated: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/pinning.json` +
    `ecosystem/fret-ui-headless/tests/tanstack_v8_pinning_parity.rs`.
  - Covered: `row.getPinnedIndex()`-equivalent visible ordering (`row_pinning.pinned_index` in fixture snapshots).
    - Snapshots: `pinning_keep_true_multi_pinned_index_page_0`, `pinning_keep_false_multi_pinned_index_page_0`.
  - Covered: `enableRowPinning: (row) => boolean` predicate (fixture marker `__enableRowPinning=odd_ids`).
  - Hardened gate coverage: pinning fixtures now also assert the 鈥渇ull derived snapshot surface鈥?    (core/filtered/sorted/expanded/paginated models, selection/expanding flags, and column sizing totals + start/after offsets),
    preventing drift that manifests as misaligned widths/offsets in UI consumers.
    - Evidence: `ecosystem/fret-ui-headless/tests/tanstack_v8_pinning_parity.rs`
  - Bugfix: TanStack option defaults are `true` for `keepPinnedRows` and `paginateExpandedRows` when omitted.
    - Evidence: `ecosystem/fret-ui-headless/src/table/tanstack_options.rs`
- [x] HTP-rowpin-015 Gate row pinning × grouping interactions (grouped model + pagination).
  - Parity-gated (grouped pinning + pagination root-row semantics):
    `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/grouping.json` +
    `ecosystem/fret-ui-headless/tests/tanstack_v8_grouping_parity.rs`.
  - Covered: `keepPinnedRows` respects grouping parents’ expansion state (TanStack
    `row.getIsAllParentsExpanded()` behavior) for leaf pinned rows.
    - Evidence: `ecosystem/fret-ui-headless/src/table/row_model.rs` (`Table::pinned_row_keys`).
  - Covered: grouped `row_pinning.center` now follows grouped root ordering + pagination
    (including sorted grouped roots) and is parity-asserted by fixture snapshots.
    - Evidence: `ecosystem/fret-ui-headless/src/table/row_model.rs` (`Table::center_row_keys`),
      `ecosystem/fret-ui-headless/tests/tanstack_v8_grouping_parity.rs` (`row_id_for_key`, row pinning assertions).
  - Update: `row_pinning.can_pin/pin_position/pinned_index` inventories now include grouped row ids
    (e.g. `role:1`) under grouping, and are fixture-gated.
    - Fixtures: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/grouping.json`,
      `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/pinning_grouped_rows.json`
    - Gates: `ecosystem/fret-ui-headless/tests/tanstack_v8_grouping_parity.rs`,
      `ecosystem/fret-ui-headless/tests/tanstack_v8_pinning_grouped_rows_parity.rs`
- [x] HTP-rowpin-020 Align `onRowPinningChange` (controlled state hook) behavior.
  - Parity-gated (state transition outcomes): `pinRow` action snapshots in
    `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/pinning.json`,
    `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/pinning_tree.json`.
  - Covered: controlled-hook noop semantics (`onRowPinningChange` ignores updater).
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/pinning.json`
      (snapshot: `pinning_action_on_row_pinning_change_noop_ignores`)
    - Evidence: `tools/tanstack-table-fixtures/extract-fixtures.mts` (`__onRowPinningChange`)
    - Parity gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_pinning_parity.rs`
- [x] HTP-rowpin-030 Align `resetRowPinning(defaultState?)` semantics.
  - Parity-gated:
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/pinning.json`
      (snapshots: `pinning_action_reset_row_pinning_restores_initial`,
      `pinning_action_reset_row_pinning_default_true_clears`)
    - Parity gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_pinning_parity.rs`
  - Evidence: `ecosystem/fret-ui-headless/src/table/row_model.rs` (`Table::reset_row_pinning`)
- [x] HTP-rowpin-040 Gate `row.pin(position, includeLeafRows, includeParentRows)` semantics.
  - Tree-only include-leaf/include-parent semantics are parity-gated via `pinning_tree.json`.
  - Grouped row model include-leaf/include-parent semantics are parity-gated via `pinning_grouped_rows.json`.
    - Snapshots:
      - `pinning_grouped_rows_action_pin_group_role_1_top_include_leaf_rows`
      - `pinning_grouped_rows_action_pin_leaf_1_top_include_parent_rows`
- [x] HTP-rowpin-050 Gate pinning grouped rows by id (e.g. `role:1`) under grouping.
  - Parity-gated:
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/pinning_grouped_rows.json`
    - Gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_pinning_grouped_rows_parity.rs`
  - Covered: `getTopRows`/`getCenterRows`/`getBottomRows` ordering under grouping + pagination when pinning
    grouped root rows by id.
- [x] HTP-expand-010 Align expanded state shape (`true | Record<RowId, boolean>`) and behaviors.
  - Done (parity-gated): expanded state transitions and row model outputs under `paginateExpandedRows` true/false.
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/expanding.json`
    - Parity gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_expanding_parity.rs`.
- [x] HTP-expand-020 Align option gates and hooks:
  - `enableExpanding`, `manualExpanding`, `onExpandedChange`, `getExpandedRowModel` override.
  - Parity-gated: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/expanding.json` +
    `ecosystem/fret-ui-headless/tests/tanstack_v8_expanding_parity.rs`.
  - Evidence: `ecosystem/fret-ui-headless/src/table/options.rs` (`enable_expanding`)
  - Evidence: `ecosystem/fret-ui-headless/src/table/row_model.rs` (`expanded_row_model`, `reset_expanded`)
  - Evidence: `tools/tanstack-table-fixtures/extract-fixtures.mts` (fixture-only markers)
- [x] HTP-expand-030 Align `autoResetExpanded` / `autoResetAll` behaviors.
  - Parity-gated (state transition outcomes): `grouping_autoreset_expanded_*` snapshots in
    `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/grouping.json`,
    `ecosystem/fret-ui-headless/tests/tanstack_v8_grouping_parity.rs`.
  - Evidence: `ecosystem/fret-ui-headless/src/table/options.rs` (`auto_reset_all`, `auto_reset_expanded`)
  - Evidence: `ecosystem/fret-ui-headless/src/table/row_model.rs` (`Table::should_auto_reset_expanded`)
- [x] HTP-expand-040 Align row capability hooks:
  - `getRowCanExpand`, `getIsRowExpanded` default behavior + overrides.
  - Parity-gated: `expanding_hook_*` snapshots in
    `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/expanding.json`,
    `ecosystem/fret-ui-headless/tests/tanstack_v8_expanding_parity.rs`.
  - Evidence: `ecosystem/fret-ui-headless/src/table/row_model.rs` (`row_can_expand_for_row`, `row_is_expanded_for_row`)
  - Evidence: `ecosystem/fret-ui-headless/src/table/row_expanding.rs` (`expand_row_model`)
- [x] HTP-page-010 Align pagination option gates and hooks:
  - `manualPagination`, `pageCount`, `rowCount`, `onPaginationChange`, `getPaginationRowModel` override.
  - Parity-gated: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/pagination.json` +
    `ecosystem/fret-ui-headless/tests/tanstack_v8_pagination_parity.rs`.
  - Evidence: `ecosystem/fret-ui-headless/src/table/options.rs` (`page_count`, `row_count`)
  - Evidence: `ecosystem/fret-ui-headless/src/table/row_model.rs` (`pagination_updater_set_page_index`, `pagination_updater_set_page_size`, `page_count`)
  - Evidence: `tools/tanstack-table-fixtures/extract-fixtures.mts` (fixture-only markers: `__onPaginationChange`, `__getPaginationRowModel`)
- [x] HTP-page-020 Align `autoResetPageIndex` / `autoResetAll` behaviors.
  - Parity-gated (state transition outcomes): `grouping_autoreset_page_index_*` snapshots in
    `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/grouping.json`,
    `ecosystem/fret-ui-headless/tests/tanstack_v8_grouping_parity.rs`.
  - Evidence: `ecosystem/fret-ui-headless/src/table/row_model.rs` (`Table::should_auto_reset_page_index`, `Table::reset_page_index`)
  - Evidence: `tools/tanstack-table-fixtures/extract-fixtures.mts` (`mkActionsAutoReset` + snapshots)
- [x] HTP-page-030 Align `paginateExpandedRows` interactions with expansion and page bounds.
  - Parity-gated: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/expanding.json` +
    `ecosystem/fret-ui-headless/tests/tanstack_v8_expanding_parity.rs`.
  - Covered: TanStack-specific `flatRows` duplication when `paginateExpandedRows=false`
    (`expanding_paginate_expanded_rows_false_expands_within_page` snapshot).
  - Covered: `pageCount`/`rowCount`/page bounds derived from `getPrePaginationRowModel()` under
    `paginateExpandedRows` true/false (fixture asserts `page_count`, `row_count`, `can_next_page`,
    `page_options`).
- [ ] HTP-page-040 Expand auto-reset trigger coverage and queue coalescing semantics.
  - Target: ensure we are not weaker than TanStack for `autoReset*` under broader triggers:
    - columnFilters/globalFilter/pagination-affecting option changes,
    - data updates (row identity-preserving and identity-changing),
    - and coalescing behavior when multiple resets are queued in one logical tick.
  - Evidence target: extend `auto_reset.json` or add a new fixture + parity gate.
- [x] HTP-sel-010 Align selection state shape and semantics (including sub-row selection defaults).
  - Done (parity-gated): `getSelectedRowModel` / `getFilteredSelectedRowModel` / `getGroupedSelectedRowModel` equivalents,
    plus basic toggle behaviors for flat rows (including `enableMultiRowSelection=false` clearing semantics).
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/selection.json`
    - Parity gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_selection_parity.rs`
  - Done (parity-gated): nested sub-row selection defaults + `row.getIsSomeSelected()` /
    `row.getIsAllSubRowsSelected()`-equivalent semantics.
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/selection_tree.json`
    - Parity gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_selection_tree_parity.rs`
    - Covered: `enableSubRowSelection=false` prevents child selection propagation.
  - Covered: per-row function options (`enableRowSelection` / `enableSubRowSelection` / `enableMultiRowSelection` as functions).
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/selection.json`
      (snapshots: `selection_enable_row_selection_fn_odd_ids_*`)
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/selection_tree.json`
      (snapshots: `selection_tree_enable_*`)
- [x] HTP-sel-020 Align option gates and hooks:
  - `enableRowSelection`, `enableMultiRowSelection`, `enableSubRowSelection`, `enableGroupingRowSelection`,
  - `onRowSelectionChange`.
  - Done (partial): table-level boolean gates (`enableRowSelection`, `enableMultiRowSelection`, `enableSubRowSelection`) are parsed from TanStack options and applied by state transitions.
    - Evidence: `ecosystem/fret-ui-headless/src/table/tanstack_options.rs`
    - Evidence: `ecosystem/fret-ui-headless/src/table/row_selection.rs`
  - Done (parity-gated): controlled-hook noop semantics (`onRowSelectionChange` ignores updater).
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/selection_tree.json` (snapshot: `selection_tree_action_toggle_on_row_selection_change_noop_ignores`)
    - Parity gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_selection_tree_parity.rs`
  - Done (parity-gated): per-row function options (`enableRowSelection` / `enableSubRowSelection` / `enableMultiRowSelection` as functions).
    - Evidence: `ecosystem/fret-ui-headless/src/table/row_model.rs` (`TableBuilder::enable_*_row_selection_by`)
    - Evidence: `ecosystem/fret-ui-headless/src/table/row_selection.rs` (TanStack `mutateRowIsSelected`-aligned)
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/selection.json`
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/selection_tree.json`
    - Parity gates:
      - `ecosystem/fret-ui-headless/tests/tanstack_v8_selection_parity.rs`
      - `ecosystem/fret-ui-headless/tests/tanstack_v8_selection_tree_parity.rs`
- [x] HTP-colpin-010 Align column pinning option gates and hooks:
  - `enablePinning`, `enableColumnPinning`, `onColumnPinningChange`.
  - Parity-gated (option gates + state transition outcomes): `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/column_pinning.json` +
    `ecosystem/fret-ui-headless/tests/tanstack_v8_column_pinning_parity.rs`.
  - Covered: `column.getPinnedIndex()`-equivalent (`column_pinning.pinned_index` in fixture snapshots).
  - Covered: group column pinning pins its leaf columns (and group `getPinnedIndex()` returns `-1` because leaf ids are stored in state).
    - Fixture snapshot: `column_pinning_action_pin_group_pins_leaf_columns`
  - Covered: controlled-hook noop semantics (`onColumnPinningChange` ignores updater).
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/column_pinning.json`
      (snapshot: `column_pinning_action_on_column_pinning_change_noop_ignores`)
    - Evidence: `tools/tanstack-table-fixtures/extract-fixtures.mts` (`__onColumnPinningChange`)
    - Parity gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_column_pinning_parity.rs`
  - Hardened gate coverage: column pinning fixtures now also assert derived row models, selection/expanding flags,
    and column sizing totals + start/after offsets under pinning state.
    - Evidence: `ecosystem/fret-ui-headless/tests/tanstack_v8_column_pinning_parity.rs`
  - Hardened gate coverage: column pinning fixtures now also assert per-row pinned cell splits
    (`getLeftVisibleCells`/`getCenterVisibleCells`/`getRightVisibleCells`).
    - Evidence: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/column_pinning.json` (`expect.cells`)
  - Update: visibility × pinning semantics are now fixture-gated:
    - Leaf-column splits (`getLeft/Center/RightLeafColumns`) ignore visibility, matching TanStack.
    - Visible cell splits (`getLeft/Center/RightVisibleCells`) filter hidden pinned columns.
    - Fixture snapshots:
      - `column_pinning_pinned_hidden_leaf_still_in_leaf_splits_but_not_visible_cells`
      - `column_pinning_pinned_hidden_leaf_in_group_pinning_keeps_leaf_splits_but_filters_visible_cells`
- [x] HTP-colpin-020 Align `resetColumnPinning(defaultState?)` semantics.
  - Parity-gated:
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/column_pinning.json`
      (snapshots: `column_pinning_action_reset_column_pinning_restores_initial`,
      `column_pinning_action_reset_column_pinning_default_true_clears`)
    - Parity gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_column_pinning_parity.rs`
  - Evidence: `ecosystem/fret-ui-headless/src/table/row_model.rs` (`Table::reset_column_pinning`)
- [x] HTP-colpin-030 Expose TanStack-like leaf-column split helpers (`getLeft/Center/RightLeafColumns`).
  - Done: `Table::{pinned_leaf_columns,left_leaf_columns,center_leaf_columns,right_leaf_columns}`.
    - Evidence: `ecosystem/fret-ui-headless/src/table/row_model.rs`
  - Gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_column_pinning_parity.rs`
    (asserts left/center/right leaf splits against upstream snapshots).
- [x] HTP-colvis-010 Align column visibility option gates and hooks:
  - `enableHiding`, `onColumnVisibilityChange`.
  - Parity-gated (state transition outcomes + derived visible leaf order): `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/visibility_ordering.json` +
    `ecosystem/fret-ui-headless/tests/tanstack_v8_visibility_ordering_parity.rs`.
  - Covered: `getAllFlatColumns/getVisibleFlatColumns` inventory surface (including group column visibility semantics).
    - Evidence: `ecosystem/fret-ui-headless/src/table/row_model.rs` (`Table::{all_flat_columns,visible_flat_columns,is_column_visible}`).
    - Gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_visibility_ordering_parity.rs` (`core_model.flat_columns`).
  - Covered: controlled-hook noop semantics (`onColumnVisibilityChange` ignores updater).
    - Fixture snapshots: `visord_toggle_column_a_off_on_column_visibility_change_noop_ignores`
    - Evidence: `tools/tanstack-table-fixtures/extract-fixtures.mts` (`__onColumnVisibilityChange`)
  - Covered: pinning 脳 visibility 脳 ordering 脳 sizing interactions (derived pinned leaf sets + `column_start`/`column_after`).
    - Fixture snapshots: `visord_pinning_left_a_right_c_*`
- [x] HTP-colord-010 Align column ordering hook:
  - `onColumnOrderChange` (state transition outcomes + derived leaf order).
  - Parity-gated: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/visibility_ordering.json` +
    `ecosystem/fret-ui-headless/tests/tanstack_v8_visibility_ordering_parity.rs`.
  - Remaining: `groupedColumnMode` interactions (covered by future `grouping` gates).
  - Covered: controlled-hook noop semantics (`onColumnOrderChange` ignores updater).
    - Fixture snapshots: `visord_set_column_order_on_column_order_change_noop_ignores`
    - Evidence: `tools/tanstack-table-fixtures/extract-fixtures.mts` (`__onColumnOrderChange`)
- [x] HTP-corecol-010 Align core column inventory helpers:
  - `getAllColumns/getAllFlatColumns/getAllLeafColumns/getColumn/getVisibleFlatColumns`.
  - Evidence: `ecosystem/fret-ui-headless/src/table/row_model.rs`
    (`Table::{column_tree,column_tree_snapshot,column_any,column_node_snapshot,all_flat_columns,visible_flat_columns,ordered_columns}`).
  - Gates:
    - `ecosystem/fret-ui-headless/tests/tanstack_v8_headers_inventory_deep_parity.rs`
    - `ecosystem/fret-ui-headless/tests/tanstack_v8_visibility_ordering_parity.rs`

### UI integration notes (workstream hygiene)

- [x] HTP-ui-rowpin-010 Wire `TableState.row_pinning` into `table_virtualized` (flat rows + grouped path).
  - Done (initial integration): when row pinning is active, `ecosystem/fret-ui-kit` now computes
     top/center/bottom display rows via the headless engine鈥檚 pinned row helpers so 鈥渒eepPinnedRows鈥?     and pagination interactions match the engine contract.
     - Evidence: `ecosystem/fret-ui-kit/src/declarative/table.rs`
   - Done (grouped rows path, initial): when grouping is active, the grouped display cache now
     surfaces pinned rows outside pagination by reordering the flattened visible list into
     `top + center(page slice) + bottom`.
     - Evidence: `ecosystem/fret-ui-kit/src/declarative/table.rs`
- [x] HTP-ui-rowpin-020 Decide grouped-mode pinning policy and align remaining semantics.
  - Done: grouped-mode default policy is now `PromotePinnedRows` (TanStack-typical),
    so pinned rows are promoted into top/bottom display bands and removed from the paged center rows.
    - Evidence: `ecosystem/fret-ui-kit/src/declarative/table.rs`
      (`grouped_row_pinning_policy_promote_pinned_rows_matches_legacy_behavior`)
  - Done: a hierarchy-preserving alternative is retained as an explicit opt-in (`PreserveHierarchy`) for
    callers that want pinned leaf rows to remain inside grouped subtrees (no promotion).
    - Evidence: `ecosystem/fret-ui-kit/src/declarative/table.rs`
      (`grouped_row_pinning_policy_preserve_hierarchy_keeps_page_rows_center_unchanged`)
  - Done: policy-level regression gates added for grouped display composition.
    - Evidence: `ecosystem/fret-ui-kit/src/declarative/table.rs`
      (`grouped_row_pinning_policy_preserve_hierarchy_keeps_page_rows_center_unchanged`)
- [x] HTP-ui-colpin-010 Wire `TableState.column_pinning` into `table_virtualized` (headers + body).
  - Done (retained path parity): `table_virtualized_retained_v0` now computes visible ordered columns,
    splits them by `column_pinning` into `left/center/right`, and renders header/body with the same split contract.
  - Done (shared offset path): header + body center groups now share the same `scroll_x` handle while left/right stay fixed,
    preventing the header/body drift that can show up as misaligned columns in UI gallery.
    - Evidence: `ecosystem/fret-ui-kit/src/declarative/table.rs` (`table_virtualized_retained_v0`)
  - Done (UI parity gate): retained path now has a dedicated regression test covering pin/unpin + resize + center-overflow alignment.
    - Evidence: `ecosystem/fret-ui-kit/src/declarative/table.rs` (`table_virtualized_retained_colpin_alignment_gate_across_pin_resize_and_overflow`)
  - Update (measured rows): leaf rows now force `width: Fill` so measured/variable-height rows cannot shrink their width and drift vs headers.
    - Regression: `ecosystem/fret-ui-kit/src/declarative/table.rs`
      (`table_virtualized_retained_colpin_alignment_gate_measured_rows_do_not_shrink_width`)
- [x] HTP-ui-table-010 Add a UI-level alignment regression gate for the UI gallery table demo.
  - Done: `fretboard diag` scripts assert header/body column alignment via semantics bounds checks.
    - Retained table torture: `tools/diag-scripts/ui-gallery-table-retained-sort-select-scroll.json`
    - Data table torture: `tools/diag-scripts/ui-gallery-data-table-retained-sort-select-scroll.json`

---

## M6.5 鈥?Faceting parity

- [x] HTP-face-010 Gate faceting surfaces (`ColumnFaceting` / `GlobalFaceting`).
  - Parity-gated:
    - Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/faceting.json`
    - Gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_faceting_parity.rs`
  - Notes:
    - `GlobalFaceting` unique/minmax are empty/null with TanStack鈥檚 built-in helpers because
      `__global__` is not a real column (upstream warns; fixture captures this).

---

## M7 鈥?Engine memoization parity + perf gates

- [~] HTP-memo-010 Introduce dependency-driven memoization for derived models (TanStack-style).
  - Done (first building block, unit-gated): a TanStack-aligned dependency snapshot + memo cache for
    鈥渇iltered + sorted root row order鈥?
    - Evidence: `ecosystem/fret-ui-headless/src/table/tanstack_memo.rs`
    - Tests: `ecosystem/fret-ui-headless/src/table/tanstack_memo.rs` (`sorted_flat_row_order_cache_*`)
  - Remaining: lift this pattern across the full derived row model pipeline (core/filtered/sorted/expanded/paginated),
    plus a stable external cache surface for rebuild-each-frame callers.
  - Planned sub-milestones (to keep the work executable and gateable):
    - [x] HTP-memo-011 Cache filtered root row order (dependency-driven; stable external cache).
      - Evidence: `ecosystem/fret-ui-headless/src/table/tanstack_memo.rs`
        (`TanStackSortedFlatRowOrderCache.filtered_memo` + `filtered_recompute_count`)
    - [x] HTP-memo-012 Cache sorted root row order (harden deps + invalidation across sorting/columns/filtering).
      - Evidence: `ecosystem/fret-ui-headless/src/table/tanstack_memo.rs`
        (`TanStackSortedFlatRowOrderCache.sorted_memo` + `flat_row_order_signature`)
      - Gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_memo_rebuild_each_frame_gate.rs`
    - [x] HTP-memo-013 Cache expanded + paginated row order (including `paginateExpandedRows=false` semantics).
      - Evidence: `ecosystem/fret-ui-headless/src/table/tanstack_memo.rs`
        (`TanStackUngroupedRowModelOrderCache`, `TanStackRowModelOrderSnapshot`)
      - Evidence: `ecosystem/fret-ui-headless/src/table/row_model.rs`
        (`Table::tanstack_ungrouped_row_model_order_with_cache`)
      - Gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_memo_rebuild_each_frame_expanded_paginated_gate.rs`
    - [x] HTP-memo-014 Add broader guardrail gates for rebuild-each-frame callers (recompute-count expectations).
      - Gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_memo_rebuild_each_frame_guardrail_gate.rs`
- [x] HTP-memo-020 Provide an integration pattern for 鈥渞ebuild each frame鈥?while retaining memo cache.
  - Done: `Table::tanstack_sorted_flat_row_order_with_cache(items_revision, cache)` integrates a persistent
    memo cache with ephemeral `Table` rebuilds.
    - Evidence: `ecosystem/fret-ui-headless/src/table/row_model.rs` (`Table::tanstack_sorted_flat_row_order_with_cache`)
    - Gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_memo_rebuild_each_frame_gate.rs`
- [x] HTP-perf-010 Add a minimal perf regression gate for large datasets (engine-only).
  - Gate: `ecosystem/fret-ui-headless/tests/tanstack_v8_perf_large_dataset_gate.rs`

---

## M8 鈥?Parity harness (fixtures)

- [x] HTP-fixt-010 Add a Node-based fixture generator that runs upstream `table-core` and emits JSON.
  - Input: deterministic datasets + deterministic option/state transitions.
  - Output: fixtures committed under `ecosystem/fret-ui-headless/tests/fixtures/` (or equivalent).
- [x] HTP-fixt-020 Add Rust tests that load fixtures and assert parity on:
  - core row model output,
  - filtered/sorted/grouped/expanded/paginated models,
  - selection/pinning interactions.

### Fixture coverage matrix (keep this in sync)

Each row is a 鈥減arity gate鈥? a committed TanStack fixture + at least one Rust test that asserts the
fixture outcomes.

| Fixture (JSON) | `case_id` | Upstream feature(s) covered | Rust parity gate | Status |
| --- | --- | --- | --- | --- |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/demo_process.json` | `demo_process` | `ColumnFiltering`, `RowSorting`, `RowPagination` (basic), option/state transition scaffolding | `ecosystem/fret-ui-headless/tests/tanstack_v8_parity.rs` | Partial |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/pagination.json` | `pagination` | `RowPagination` (option gates + pageCount/rowCount + controlled hook semantics) | `ecosystem/fret-ui-headless/tests/tanstack_v8_pagination_parity.rs` | Partial |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/sort_undefined.json` | `sort_undefined` | `RowSorting` (`sortUndefined`: `first/last/-1/1`) | `ecosystem/fret-ui-headless/tests/tanstack_v8_sort_undefined_parity.rs` | Partial |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/column_sizing.json` | `column_sizing` | `ColumnSizing` (totals, start/after offsets, clamp, resize lifecycle + `columnSizingInfo`, RTL delta sign flip) | `ecosystem/fret-ui-headless/tests/tanstack_v8_column_sizing_parity.rs` | Partial |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/column_resizing_group_headers.json` | `column_resizing_group_headers` | `ColumnSizing` (group header resize fan-out + group entry in `columnSizingStart`) | `ecosystem/fret-ui-headless/tests/tanstack_v8_column_resizing_group_headers_parity.rs` | Partial |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/sorting_fns.json` | `sorting_fns` | `RowSorting` (sortingFn resolution: `auto` + built-ins + registry/custom) | `ecosystem/fret-ui-headless/tests/tanstack_v8_sorting_fns_parity.rs` | Partial |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/sorting_manual.json` | `sorting_manual` | `RowSorting` (`manualSorting` + `getSortedRowModel` override semantics) | `ecosystem/fret-ui-headless/tests/tanstack_v8_sorting_manual_parity.rs` | Partial |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/filtering_fns.json` | `filtering_fns` | `ColumnFiltering` / `GlobalFiltering` (`filterFns`, `resolveFilterValue`, `autoRemove`, gates) | `ecosystem/fret-ui-headless/tests/tanstack_v8_filtering_fns_parity.rs` | Partial |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/filtering_meta.json` | `filtering_meta` | `ColumnFiltering` (`row.columnFiltersMeta` / custom `addMeta` callback semantics) | `ecosystem/fret-ui-headless/tests/tanstack_v8_filtering_meta_parity.rs` | Partial |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/headers_cells.json` | `headers_cells` | `core/*` (header groups + cell ids, including pinning split families) | `ecosystem/fret-ui-headless/tests/tanstack_v8_headers_cells_parity.rs` | Partial |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/headers_inventory_deep.json` | `headers_inventory_deep` | `core/*` (deep header nesting + flat/leaf/footer inventories + hide-branch edge cases) | `ecosystem/fret-ui-headless/tests/tanstack_v8_headers_inventory_deep_parity.rs` | Partial |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/pinning.json` | `pinning` | `RowPinning` (`keepPinnedRows` vs sorting/pagination/filtering) | `ecosystem/fret-ui-headless/tests/tanstack_v8_pinning_parity.rs` | Partial |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/pinning_tree.json` | `pinning_tree` | `RowPinning` (includeLeaf/includeParent + expanded gating) | `ecosystem/fret-ui-headless/tests/tanstack_v8_pinning_tree_parity.rs` | Partial |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/column_pinning.json` | `column_pinning` | `ColumnPinning` (option gates + `pin()` transitions) | `ecosystem/fret-ui-headless/tests/tanstack_v8_column_pinning_parity.rs` | Partial |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/selection.json` | `selection` | `RowSelection` (selected models + toggle semantics for flat rows) | `ecosystem/fret-ui-headless/tests/tanstack_v8_selection_parity.rs` | Partial |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/selection_tree.json` | `selection_tree` | `RowSelection` (nested sub-row selection + `isSomeSelected`/`isAllSubRowsSelected` semantics + hook noop) | `ecosystem/fret-ui-headless/tests/tanstack_v8_selection_tree_parity.rs` | Partial |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/expanding.json` | `expanding` | `RowExpanding` (expanded row model + pagination interactions) | `ecosystem/fret-ui-headless/tests/tanstack_v8_expanding_parity.rs` | Partial |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/grouping.json` | `grouping` | `ColumnGrouping` (grouped model + flat row ordering) | `ecosystem/fret-ui-headless/tests/tanstack_v8_grouping_parity.rs` | Partial |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/grouping_sorting_precedence.json` | `grouping_sorting_precedence` | `ColumnGrouping` + `RowSorting` (grouped multi-sort precedence + aggregation-any sort keys) | `ecosystem/fret-ui-headless/tests/tanstack_v8_grouping_sorting_precedence_parity.rs` | Partial |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/grouping_row_model_pipeline.json` | `grouping_row_model_pipeline` | `ColumnGrouping` + `RowSorting` + `RowExpanding` (grouped row-model pipeline composition in `Table::row_model()`) | `ecosystem/fret-ui-headless/tests/tanstack_v8_grouping_row_model_pipeline_parity.rs` | Partial |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/grouping_cells.json` | `grouping_cells` | `ColumnGrouping` (grouped/placeholder/aggregated cell inventory flags across nested grouped rows) | `ecosystem/fret-ui-headless/tests/tanstack_v8_grouping_cells_parity.rs` | Partial |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/visibility_ordering.json` | `visibility_ordering` | `ColumnVisibility` + `ColumnOrdering` (state transitions + derived leaf column order) | `ecosystem/fret-ui-headless/tests/tanstack_v8_visibility_ordering_parity.rs` | Partial |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/faceting.json` | `faceting` | `ColumnFaceting` / `GlobalFaceting` | `ecosystem/fret-ui-headless/tests/tanstack_v8_faceting_parity.rs` | Partial |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/auto_reset.json` | `auto_reset` | auto-reset semantics (`autoResetAll`, `autoResetPageIndex`) under sorting/globalFilter changes | `ecosystem/fret-ui-headless/tests/tanstack_v8_auto_reset_parity.rs` | Partial |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/row_id_state_ops.json` | `row_id_state_ops` | Row identity/state ops (`RowSelection` + `RowExpanding` + `RowPinning` by string RowId, grouped/nested grouped ids) | `ecosystem/fret-ui-headless/tests/tanstack_v8_row_id_state_ops_parity.rs` | Partial |
| `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/row_id_lookup.json` | `row_id_lookup` | Row identity lookup (`table.getRow(id, searchAll?)`) + `rowsById` parity across core/pre-pagination/final models (custom RowId + grouped ids) | `ecosystem/fret-ui-headless/tests/tanstack_v8_row_id_lookup_parity.rs` | Partial |
