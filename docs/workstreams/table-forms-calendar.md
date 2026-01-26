# Table / Forms / Calendar Roadmap (Workstream)

Status: Draft (tracking only; ADRs remain the source of truth)

This document tracks the implementation plan and progress for:

- Headless Table / DataGrid (align ADR 0101; remove long-lived feature gate if no heavy deps)
- Forms (headless form state + validation)
- Calendar / Date Picker (date math + interactions; shadcn-aligned recipes)
 - Diagnostics for “table not visible” regressions (bundle semantics + optional screenshot)

It is intentionally non-normative: do not treat this as a contract. For contracts, see ADRs in `docs/adr/`.

## Goals

- Ship a production-usable, **headless** Table core in `ecosystem/fret-ui-kit` that can back a shadcn-aligned `DataTable`.
- Keep `crates/fret-ui` mechanism-only (ADR 0066): no component policies, no table-specific UI defaults.
- Unblock editor-grade surfaces that depend on tables/forms/dates (inspector-like panels, settings UIs, list/detail pages).
- Provide a “performance ceiling” DataGrid path for spreadsheet-scale density: canvas/GPU-backed rendering with constant-ish UI node count.

## Non-Goals (v1)

- Full spreadsheet engine (formulas, merged cells, pivot tables).
- Perfect a11y parity for all grid patterns on day one (but we must not regress baseline semantics).
- Server-side data adapters and networking policy.
- Perfect “Excel-class” feature parity (merged cells, formulas, pivot tables, collaborative editing).

## Primary Contracts (Source of Truth)

