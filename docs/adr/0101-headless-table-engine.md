# ADR 0101: Headless Table Engine (TanStack-Aligned) and UI Recipes

Status: Accepted

## Context

Fret targets a “primitives + recipes” UI ecosystem similar to shadcn/Radix, but on a non-DOM
runtime (ADR 0066, ADR 0090). For tables, shadcn explicitly recommends composing the low-level
`<Table />` primitives with a headless table engine (TanStack Table v8) rather than shipping a
single monolithic “DataTable component”.

We already have:

- shadcn-aligned `Table` primitives (`ecosystem/fret-ui-shadcn/src/table.rs`),
- a first-pass `DataTable` that provides virtualization + fixed header but intentionally does not
  aim for TanStack feature parity,
- a `DataGrid` prototype to explore 2D virtualization (rows + columns).

What we are missing is a reusable, framework-native headless “table state + row model pipeline”
layer that can power:

- sorting, filtering, pagination,
- column visibility and ordering,
- row selection and “selected count” derived models,
- column sizing/resizing and pinning (future),
- consistent stable IDs and metadata needed for accessibility and virtualization (ADR 0033, ADR
  0085, ADR 0070).

We also want to avoid introducing a new top-level crate solely for table logic unless it provides
clear value. Our existing layering already positions `ecosystem/fret-ui-kit` as the natural home
for deterministic headless logic (ADR 0090).

Reference implementation and terminology:

- TanStack Table v8 (pinned locally under `repo-ref/table`).

## Decision

### 1) Put the headless table engine in `ecosystem/fret-ui-kit` (feature-gated)

We will implement a TanStack-aligned headless table engine under:

- `ecosystem/fret-ui-kit/src/headless/table/` (module name: `headless::table`)
- Cargo feature: `table` (default-off unless/until it becomes widely used)

The headless engine:

- MUST be deterministic and unit-testable without `fret-ui` or any renderer backend.
- MUST NOT depend on `ecosystem/fret-ui-shadcn` (no circular layering).
- SHOULD avoid `fret-ui` dependencies in the headless module; UI integration happens in recipes.

Rationale:

- Keeps `crates/fret-ui` mechanism-only (ADR 0066).
- Keeps policy and state machines in `fret-ui-kit` (ADR 0090).
- Avoids a new workspace crate while still providing a stable “headless” surface.

### 2) Expose a TanStack-inspired contract (not a 1:1 port)

We will adopt TanStack’s core concepts and vocabulary where they improve ergonomics and migration:

- columns (`ColumnDef`), header groups, rows, cells,
- a table instance holding `state` + derived row models,
- a “row model pipeline” (core -> filtered -> sorted -> paginated, with optional steps).

We will NOT try to replicate React hook APIs. Instead, we expose explicit Rust types and methods.

### 3) Stable row identity is a first-class requirement

The engine MUST provide stable row identity needed for:

- caching keyed UI subtrees (ADR 0039),
- row selection state that survives sorting/filtering,
- accessibility collection semantics under virtualization (ADR 0085).

Contract:

- `ColumnId`: stable string identifier (recommended: `Arc<str>`).
- `RowKey`: stable numeric identity chosen by the caller (`u64` newtype).
- `CellId`: derived from `(row_key, column_id)`; stable across row-model transforms.

The engine will not assume that row indices are stable.

Rationale (performance and future-proofing):

- Using `RowKey(u64)` makes row identity usable in hot paths (selection, row maps, virtualization
  keys, derived-model tie-breakers) without heap allocation and with low hash/compare cost.
- Callers that already have a stable backend primary key can use it directly.
- Callers that only have a string identifier can map it to a `RowKey` (e.g. a stable numeric id in
  the data model, or a persisted mapping), but the headless core does not force string allocation.

We may still carry optional human-readable row labels/paths for debugging and diagnostics, but they
must not be required for correctness and must not be used as the primary hot-path key.

### 4) Table state is explicitly owned by the caller

Like TanStack, the engine will support both controlled and uncontrolled patterns, but the default
should be explicit-state-in, derived-model-out:

- `TableState` includes sorting, filters, pagination, visibility, selection, and (future) sizing.
- Derived models are computed from `(columns, data, state, revisions)` with predictable invalidation.

This aligns with Fret’s general “app-owned models” and revision-driven invalidation patterns (ADR
0031, ADR 0051).

### 5) UI recipes live in `fret-ui-shadcn` behind features

We will provide convenience recipes in `ecosystem/fret-ui-shadcn`, but keep them clearly scoped as
“guides / reusable examples”:

- Feature: `tanstack_table` (name not final)
- Modules (indicative):
  - `data_table_tanstack` (renders with `Table` primitives + pagination/sorting/filters widgets)
  - `data_grid_tanstack` (renders 2D virtualization when column sizing/pinning exist)

