# Fret Ecosystem Status (ADR + Code Audit Snapshot)

Status: Draft (notes only; ADRs remain the source of truth)

This document summarizes the current state of the Fret library + ecosystem by reading:

- ADRs under `docs/adr/`
- the crate layout under `crates/` and `ecosystem/`
- the current shadcn-aligned surface tracker: `docs/shadcn-declarative-progress.md`

It is intentionally **non-normative**. Treat it as a “where we are today” snapshot, not a contract.

## Authoring Cleanup Status (2026-03-16)

The pre-release authoring cleanup chain should now be read as a **closed sequence**, not as a set
of competing active lanes:

1. `authoring-surface-and-ecosystem-fearless-refactor-v1` closed the app/component/advanced lane
   split and the default teaching surface.
2. `ecosystem-integration-traits-v1` is now a maintenance closeout lane:
   the core trait-budget seams are landed and `QueryAdapter` is explicitly deferred for v1.
3. `into-element-surface-fearless-refactor-v1` is now a closeout / maintenance lane:
   conversion taxonomy collapse is landed, and the remaining work is raw-seam inventory plus
   source-gate upkeep.
4. `authoring-density-reduction-fearless-refactor-v1` is closed:
   the shorter default-path docs/templates/examples/gates are aligned.
5. `local-state-architecture-fearless-refactor-v1` is closed as a design-decision lane:
   `LocalState<T>` remains model-backed for v1.
6. `local-state-facade-boundary-hardening-v1` is closed:
   default local-state guidance, explicit raw-model seams, and explicit bridge APIs are now
   clearly classified.

Practical meaning:

- app/component/advanced tiering and the default authoring story are now stable enough to teach,
- authoring follow-up should stay maintenance- and evidence-driven rather than reopen broad surface
  redesign by default,
- the next truly active product work should come from still-open subsystem lanes such as text,
  font, shadcn alignment, docking, tables, and other editor-grade capability gaps rather than from
  restarting the already-closed authoring cleanup chain without new evidence.

## What Fret Is (and Is Not)

Fret is a documentation-driven, editor-grade UI runtime and ecosystem for Rust.

- `crates/fret-ui` is a **mechanism-only runtime** (elements, layout, paint, hit-test, semantics/a11y, overlays).
- `ecosystem/fret-ui-kit` hosts **tokens + reusable infra + headless state machines**.
- `ecosystem/fret-ui-shadcn` provides **shadcn/ui v4-aligned taxonomy + recipes** (composition + styling).

Non-goals for the core runtime:

- No monolithic “kitchen sink components” in `crates/fret-ui`.
- No policy-heavy interactions encoded as runtime behavior defaults (see ADR 0074).

## Current Architecture (High-Level)

### Runtime substrate (`crates/`)

- `crates/fret-core`: minimal cross-platform types/IDs.
- `crates/fret-runtime`: portable runtime services/value types.
- `crates/fret-ui`: the UI runtime contract surface (ADR 0066).
- `crates/fret-render`: wgpu renderer building blocks.
- `crates/fret-platform*`: platform I/O contracts + native/web implementations.
- `crates/fret-runner-*`: adapters for `winit` / web.
- `crates/fret-launch`: launcher glue.
- `crates/fret-framework`: facade crate (re-exports).

### Ecosystem (`ecosystem/`)

- `fret-ui-kit`: tokens/recipes/headless helpers (policy lives here, not the runtime).
- `fret-ui-shadcn`: shadcn-aligned component recipes and naming surface.
- `fret-docking`: docking interactions and persistence (ADR 0013 and related).
- `fret-icons*`: icon registries and packs.
- `fret-bootstrap`: golden-path startup layer (optional).

## “Where We Are Today” (Capabilities Snapshot)

### Declarative authoring baseline

- Declarative elements are the primary authoring model (ADR 0028, ADR 0039).
- Retained widgets are runtime-internal; component crates author declaratively.

### State management (authoring ergonomics)

Kernel primitives are solid (`Model<T>`, explicit invalidation, driver-boundary inbox draining), but
apps still need ecosystem-level ergonomics to avoid re-inventing patterns.

Current ecosystem surfaces:

- Typed UI ? app routing for dynamic per-item actions: `fret::payload_actions!` +
  `AppUi::on_payload_action*` (avoids `"prefix.{id}"` parsing and does not rely on per-frame router maps).
- Typed UI ? app routing for globally addressable actions: `fret::actions!` + `AppUi::on_action*`
  (stable `CommandId`s where keymaps/menus/palette integration matters).
- Async resource state (loading/error/cache/invalidation): `ecosystem/fret-query` (TanStack Query-like,
  adapted to ADR 0175 and `Dispatcher.exec_capabilities()`).
- Derived state (selectors/computed): `ecosystem/fret-selector` (memoized derived values with explicit
  dependency signatures + `use_selector` UI sugar).
  - Tracking: `docs/workstreams/state-management-v1/state-management-v1.md`

### Overlays / positioning

- There is a stable anchored overlay placement contract (ADR 0064).
- Component-owned dismissal/focus policies exist via action hooks (ADR 0074).

### Virtualization

