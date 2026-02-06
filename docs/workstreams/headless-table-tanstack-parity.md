Status: Active (workstream note; not a contract)

This workstream targets **full capability parity** with **TanStack Table v8 `table-core`** (engine
layer, not React integration). “Capability parity” means: for every upstream feature and public API
surface that exists in `table-core`, Fret’s headless engine can express it without losing power.

- Upstream reference (local checkout): `repo-ref/table/packages/table-core`
- Fret implementation: `ecosystem/fret-ui-headless/src/table/`
- Primary consumer (UI integration): `ecosystem/fret-ui-shadcn/src/data_table.rs`
- Related ADR: `docs/adr/0101-headless-table-engine.md`

---

## Version Stamp

Parity fixtures are generated from a local `repo-ref/table` checkout. The committed baseline fixture
captures the exact upstream version and commit:

- Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/demo_process.json`
- Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/sort_undefined.json`
- Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/sorting_fns.json`
- Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/filtering_fns.json`
- Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/auto_reset.json`
- Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/column_sizing.json`
- Fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/headers_cells.json`
- Upstream: `@tanstack/table-core@8.21.3` (repo-ref commit `e172109fca4cc403a07236ed8fa103450ceba5e9`)

“Parity” in this workstream means:

1. **Feature coverage parity**: every `table-core` feature is represented and behaves equivalently.
2. **Compatibility by default**: for overlapping behaviors, the default configuration matches
   upstream semantics (fixtures gate the observable outcomes).
3. **Superset allowances**: behavior may differ *only* when it unlocks extra capabilities, and
   should be opt-in and/or policy-configurable so consumers can still choose TanStack-compatible
   outcomes.
4. **State-shape parity**: state can be round-tripped to/from a TanStack-equivalent JSON shape.
5. **Performance parity (at the engine level)**: derived models are computed via dependency-driven
   memoization (TanStack `memo(getDeps, fn)`-style), so large tables remain predictable.

Non-goals:

- Rendering, virtualization, or interaction policy (those belong to UI recipes, e.g. `DataTable`).
- HTML semantics or DOM-only concerns. We match `table-core` outcomes, not browser layout.

---

## Guiding Principles

1. **Upstream is the spec**
   - When docs and code disagree, treat `table-core` code as the source of truth.
2. **Parity by observable outcomes**
   - Parity is gated by fixtures/tests that compare derived models and state transitions.
3. **Separate “engine parity” from “Rust ergonomics”**
   - Provide a TanStack-compatible “surface” (state + behaviors), and optionally a Rust-native
     facade. Do not compromise parity to simplify the public API.
4. **Stable IDs first**
   - IDs (row/column/header) must be stable and deterministic; everything else depends on this.

---

## Capability Inventory (selected; expand over time)

This is the “not weaker than TanStack” checklist. Each item should map to a concrete Rust surface
and be covered by parity fixtures when behavior overlap exists.

### Instance-level surface (Table/Row/Column/Header/Cell)

Upstream source of truth:

- Table: `repo-ref/table/packages/table-core/src/core/table.ts` (`CoreInstance`)
- Row: `repo-ref/table/packages/table-core/src/core/row.ts` (`CoreRow`)
- Column: `repo-ref/table/packages/table-core/src/core/column.ts` (`CoreColumn`)
- Header/Cell: `repo-ref/table/packages/table-core/src/core/headers.ts` and `core/cell.ts`
- Feature methods: `repo-ref/table/packages/table-core/src/features/*.ts` (`createTable/createRow/createColumn`)

Inventory scope (initial; expand as we hit consumers):

- Table:
  - Column trees: `getAllColumns/getAllFlatColumns/getAllLeafColumns/getColumn`.
  - Row models: `getCoreRowModel/getRowModel` + intermediate row model helpers (pre-pagination, etc).
  - Identity/lookup: `getRow(rowId, searchAll?)` and `rowsById`-equivalent map.
  - Header groups: `getHeaderGroups` + pin-family variants.
- Row:
  - Identity: `id` (string), parent/leaf relations, `getLeafRows/getParentRows`.
  - Cells: `getAllCells/getVisibleCells` + pinned splits.
  - Behaviors: selection/expanding/pinning helpers + “getXHandler” style updaters.
- Column:
  - Visibility/ordering/pinning/sizing/grouping/sorting/filtering helper methods + handler updaters.
- Header/Cell:
  - Stable IDs and the minimal accessors required by `DataTable` UI recipes (`getSize/getStart` for headers,
    `getValue/renderValue` for cells, etc).

Initial mapping snapshot (keep updated):

| Upstream API | Fret surface (today) | Status | Notes |
| --- | --- | --- | --- |
| `table.getRow(id, searchAll?)` | `Table::row_by_id(&str, search_all)` | Partial | Leaf rows are addressable by `RowId`; grouped row ids + state-keyed behaviors are still tracked under `HTP-id-010`. |
| `row.id: string` | `Row::id: RowId` (`Arc<str>`) | Partial | Leaf rows have a string id; grouped row ids are not first-class in the main pipeline yet. |
| `RowModel.rowsById` | `RowModel::rows_by_id()` | Partial | Present for leaf rows; grouped row ids + “searchAll” coverage still needs broader gates. |
| `table.getHeaderGroups()` (+ pinned variants) | `Table::header_groups/left_header_groups/center_header_groups/right_header_groups` | Aligned (core) | Fixture-gated via `headers_cells.json`. |
| `header.getSize()` / `header.getStart()` | `Table::header_size/header_start` | Aligned (core) | Fixture-gated via column sizing/header tests. |
| `column.getSize()` / `column.getStart()` / `column.getAfter()` | `Table::column_size/column_start/column_after` | Aligned (core) | Fixture-gated via column sizing tests. |
| `column.getIsPinned()` / `column.pin()` | `Table::column_pin_position` + `Table::toggled_column_pinning` | Aligned (core) | Column pinning fixtures gate state transitions + derived splits. |
| `table.getTopRows/getCenterRows/getBottomRows` | `Table::top_row_keys/center_row_keys/bottom_row_keys` | Partial | Keys-only; group-root semantics under grouping require `RowId` integration. |

### Row identity (`RowId`) and lookup

Upstream:

- `table.getRow(rowId, searchAll?)` (pre-pagination vs current model lookup)
- `table.getCoreRowModel().rowsById` (stable lookup)
- Grouped row ids (string ids like `role:1`) are first-class rows

Fret status:

- Leaf rows now carry a stable string `RowId` alongside the existing numeric `RowKey(u64)` fast path.
- `RowModel` maintains both `rows_by_key` and `rows_by_id` for lookup, and `Table::row_by_id` mirrors
  TanStack `getRow(id, searchAll?)` shape for leaf rows.
- Remaining capability gap: grouped row ids (e.g. `role:1`) and all id-keyed feature state surfaces
  (selection/expanded/pinning maps keyed by string ids) must be promoted to `RowId` without losing
  existing `RowKey` optimizations.
  - Tracked in TODO: `HTP-id-010`.

Compatibility requirement:

- Fret must be able to address rows by a stable string id without loss of capability (pin/select/expand/lookup),
  even if a Rust-native `RowKey(u64)` is kept as an optimization.

### Row pinning over grouped rows

Upstream:

- `row.pin(position, includeLeafRows?, includeParentRows?)` works for any `Row`
- `getTopRows/getCenterRows/getBottomRows` operate on `table.getRowModel().rows`

Fret status:

- Leaf row pinning is parity-gated; grouped interactions are partially gated.
- Capability gap to close: pin/group rows as first-class and align `getCenterRows` semantics under
  grouping.

---

## Upstream Feature Map (TanStack → Fret)

The upstream `table-core` feature files live under:

- `repo-ref/table/packages/table-core/src/features/*.ts`

Initial mapping (module parity; behavior parity is tracked in TODOs):

- ColumnFiltering / GlobalFiltering → `filtering.rs`
- ColumnFaceting / GlobalFaceting → `faceting.rs`
- RowSorting → `sorting.rs`
- RowPagination → `pagination.rs`
- RowSelection → `row_selection.rs`
- RowPinning → `row_pinning.rs`
- ColumnVisibility → `column_visibility.rs`
- ColumnOrdering → `column_ordering.rs`
- ColumnPinning → `column_pinning.rs`
- ColumnSizing → `column_sizing.rs` + `column_sizing_info.rs`
- ColumnGrouping → `grouping.rs` + `grouped_sorting.rs` + `grouped_aggregation.rs`
- RowExpanding → `row_expanding.rs`

Known “core engine” gaps (as of this workstream start):

- TanStack core types (`headers`, `cells`, nested column trees / header groups) are not yet
  represented 1:1 in `fret-ui-headless`.
- Filtering supports typed filter values (via `serde_json::Value`) and parity-gated built-in
  `filterFns` + `resolveFilterValue` + `autoRemove`, but still lacks per-row filter meta, depth
  controls, and a fully typed `globalFilter` surface.
- Sorting lacks TanStack behaviors like sortingFn auto-selection, and the remaining
  `sortUndefined: false` semantics (see M3 / HTP-sort-010 + HTP-sort-040).
- “Auto-reset” behaviors are parity-gated, but we do not yet model TanStack’s instance-level
  `_queue` mechanism as a first-class runtime concern. Current parity tests simulate the
  “register first, reset later” semantics at the state-transition layer.

---

## Parity Matrix (feature-by-feature)

Status legend:

- **Partial**: module exists, but one or more upstream option/edge-case behaviors are missing.
- **Aligned**: parity-gated by fixtures (not just “looks right in a demo”).

This matrix is the living checklist. When a row becomes “Aligned”, add at least one evidence anchor
(test/fixture path) and keep it current.

| Upstream (`table-core`) | Fret module(s) | Status | Notes (non-exhaustive) |
| --- | --- | --- | --- |
| `ColumnFiltering.ts` | `filtering.rs` | Partial | Built-in filterFns + `resolveFilterValue` + `autoRemove` are parity-gated by `ecosystem/fret-ui-headless/tests/tanstack_v8_filtering_fns_parity.rs` (fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/filtering_fns.json`). Remaining: `filterFromLeafRows`, `maxLeafRowFilterDepth`, per-row filter meta, and `getCanFilter` option-gates surfaces parity. |
| `GlobalFiltering.ts` | `filtering.rs` | Partial | Global filter row-model behavior (default `globalFilterFn: 'auto'` => includesString) and `getColumnCanGlobalFilter` eligibility semantics are parity-gated by `ecosystem/fret-ui-headless/tests/tanstack_v8_filtering_fns_parity.rs` (fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/filtering_fns.json`, snapshot: `filtering_fns_global_filter_default_excludes_bool`). Remaining: fully typed `globalFilter` value surface + controlled state hook parity (`onGlobalFilterChange`). |
| `ColumnFaceting.ts` | `faceting.rs` | Aligned | Parity-gated by `ecosystem/fret-ui-headless/tests/tanstack_v8_faceting_parity.rs` (fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/faceting.json`). |
| `GlobalFaceting.ts` | `faceting.rs` | Aligned | Parity-gated by `ecosystem/fret-ui-headless/tests/tanstack_v8_faceting_parity.rs` (fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/faceting.json`). Note: built-in helpers produce empty/null global unique/minmax since `__global__` is not a real column (fixture captures upstream warning). |
| `RowSorting.ts` | `sorting.rs` | Partial | sortingFn resolution (`auto` + built-ins + named registry) is parity-gated by `ecosystem/fret-ui-headless/tests/tanstack_v8_sorting_fns_parity.rs` (fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/sorting_fns.json`). Remaining: `getAutoSortDir` first-toggle inference; `invertSorting`, `sortDescFirst`, multi-sort gates/transitions, and `sortUndefined` semantics are parity-gated by `ecosystem/fret-ui-headless/tests/tanstack_v8_parity.rs` + `ecosystem/fret-ui-headless/tests/tanstack_v8_sort_undefined_parity.rs` (fixtures under `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/`). |
| `RowPagination.ts` | `pagination.rs` | Partial | Pagination option gates + `pageCount`/`rowCount` + controlled-state hook parity is gated by `ecosystem/fret-ui-headless/tests/tanstack_v8_pagination_parity.rs` (fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/pagination.json`). `autoResetPageIndex` / `autoResetAll` semantics are parity-gated by `ecosystem/fret-ui-headless/tests/tanstack_v8_auto_reset_parity.rs` (fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/auto_reset.json`). Remaining: expand the auto-reset harness to cover more triggers (e.g. columnFilters, data updates) and queue coalescing behavior. |
| `RowSelection.ts` | `row_selection.rs` | Partial | Parity-gated selected row models + toggle semantics for flat rows: `ecosystem/fret-ui-headless/tests/tanstack_v8_selection_parity.rs` (fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/selection.json`). Nested sub-row selection + `isSomeSelected`/`isAllSubRowsSelected` semantics are gated by `ecosystem/fret-ui-headless/tests/tanstack_v8_selection_tree_parity.rs` (fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/selection_tree.json`). Remaining: per-row option functions parity. |
| `RowExpanding.ts` | `row_expanding.rs` | Partial | Parity-gated expanded row model + pagination interactions (including `paginateExpandedRows` true/false, page bounds, and TanStack `flatRows` duplication): `ecosystem/fret-ui-headless/tests/tanstack_v8_expanding_parity.rs` (fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/expanding.json`). Remaining: deeper per-row option function parity + controlled hook coverage expansion. |
| `ColumnGrouping.ts` | `grouping.rs`, `grouped_sorting.rs`, `grouped_aggregation.rs` | Partial | groupedColumnMode ordering parity is gated by `ecosystem/fret-ui-headless/tests/tanstack_v8_headers_cells_parity.rs` (fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/headers_cells.json`, snapshots: `headers_cells_grouped_column_mode_*`). aggregationFns + renderFallbackValue parity is gated by `ecosystem/fret-ui-headless/tests/tanstack_v8_grouping_aggregation_fns_parity.rs` + `ecosystem/fret-ui-headless/tests/tanstack_v8_render_fallback_parity.rs` (fixtures: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/grouping_aggregation_fns.json`, `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/render_fallback.json`). Remaining: deeper grouped sorting parity, multi-sort precedence, placeholder/aggregated cell inventories, and grouping value surface parity for non-numeric grouping values. |
| `ColumnPinning.ts` | `column_pinning.rs` | Aligned | Option gates + hook/state transition parity is gated by `ecosystem/fret-ui-headless/tests/tanstack_v8_column_pinning_parity.rs` (fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/column_pinning.json`). Gate also asserts derived row models, selection/expanding flags, and column sizing totals + start/after offsets under pinning state. |
| `ColumnOrdering.ts` | `column_ordering.rs` | Aligned | Parity-gated by `ecosystem/fret-ui-headless/tests/tanstack_v8_visibility_ordering_parity.rs` (fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/visibility_ordering.json`). |
| `ColumnVisibility.ts` | `column_visibility.rs` | Aligned | Parity-gated by `ecosystem/fret-ui-headless/tests/tanstack_v8_visibility_ordering_parity.rs` (fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/visibility_ordering.json`). |
| `ColumnSizing.ts` | `column_sizing.rs`, `column_sizing_info.rs` | Partial | Totals, pinned start/after offsets, size clamp, LTR/RTL resize lifecycle (`columnResizeMode`: OnChange vs OnEnd + `columnSizingInfo`) are parity-gated by `ecosystem/fret-ui-headless/tests/tanstack_v8_column_sizing_parity.rs` (fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/column_sizing.json`). Group-header resize fan-out + `columnSizingStart` shape are parity-gated by `ecosystem/fret-ui-headless/tests/tanstack_v8_column_resizing_group_headers_parity.rs` (fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/column_resizing_group_headers.json`). Remaining: complete `columnSizingInfo` inventory + cross-feature interactions coverage. |
| `RowPinning.ts` | `row_pinning.rs` | Aligned | keepPinnedRows + filter/sort/pagination interaction parity is gated by `ecosystem/fret-ui-headless/tests/tanstack_v8_pinning_parity.rs` + `ecosystem/fret-ui-headless/tests/tanstack_v8_pinning_tree_parity.rs` (fixtures: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/pinning.json`, `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/pinning_tree.json`). Gate also asserts derived row models (incl. `paginateExpandedRows` flattened `expanded.rows`), selection/expanding flags, and column sizing totals + start/after offsets. |
| `core/*` (columns/headers/rows/cells) | `headers.rs` + `cells.rs` + `core_model.rs` | Partial | Core model snapshot (column tree + leaf sets + header groups + row/cell ids) is parity-gated by `ecosystem/fret-ui-headless/tests/tanstack_v8_headers_cells_parity.rs` (fixture: `ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/headers_cells.json`). Remaining: full column/header/cell inventories + deeper nesting + visibility edge cases. |

---

## Milestones

### M0 — Tracker + scope lock (this doc)

Definition of done:

- This narrative tracker exists and is referenced by a TODO tracker with executable tasks.
- Parity target is explicitly “TanStack Table v8 `table-core` engine semantics”.

Evidence:

- `docs/workstreams/headless-table-tanstack-parity.md`
- `docs/workstreams/headless-table-tanstack-parity-todo.md`

### M1 — Core type system parity (columns/headers/rows/cells)

Goal: match TanStack’s “core” constructs so feature parity has a stable foundation.

Definition of done:

- Nested column trees + header group generation exist and are deterministic.
- Core derived outputs have a stable JSON-serializable representation (for fixtures).

### M2 — State-shape parity + reset semantics

Goal: represent TanStack-equivalent state (and defaults) without losing information.

Current status:

- A round-trip parity gate exists for a growing subset of state keys (see `tanstack_v8_state_roundtrip_parity.rs`),
  but lossless “presence” semantics (omitted vs explicit defaults) are not fully modeled yet.
- A dedicated fixture (`ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/state_shapes.json`) covers
  grouping/expanded/rowPinning/globalFilter state shapes.
- Reset semantics parity gates exist for pinning (`resetRowPinning` / `resetColumnPinning`), including
  `initialState` vs `state` behavior (fixtures: `pinning.json`, `column_pinning.json`).
- A dedicated reset semantics parity gate exists for the remaining table-level reset surfaces
  (sorting/filtering/grouping/visibility/order/rowSelection): fixture `resets.json`.

Definition of done:

- State types can be round-tripped from/to a TanStack-compatible JSON schema.
- “resetX(defaultState?)” behaviors are expressible and tested (where applicable).

### M3 — Filtering + sorting depth parity

Goal: implement the “hard parts” that most commonly diverge.

Definition of done:

- Filtering supports typed filter values and TanStack behaviors:
  - per-column filterFn resolution (auto/built-in/custom),
  - `resolveFilterValue`, `autoRemove`,
  - `getColumnCanGlobalFilter` default + override hooks,
  - `maxLeafRowFilterDepth` semantics,
  - per-row filter pass/fail map and optional filter meta.
- Sorting supports TanStack behaviors:
  - sortingFn selection (`auto` and custom),
  - `sortUndefined`, `invertSorting`, `sortDescFirst`,
  - multi-sort gating + deterministic tie-breaking.

### M4 — Column sizing/resizing parity (including pinned offsets)

Definition of done:

- `columnResizeMode` OnChange vs OnEnd is equivalent.
- RTL resize direction matches TanStack semantics.
- Resizing a group header fans out to leaf columns (and `columnSizingStart` matches TanStack’s shape).
- Start/after offsets in pinned regions match (used by UI layout).

### M5 — Grouping + aggregation parity

Definition of done:

- Grouped rows and aggregations match TanStack’s grouped row model outputs.
- Grouped sorting parity (including how group rows compare and how children are ordered).

### M6 — Pinning/expanding/selection pagination interactions parity

Definition of done:

- Options like `keepPinnedRows` and `paginateExpandedRows` match TanStack.
- Selection/expansion interactions are parity-gated via fixtures.

### M7 — Engine memoization parity + perf gates

Definition of done:

- Derived models are computed using dependency-driven memoization (TanStack-style).
- Large-table regressions are guarded by targeted perf tests/bench harness (not UI-dependent).

### M8 — Parity harness (fixtures)

Definition of done:

- A repeatable process generates upstream fixtures (via `repo-ref/table`) for:
  - derived row models,
  - state transitions for each feature,
  - selected edge cases.
- Rust tests load fixtures and assert parity.

Fixture schema (v0; evolving, but kept stable for tests):

- `upstream`:
  - `package`, `version`, `commit`, `commit_short`, `source` (string stamps)
- `case_id`: string ID for the fixture data set
- `data`: array of rows (case-specific row shape)
- `snapshots[]`:
  - `id`: snapshot ID (stable string)
  - `options`: subset of TanStack table options serialized as JSON
  - `state`: subset of TanStack table state serialized as JSON (pre-action)
  - `actions?`: optional action list to derive a post-action state (state transitions)
    - `{ type: "toggleSorting", column_id, multi? }`
    - `{ type: "toggleSortingHandler", column_id, event_multi? }`
    - `{ type: "setColumnFilterValue", column_id, value }`
    - `{ type: "setGlobalFilterValue", value }`
  - `expect`: observed upstream outputs
    - `core` / `filtered` / `sorted` / `paginated` / `row_model`: row id lists (`root` + `flat`)
    - `next_state?`: expected post-action state subset (when `actions` is present)

---

## Next Actions

- Use the TODO tracker: `docs/workstreams/headless-table-tanstack-parity-todo.md`
- Start with M1/M2 (types + state), then lock parity by fixtures before large refactors.

Current focus (2026-02-06): M8 — Auto-reset fixture gate (`autoResetPageIndex`/`autoResetAll`) + start M7 memoization plan.