The recipes:

- MAY depend on `fret-ui` primitives (VirtualList, Scrollbar, WheelRegion).
- SHOULD emit accessibility semantics using the runtime semantics tree (ADR 0033).
- SHOULD remain optional so apps can build their own table variations without committing to a
  single framework-provided “DataTable component”.

### 6) Virtualization and rendering are integration concerns, not headless concerns

The headless engine produces:

- ordered row keys for the active model,
- per-row visible cells / per-column metadata,
- column sizing results (future).

Virtualization is applied by the UI layer:

- `DataTable`: vertical virtualization via `VirtualList`.
- `DataGrid`: 2D virtualization via `(VirtualList rows) + (VirtualList columns)` or an eventual
  dedicated 2D virtualization primitive.

UI recipes MUST ensure the virtualization viewport is bounded (e.g. `Fill` size + `flex: 1` and
`overflow: clip/scroll` on the container); otherwise virtualization may degenerate into “render all
rows”, causing severe frame-time spikes (ADR 0070, ADR 0088).

## Design Outline (Non-Normative)

This section describes the intended shape; exact names may change during implementation.

### Core types

- `ColumnDef<TData>`:
  - `id: ColumnId`
  - `header: HeaderDef` (label or custom key)
  - `accessor: AccessorFn<TData>` (value extraction for sort/filter)
  - optional feature hooks: `sorting`, `filtering`, `visibility`, `sizing`
- `TableState`:
  - `sorting: Vec<SortSpec>`
  - `column_filters: Vec<FilterSpec>`
  - `column_visibility: HashMap<ColumnId, bool>`
  - `pagination: PaginationState`
  - `row_selection: HashSet<RowKey>` (future: bitset when `RowKey` is dense)
  - future: `column_order`, `column_sizing`, `column_pinning`
- `Table<TData>` / `TableInstance<TData>`:
  - accepts `data: Arc<[TData]>` (or a data provider + revision)
  - exposes `get_row_model()` and per-feature helpers

### Row model pipeline (initial)

We standardize the initial pipeline order:

1. **Core**: materialize rows with stable row keys and base cells.
2. **Filtering**: apply column filters and/or global filter (optional).
3. **Sorting**: stable sort (tie-breaker by row key) using column sort functions.
4. **Pagination**: apply page index/page size.

Later steps (deferred): grouping, expanded rows, column pinning, column ordering/sizing.

### Memoization model (TanStack-aligned)

To match TanStack Table's performance characteristics, derived models SHOULD be computed through a
dependency-driven memo layer (similar to TanStack's `memo(getDeps, fn)`):

- Each derived model (`core`, `sorted`, `paginated`, `selected`, etc.) has explicit dependencies
  (data revision + relevant slice of `TableState` + column defs revision).
- The engine recomputes only the models whose dependency tuple changed.

This avoids accidental O(n) work per-frame and makes “large table” performance predictable.

## Accessibility Requirements

Accessibility is expressed via Fret’s semantics tree (ADR 0033) and virtualization semantics (ADR
0085). The headless layer must therefore supply:

- stable row and column IDs,
- row/column indices *in the active model* (even if virtualized),
- selection state and sort state as derived metadata.

UI recipes must map these to appropriate roles/fields (exact mapping depends on the current A11y
backend; see ADR 0033 and platform conformance work).

## Alternatives Considered

1) **New workspace crate `ecosystem/fret-table`**
   - Pros: very clean dependency boundary; could be reused without `fret-ui-kit`.
   - Cons: extra crate and versioning surface; more workspace churn.
   - Decision: rejected for now; revisit if `fret-ui-kit` becomes too broad.

2) **Implement in `crates/fret-ui`**
   - Pros: “one place” and easy access to runtime primitives.
   - Cons: violates mechanism-only boundary (ADR 0066) and would pressure the runtime contract.
   - Decision: rejected.

3) **Keep table logic inside each component recipe**
   - Pros: fastest short-term.
   - Cons: drift across components, poor testability, hard to stabilize.
   - Decision: rejected.

## Consequences

- We gain a reusable, testable headless table layer aligned with shadcn/TanStack expectations.
- `fret-ui-shadcn` can ship “guide-level” table recipes without becoming a monolithic data grid.
- We increase the scope of `fret-ui-kit`; feature-gating mitigates footprint.

## Implementation Plan (Phased)

1) Add `headless::table` module skeleton + core row model + stable IDs.
2) Implement sorting + pagination + row selection + column visibility (minimal shadcn guide set).
3) Implement filtering (column + optional global).
4) Add initial shadcn recipes behind a feature flag.
5) Add column sizing/resizing + pinning (needed for production-grade DataGrid).