- ADR 0101 (Headless Table Engine): `docs/adr/0101-headless-table-engine.md`
- Runtime contract gates: `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- Virtualization vocabulary: `docs/adr/0070-virtualization-contract.md`
- Semantics + accessibility bridge: `docs/adr/0033-semantics-tree-and-accessibility-bridge.md`
- Active descendant patterns (for composite widgets): `docs/adr/0073-active-descendant-and-composite-widget-semantics.md`

## Reference Sources (repo-ref)

We treat these as reading aids, not as implementation targets.

### Table / DataGrid

- TanStack Table: `repo-ref/tanstack-table` (record commit SHA below)
- Glide Data Grid (optional perf/UX reference): `repo-ref/glide-data-grid` (record commit SHA below)

### Forms

- React Hook Form: `repo-ref/react-hook-form` (record commit SHA below)
- Zod: `repo-ref/zod` (record commit SHA below)

### Calendar / Date Picker

- React Day Picker: `repo-ref/react-day-picker` (record commit SHA below)
- WAI-ARIA APG: `repo-ref/aria-practices` (record commit SHA below)

### Shared

- Radix UI Primitives: `repo-ref/primitives` (pinned via `tools/fetch_repo_refs.ps1`)
- Floating UI: `repo-ref/floating-ui`
- TailwindCSS vocabulary: `repo-ref/tailwindcss`
- GPUI/Zed references: `repo-ref/zed`, `repo-ref/gpui-component`

### Variable-size virtualization (reference implementations)

These are used to study **variable row heights / variable column widths** range computation,
measurement write-back, and large-scroll handling. They are not implementation targets.

- TanStack Virtual: `repo-ref/tanstack-virtual`
- React Virtualized: `repo-ref/react-virtualized` (notably `CellSizeAndPositionManager` + scaling)
- React Virtuoso: `repo-ref/react-virtuoso` (measurement + compensation; useful for “content is dynamic” failure modes)

#### Reference commit SHAs (fill after cloning)

- tanstack-table: `e172109fca4c`
- glide-data-grid: `ab7042389afd`
- react-hook-form: `195139d5a969`
- zod: `9977fb086843`
- react-day-picker: `fee7c41a14a3`
- aria-practices: `84b921a0c664`
- floating-ui: `0681dbb620ca`
- tailwindcss: `9720692edac4`
- tanstack-virtual: `5d6acc953f62`
- react-virtualized: `c737715486f7`
- react-virtuoso: `0f322855cb08`

Note: attempted to add `react-window` (VariableSizeGrid), but the clone failed due to a Windows Git
TLS handshake error. If needed, retry with an explicit proxy or alternate SSL backend.

## Current Code Surfaces (Audit)

As of the initial audit:

- Diagnostics:
  - The UI gallery exposes stable `test_id` anchors for Table/DataTable and ships smoke scripts under `tools/diag-scripts/*`.
  - If semantics + bounds look correct but pixels look blank, run with `FRET_DIAG_SCREENSHOT=1` and inspect `frame.bmp` in the dumped bundle directory.
- `ecosystem/fret-ui-shadcn`
  - `Table` primitives exist and are always available: `ecosystem/fret-ui-shadcn/src/table.rs` (shadcn taxonomy).
  - `DataTable` is headless-backed (ADR 0101) and rendered via the shared declarative table view:
    `ecosystem/fret-ui-shadcn/src/data_table.rs` -> `ecosystem/fret-ui-kit/src/declarative/table.rs::table_virtualized`.
    - It provides: fixed header + vertical virtualization + the headless pipeline (sorting/filtering/pagination/selection/visibility).
    - Public surface: `DataTable`.
    - Current virtualization mode: the shared view uses `VirtualListMeasureMode::Fixed` and
      `VirtualListKeyCacheMode::VisibleOnly` for the body rows (fast path for large tables; fixed row height).
      Variable-height rows are supported by the runtime virtualizer, but are not enabled by default for tables yet.
  - `DataGrid` is the recommended performance-ceiling surface (canvas-backed):
    - API alias: `fret-ui-shadcn::DataGrid` -> `DataGridCanvas` (see `ecosystem/fret-ui-shadcn/src/lib.rs`).
    - Implementation: `ecosystem/fret-ui-shadcn/src/data_grid_canvas.rs`.
    - Text caching: cell text blobs are keyed by `(row_key, col_key)` via `CanvasPainter.key_scope` + `child_key`.
  - `experimental::DataGridElement` prototype exists at `ecosystem/fret-ui-shadcn/src/data_grid.rs`.
    - It explores element-based 2D virtualization (rows + columns) and custom scrollbars.
  - `DataTable` and `DataGrid` used to be behind the `datagrid` crate feature; the gate has been removed because it had no heavy deps.
- `ecosystem/fret-ui-kit`
  - TanStack-inspired headless engine already exists (always available; `table` is a no-op compatibility flag):
    - Implementation: `ecosystem/fret-ui-headless/src/table/*` (crate: `ecosystem/fret-ui-headless`)
    - Re-export surface: `ecosystem/fret-ui-kit/src/headless/mod.rs` (`pub use fret_ui_headless::table;`)
    - `fret-ui-kit` Cargo feature: `table` (retained for compatibility; does not gate compilation).
  - There is also a substantial declarative table view implementation (also always available):
    - `ecosystem/fret-ui-kit/src/declarative/table.rs` (uses `headless::table` and `fret-ui` primitives).
- Forms
  - `ecosystem/fret-ui-shadcn/src/form.rs` is a taxonomy facade over field primitives; it is not a headless form state engine.
  - A dedicated headless form state module exists in `fret-ui-headless` (re-exported via `fret-ui-kit::headless`):
    - Implementation: `ecosystem/fret-ui-headless/src/form_state.rs` + `ecosystem/fret-ui-headless/src/form_validation.rs`
    - Re-export surface: `ecosystem/fret-ui-kit/src/headless/mod.rs` (`pub use fret_ui_headless::form_state;` / `form_validation;`)
    - Declarative registry/wiring: `ecosystem/fret-ui-kit/src/declarative/form.rs`
- Calendar / Date Picker
  - A shadcn-aligned `Calendar` + `DatePicker` exist in `ecosystem/fret-ui-shadcn/src/{calendar,date_picker}.rs`.
  - `DatePicker` is implemented as a `Popover` + `Calendar` recipe and should be validated against APG keyboard/a11y outcomes.

## Milestones

### M0 — Audit + Plan Lock (this week)

- [x] Read ADR 0101 and translate into a v1 API sketch.
- [x] Identify current `data_table` feature gate and its dependency reasons (if any).
- [ ] Decide the minimal semantics role strategy (Grid vs Table vs ListBox) for v1.
- [x] Record reference SHAs for all new `repo-ref/*`.

### M1 — Headless Table Core (v1)

- [ ] Audit the existing `fret-ui-kit` headless engine (`headless::table`) for ADR 0101 parity gaps.
- [ ] Lock the minimal v1 public API surface (types + invariants) and add/adjust unit tests as needed.
- [ ] Validate stable row keys + selection + sorting tie-breakers + column visibility semantics with tests.
- [x] Decide the feature gate story:
  - `headless::table` has no heavy deps, so it is always available (no long-lived feature gate).

### M2 — DataTable UI Surface (shadcn-aligned)

- [x] Implement `DataTable` composition in `ecosystem/fret-ui-shadcn`.
- [x] Remove the `datagrid` feature gate (no heavy deps).
- [x] Add headless-backed `DataTable`.
- [x] Add a demo page in `apps/fret-examples` to validate interaction outcomes (`apps/fret-examples/src/datatable_demo.rs`).
- [x] Add recipe controls: `DataTableToolbar` + `DataTablePagination` (wires `global_filter`, `column_visibility`, `pagination`).
- [x] Add baseline keyboard navigation for `DataTable` (focusable list container + active-descendant + Arrow/Home/End/Page keys; Enter/Space toggles).
- [x] Add range selection + typeahead navigation for `DataTable` (headless-aligned, widget-agnostic).

### M3 — Forms (headless + shadcn wrappers)

- [x] Headless `FormState` core (dirty/touched/errors/submitting).
- [x] Validation hooks (sync v1): field registry + submit lifecycle + revalidate-on-change glue (`fret-ui-kit::declarative::form::FormRegistry`).
- [x] shadcn wrappers (`Form`, `FormField`, and control decoration for `Input`/`Textarea`/`Select`-style triggers).

### M4 — Calendar / Date Picker

- [x] Calendar date math core (month grid + month navigation).
- [x] shadcn `Calendar` surface + `DatePicker` recipe (`Popover` + `Calendar`).
- [x] DatePicker trigger formatting (PPP-style, shadcn-aligned default; customizable).
- [x] Validate `FormField` decoration against `DatePicker` trigger (button/pressable) in `form_demo`.
- [ ] Keyboard/a11y outcomes review against APG; add targeted tests where feasible.

### M5 — CanvasDataGrid (performance ceiling)

- [x] Add a headless 2D viewport module (row/col visible ranges + offsets) with unit tests.
- [x] Add a canvas-backed `DataGridCanvas` surface (experimental) with:
  - fixed/estimated sizing (v0),
  - optional caller-provided variable sizes (v0),
  - minimal text cell rendering (v0).
- [x] Add a stress/demo harness that renders large grids without UI-node blowup (and documents limits).

#### Glide Data Grid notes (perf / architecture)

Glide’s renderer architecture is a useful reference for “spreadsheet scale”:

- Separate header canvas + content canvas, and optionally double-buffer + blit scrolling deltas:
  - `repo-ref/glide-data-grid/packages/core/src/internal/data-grid/render/data-grid-render.ts`
- Column/row walking is done via procedural walkers and a `rowHeight` callback:
  - `repo-ref/glide-data-grid/packages/core/src/internal/data-grid/render/data-grid-render.walk.ts`
- Rendering code explicitly calls out future optimizations (retain-mode drawing, partial redraw caches, image workers).

## Open Questions / Decision Gates

- What is the v1 semantics role for DataGrid-like surfaces (Grid vs Table), and how does it interact with active-descendant?
- For v1, `DataTable` currently exposes list semantics (`SemanticsRole::List` + `ListItem`) and uses `active_descendant` for the highlighted row.
- How do we model cell focus vs row selection (and multi-select) without leaking policy into runtime?
- Clipboard contract for cells vs rows (tie to ADR 0041 / existing selection primitives).
- Do we want variable-height rows in `DataTable` (e.g. Markdown cells with wrapping)?
  - The runtime supports measured virtualization, but the current table view forces fixed-height rows for performance.
  - If we enable measured rows, we must validate scroll anchoring, key caching costs, and width-change reflow behavior.

## Decision: Performance-Ceiling DataGrid (Canvas/GPU-backed)

We will pursue a Glide-style architecture for the “upper bound” DataGrid:

- Headless: 2D viewport/range computation (rows + columns) as a pure algorithm module in `fret-ui-headless` (re-exported via `fret-ui-kit::headless`).
- Rendering: a canvas-backed grid that draws the visible cell region in a small number of paint passes (background, grid lines, text).
- Interaction: selection/focus/caret/drag handles as lightweight overlay layers; in-place editing via a single floating editor (popover/portal).
- API: `rows + columns + get_cell(row, col)` contract for on-demand data, with explicit revision/invalidation hooks.

We keep the existing element-based `DataTable`/`experimental::DataGridElement` for "rich cell UI"
scenarios and as a correctness reference.

## Consolidation Plan (Table/DataGrid Surfaces)

Status: In progress (consolidation landed; follow-ups tracked below)

We currently have multiple table/grid surfaces in `fret-ui-shadcn`. This section defines how we
intend to **converge** the public surface so users have a single obvious “default” path, while
preserving specialized variants and performance ceilings.

### Goal

- One recommended `DataTable` surface for “admin/settings/inspector” UIs (headless-backed, TanStack vocabulary).
- One recommended `DataGrid` surface for “spreadsheet density” UIs (canvas/GPU-backed).
- Keep element-based grids/tables available for rich cell UI, but do not position them as the default.

### Public mapping (current)

- **Headless (source of truth):** `fret-ui-kit::headless::table` (always available).
- **Default table recipe:** `fret-ui-shadcn::DataTable` is backed by the headless engine (ADR 0101) via the shared
  `fret-ui-kit` view: `ecosystem/fret-ui-shadcn/src/data_table.rs` calls
  `ecosystem/fret-ui-kit/src/declarative/table.rs::table_virtualized`.
- `DataTable` is the stable name.
- **Experimental grid prototype:** `experimental::DataGridElement` remains for rich cell UI and as a correctness/reference surface.
- **Simple preset:** the older “simple table” surface is not kept as a separate public type; any “simple” usage
  should be expressed as a `DataTable` preset/config (keeps the public surface small and forces configurability).
- **Default grid (performance ceiling):** `DataGrid` is an alias for `DataGridCanvas` (canvas-rendered; spreadsheet scale).

### Remaining decision gates

1) **A11y semantics:** `DataTable` should use Table semantics, `DataGrid(Canvas)` should use Grid semantics
   (recommended), but we should confirm this against our current semantics mapping support (ADR 0033 + ADR 0073).

### Follow-ups (to fully “close” the DataTable recipe)

- Profile and validate pagination + filtering at scale (now that we have a `TableViewOutput` contract for bounds),
  and confirm it behaves well under rapid filter input.
- Lock pagination reset rules in tests (global filter / sorting / column visibility reset `page_index` to 0; out-of-range
  indices are clamped by the table view output).
- Profile and validate large-table performance:
  - stable `items_revision` and cache invalidation behavior,
  - overscan defaults for typical inspector/admin tables,
  - confirm bounded viewport constraints remain correct (no accidental “render all rows”).

### Definition of done (consolidation)

- Docs: users can answer “which table do I use?” in < 30 seconds:
  - `DataTable` (headless-backed) vs `DataGrid` (canvas performance ceiling) vs `experimental::DataGridElement` (rich cell UI).
- API: public exports are stable and consistent (`lib.rs` tells the truth).
- Demos: at least one end-to-end demo validates:
  - sorting, filtering, pagination, selection, column visibility
  - virtualization is bounded (no unbounded children/layout blowups)

## Variable Size Support (Why v1 likely needs it)

Markdown-like cell content often implies:

- **variable row heights** (wrapping, multi-paragraph blocks, images),
- **variable column widths** (resizing, content-driven sizing),
- measurement that can change when width changes (reflow).

The reference implementations above converge on the same core idea:

- maintain an indexable “size -> offset prefix sum” structure per axis,
- provide an estimated size for unknown items,
- accept measurement write-backs and re-run visible-range queries,
- optionally apply a “large scroll scaling” layer to avoid precision/scroll-limit issues.

For our v1, we should explicitly support both sizing strategies:

- **Caller-provided sizes (preferred for huge grids):** row heights / column widths are provided by the data layer
  (possibly computed async). The grid is purely a consumer.
- **UI-measured sizes (fallback / smaller grids):** the grid measures visible content and writes back sizes, which
  implies invalidation and reflow risks (especially when widths change).

Implementation anchor (headless algorithm surface):

- Algorithm: `ecosystem/fret-ui-headless/src/grid_viewport.rs` (`GridAxisMetrics`, `GridAxisMeasureMode`,
  `GridAxisRange`, `compute_grid_viewport_2d`).
- Re-export surface: `ecosystem/fret-ui-kit/src/headless/mod.rs` (`pub use fret_ui_headless::grid_viewport;`).

## Performance Review (Initial)

Baseline observations (what we already have):

- Row virtualization exists and is robust:
  - `fret-ui` provides `VirtualList` with fixed-measure support and visible-only key caching.
  - `fret-ui-kit`’s `declarative::table` configures fixed row height (`VirtualListMeasureMode::Fixed`) and uses `VirtualListKeyCacheMode::VisibleOnly`.
- A TanStack-aligned headless engine already exists (`fret-ui-kit/headless::table`) and is memoized/unit-testable.
- `fret-ui-shadcn`’s `experimental::DataGridElement` prototype performs 2D virtualization (rows + columns) by computing
  visible ranges once per frame (via `fret-ui-kit::headless::grid_viewport`) and only instantiating
  visible cells (no per-row nested `VirtualList`).

What Glide Data Grid suggests (when pushing to “spreadsheet scale”):

- DOM-style per-cell UI nodes become the bottleneck when hundreds/thousands of cells are onscreen.
- A canvas/GPU renderer that draws cells in a single pass avoids per-cell node churn and improves scroll smoothness.

Likely next optimizations (if we need “million-row spreadsheet” class performance):

- Generalize the 2D range computation + absolute-position pattern into a reusable “virtual grid” primitive,
  so higher-level grids (tables, calendars, inspectors) do not need to re-implement the loop plumbing.
- Consider a “canvas-backed grid” mode:
  - render cell backgrounds/text via a single `Canvas`/paint pass for the dense region,
  - keep interactive editing/selection as lightweight overlay layers (selection rects, caret, editor popover).
- Keep the API extensible:
  - prefer a `rows + columns + get_cell(row, col)` contract (Glide-style) for the UI surface,
  - keep TanStack-like transforms (sort/filter/group/paginate) in the headless engine.

## Progress Log

- 2026-01-13: Created worktree `feat/ecosystem-table-forms-calendar` and initialized this workstream tracker.
- 2026-01-13: Cloned/checked out reference sources under `repo-ref/*` (commit SHAs recorded above).
- 2026-01-13: Completed initial audit of existing table/datagrid/forms/calendar surfaces (see “Current Code Surfaces”).
- 2026-01-13: Removed `fret-ui-shadcn` `datagrid` feature gate and validated with `cargo check -p fret-ui-shadcn` and `cargo nextest run -p fret-ui-shadcn`.
- 2026-01-13: Hardened `DataTable`/`DataGrid` vertical virtualization options for fixed row height (`VirtualListMeasureMode::Fixed` + `VirtualListKeyCacheMode::VisibleOnly`).
- 2026-01-13: Added headless-backed `DataTable` native demo (`apps/fret-examples/src/datatable_demo.rs`).
- 2026-01-13: Extended `headless::grid_viewport` to support “count + key_fn” axes (no need to allocate a `Vec<K>` for fixed/identity-key axes).
- 2026-01-13: Refactored `fret-ui-shadcn` `DataGrid` prototype to use `Scroll` + `headless::grid_viewport` (single range computation per frame; absolute-positioned visible cells).
- 2026-01-14: Started implementing shadcn `Calendar` + `DatePicker` (Calendar WIP; `time` dependency added to `fret-ui-shadcn`).
- 2026-01-14: Removed the long-lived `fret-ui-kit` `table` feature gate (kept the feature as a no-op compatibility flag) and updated ADR/docs accordingly.
- 2026-01-15: Enabled focus traversal for semantics wrappers (`SemanticsProps.focusable`) and made `Pressable` semantics respect `PressableProps.focusable`; wired `DataTable` keyboard navigation + active-descendant in `fret-ui-kit::declarative::table::table_virtualized` with tests.
- 2026-01-15: Added shadcn `FormField` helper to reduce wiring boilerplate; auto-decorates common controls (a11y labels + destructive focus/border on error).
- 2026-01-15: Extended `DataTable` interaction outcomes (range select + typeahead) and added `ColumnHelper::accessor_str` to support string-based typeahead.
- 2026-01-15: Updated `DatePicker` trigger label formatting (PPP-style default) and added `form_demo` fields (`Role` select + `Start date` picker) to validate `FormField` decoration.