- A stable virtualization vocabulary exists in the runtime (ADR 0070).
- Large-list constraints are explicitly documented (ADR 0042, ADR 0047).

### Tables / grids

What exists:

- shadcn `Table` primitives (`ecosystem/fret-ui-shadcn/src/table.rs`).
- shadcn `DataTable` (headless-backed): fixed header + vertical virtualization via the shared table view
  (`ecosystem/fret-ui-shadcn/src/data_table.rs` -> `ecosystem/fret-ui-kit/src/declarative/table.rs::table_virtualized`).
  - Recipe controls: `DataTableToolbar` + `DataTablePagination` (`ecosystem/fret-ui-shadcn/src/data_table_recipes.rs`).
- Note: the current shared table view uses fixed-height virtualization for rows by default (fast path).
  The runtime virtualizer supports measured (variable-height) rows, but table recipes have not enabled it by default yet.
- `DataGrid` default surface is canvas-backed (performance ceiling): `ecosystem/fret-ui-shadcn/src/data_grid_canvas.rs`
  (exported as `fret-ui-shadcn::DataGrid`).
- `experimental::DataGridElement` prototype exists for rich per-cell UI: `ecosystem/fret-ui-shadcn/src/data_grid.rs`.
- A TanStack-inspired headless engine exists in `fret-ui-kit` (ADR 0100) and is now integrated into the shadcn
  `DataTable` recipe, but still needs recipe-level widgets + validation for editor-grade ergonomics.
  - Note: the long-lived `fret-ui-kit` `table` feature gate has been removed; the table modules are now always available.

Recent work (workstream prototype surfaces):

- Headless 2D viewport helper for grid-style virtualization:
  - Algorithm: `ecosystem/fret-ui-headless/src/grid_viewport.rs` (re-exported via `fret-ui-kit::headless::grid_viewport`)
- Canvas/GPU-backed grid prototype to set a performance ceiling (Glide-style direction):
  - `ecosystem/fret-ui-shadcn/src/data_grid_canvas.rs`
  - Stress harness: `apps/fret-demo/src/bin/canvas_datagrid_stress_demo.rs`

## Ecosystem Gap Analysis (Compared to “Porting List”)

The following areas are the most “high leverage” gaps for editor-grade apps built on Fret:

### 1) Headless Table/DataGrid (ADR 0100 alignment)

Gap:

- The core headless-backed `DataTable` recipe exists (incl. toolbar + pagination), but it still needs
  large-table profiling/validation (especially variable-size row/col constraints and caching behavior).

Why it matters:

- Sorting/filtering/pagination/selection/column visibility are “B-side app defaults”.
- Stable IDs are foundational for virtualization + a11y semantics.

### 2) Variable-size virtualization (rows + columns)

Gap:

- v1 needs **variable row height** and **variable column width** (markdown + wrap + resize).

Why it matters:

- Without variable sizing, grids force fixed-height rows and break common content surfaces.
- Without explicit “reflow invalidation” on width changes, later refactors become unavoidable.

### 3) Forms (headless state + validation)

Gap:

- A form value binding + validation contract exists (`FormState` + `FormRegistry`), but it still needs broader shadcn integration
  beyond `Input` (e.g. `Select`, `Textarea`, `Checkbox` groups) and async validation ergonomics.

Why it matters:

- Forms are everywhere: settings panels, inspectors, property sheets, editor commands.
- Good form ergonomics prevents every app from reinventing error handling and lifecycle bugs.

### 4) Calendar / Date Picker

Gap:

- `Calendar` + `DatePicker` are now present in `fret-ui-shadcn` as a first-pass v1 surface.

Why it matters:

- It is a high-frequency input surface in admin/tools apps and needs correct keyboard/a11y outcomes.

### 5) Rich text / block editing (future)

Gap:

- No ProseMirror-like transaction core or Editor.js-style block model is present yet.

Why it matters:

- Markdown rendering and editing are “inevitable” in tools (docs panels, notes, changelogs).
- This is a larger investment; it should come after tables/forms/dates have stable contracts.

## Recommended Plan (Practical, v1)

### Phase A — Tables/DataGrid (correctness + portability)

- Align a shadcn `DataTable` recipe with the headless engine (ADR 0100):
  - sorting, filtering, pagination, selection, visibility
  - stable `RowKey` and derived `CellId`
- Keep the existing element-based `DataTable`/`experimental::DataGridElement` for "rich cell UI" use cases.

### Phase B — DataGrid performance ceiling (canvas/GPU-backed)

- Keep a canvas-backed grid as an explicit “upper bound” path:
  - constant-ish UI node count in large datasets
  - on-demand cell data access (`get_cell(row, col)`)
  - variable sizing supported as a first-class capability (row/col metrics + measurement write-back)
- Add clamp support (row height max / truncation strategy) as a v1 requirement.

### Phase C — Forms, then Calendar

- Add a headless form state layer in `fret-ui-headless` (re-exported via `fret-ui-kit::headless`) and shadcn recipes in `fret-ui-shadcn`.
- Implement `Calendar` + `DatePicker` using overlay primitives (Popover + Calendar recipe).
