# ADR 0114: Delinea Option Patching, Stable Identity, and the Scheduler Pipeline (ECharts-Inspired)

Status: Proposed
Scope: `ecosystem/delinea` (engine) and adapter crates (`ecosystem/fret-chart`, future `delinea-echarts`)

## Context

ADR 0111 establishes `delinea` as a headless chart engine and `fret-chart` as the Fret UI integration layer.
ADR 0112 locks the performance baseline (large data, progressive execution, “zero-alloc steady state”).

We want to borrow the best “mental model” from Apache ECharts:

- **Option updates** via `setOption`:
  - merge-by-identity by default,
  - whole-chart replace (`notMerge`),
  - replace-by-family (`replaceMerge`) where only items with matching IDs survive.
- **Runtime interactions** via `dispatchAction`:
  - actions mutate runtime state (zoom window, axis pointer, selection),
  - actions may request an update without rebuilding the entire option/model.
- **A staged pipeline** (scheduler + tasks):
  - processors and visuals are executed in a deterministic order,
  - large series can run progressively under a frame budget.

At the same time, `delinea` must remain “clean”: we do not want ECharts JSON compatibility quirks to leak into the
canonical engine model, because that quickly becomes a maintenance burden (stringly typed options, legacy aliases,
and merge corner-cases).

This ADR locks the update semantics, identity rules, and pipeline shape early so we avoid a large redesign later
when adding chart breadth (legend/tooltip/dataZoom/visualMap/brush, multiple coord systems, and 2D/3D expansion).

## Goals

- Provide an **ECharts-like update experience** (merge/replace/replaceMerge) without committing to ECharts JSON.
- Make identity and diff semantics stable enough for:
  - cache keys,
  - future transitions/animations,
  - stable selection/hover state across updates.
- Lock a scheduler pipeline shape that scales to many components and large datasets.
- Keep the engine portable (desktop + wasm) and renderer-agnostic.

## Non-goals (P0)

- Full parity with ECharts option JSON schema.
- A JS runtime bridge (that is the niche of Charming and is explicitly not the goal here).
- 3D parity in the initial engine (see “2D vs 3D”).

## Decision

### 1) Separate three state surfaces: `Spec`, `Model`, and `Runtime State`

We keep a strict separation:

- **`ChartSpec`**: user-provided declarative configuration (typed, stable, evolvable).
- **`ChartModel`**: the normalized internal representation derived from spec:
  - defaults applied,
  - indices resolved (dataset/axis refs),
  - stable identity assigned where missing (temporary only; see below).
- **`ChartState`**: runtime interaction state (zoom windows, pointer state, selections).

Rules:

- `ChartPatch` applies to spec/model structure (components/series/config).
- `Action` applies to runtime state (pointer/zoom/selection), never raw device events.
- Theme changes are treated as a **separate input revision** (see ADR 0102 / ADR 0050) and re-run visual stages.

This mirrors ECharts’ `setOption` vs `dispatchAction` split without importing ECharts’ option schema.

### 2) Adopt ECharts-inspired patch modes, but keep them typed and explicit

`delinea` supports patch modes aligned with ECharts semantics:

- `PatchMode::Replace`: replace the entire model (ECharts `notMerge: true`).
- `PatchMode::Merge`: merge by identity (ECharts default `notMerge: false`).
- `PatchMode::ReplaceMerge { families: ... }`: replace selected component families by identity (ECharts `replaceMerge`).

Where ECharts uses component `mainType` strings, `delinea` uses typed families:

- `ComponentFamily::CoordSys(CoordSysFamily)` (cartesian2d, polar, radar, …)
- `ComponentFamily::Axis(AxisFamily)` (x/y/angle/radius/…)
- `ComponentFamily::Series(SeriesFamily)` (line/scatter/bar/area/…)
- `ComponentFamily::Overlay(OverlayFamily)` (legend, tooltip, dataZoom, axisPointer, brush, …)

Rationale:

- It keeps the “replaceMerge mental model” while avoiding stringly typed logic in the engine.
- It makes diffs explicit and debuggable.

### 3) Identity rules: stable IDs first; never rely on indices for stateful elements

Identity is the cornerstone for:

- cache stability,
- progressive execution resumption,
- future transitions,
- selection/hover persistence.

Rules:

1. Prefer explicit `Id`:
   - `DatasetId`, `AxisId`, `SeriesId`, `ComponentId`, …
2. Allow `name` only as a secondary identity hint.
3. Index-based identity is a **last resort** and should be treated as “unstable”.

Engine behavior:

- In `Merge` mode, items are matched by:
  - `id` (strong),
  - `name` (weak),
  - index (fallback; use only when the item is known to be stateless).
- In `ReplaceMerge` mode:
  - only items with matching `id` are merged,
  - non-matching existing items in those families are removed (holes are allowed, preserving order stability).

This is directly inspired by ECharts’ component mapping behavior, but implemented on typed IDs.

### 4) Lock a deterministic scheduler pipeline with explicit dependencies

We adopt an ECharts-like staged pipeline (ADR 0112), but with two additional constraints:

- **Deterministic ordering**: tasks must run in a stable topo order to keep caches stable.
- **Explicit dependencies**: components declare dependencies; no hidden “if (has dataZoom)” coupling.

Pipeline stages:

1. **Data**: dataset parsing + transforms + filtering + derived columns.
2. **Layout**: coord system layout + axes + tick/label measurement.
3. **Visual**: palette assignment + visual mappings (including emphasis/selection state).
4. **Marks**: geometry generation + hit-test accelerators.

Each stage has:

- an invalidation key (revision),
- a progressive execution hook (budgeted work),
- a stable output revision (used by UI/renderer caches).

Large data policy is aligned with ECharts’ “pixel-driven sampling” idea:

- sampling rate derived from viewport pixel span,
- series-specific strategy (min/max per pixel, LTTB, binning),
- deterministic output across frames.

### 5) Model “dataZoom” as `DataWindow` + axis constraints (not as a UI widget)

ECharts separates:

- data window semantics (`dataZoom` processor, axis extents),
- UI presentations (slider, inside zoom, toolbox).

`delinea` matches this approach:

- The canonical runtime window is `DataWindow` per `AxisId`.
- UI widgets (slider/handles) live in `fret-chart` and dispatch actions:
  - `SetDataWindowX/Y`,
  - `SetViewWindow2D` for box zoom.
- Axis constraints (auto/lock/fixed) are part of the spec/model, not UI state.

This keeps headless semantics portable and testable.

### 6) Keep ECharts JSON compatibility out of the canonical engine

If we want to ingest ECharts option JSON:

- Add `delinea-echarts`:
  - owns JSON parsing, legacy aliases, and ECharts merge quirks,
  - translates `EChartsOptionJson -> ChartSpec` (or a `ChartPatch`).

This mirrors how Charming models the ECharts schema, but instead of executing JS, we translate into a native model.

### 7) 2D vs 3D: plan for 3D, but isolate it as a separate crate

ECharts 3D is largely delivered by a separate extension (`echarts-gl`) with distinct rendering concerns.

For Fret:

- Keep `delinea` focused on 2D in v1.
- Introduce a future `delinea-3d` (headless) + `fret-chart-3d` (UI adapter) when ready.
- Share only the lowest common contracts:
  - datasets/transforms,
  - option patching/identity,
  - theming and palette primitives.

This avoids dragging 3D dependencies and complexity into the 2D engine while still keeping an upgrade path.

## Consequences

- We get ECharts-like option update ergonomics without importing ECharts schema debt.
- Identity becomes a first-class contract; it protects both performance and UX consistency.
- The scheduler pipeline stays extendable as new component families land.
- 3D remains a planned extension with minimal surface area contamination.

## Alternatives considered

### A) Implement ECharts JSON option directly in `delinea`

Rejected: it makes the engine stringly typed and hard to evolve, and it bakes in legacy behavior that is not
aligned with Fret’s “clean, typed contracts” direction.

### B) “Plot-first” design: extend `fret-plot` into an ECharts-like chart engine

Rejected: it would pollute the plot surface with non-cartesian chart concepts and force `fret-plot` to carry
many incompatible policies and UI components.

### C) Make `Action` accept raw pointer/keyboard input events

Rejected: it makes headless behavior platform-specific, complicates keymap policy (ADR 0043), and blocks reuse
across different UI presentations.

## References

- ECharts `setOption` modes (`notMerge`, `replaceMerge`) and scheduler/tasks:
  - `repo-ref/echarts/src/core/echarts.ts`
  - `repo-ref/echarts/src/model/Global.ts`
  - `repo-ref/echarts/src/core/Scheduler.ts`
  - `repo-ref/echarts/src/processor/dataSample.ts`
- Charming (typed ECharts option builder; reference for schema ergonomics, not engine design):
  - `repo-ref/charming/charming/src/lib.rs`
- Fret:
  - `docs/adr/0097-plot-widgets-and-crate-placement.md`
  - `docs/adr/0099-plot-architecture-and-performance.md`
  - `docs/adr/0111-delinea-headless-chart-engine.md`
  - `docs/adr/0112-delinea-large-data-and-rendering-contracts.md`
  - `docs/adr/0113-painted-paths-and-paint-v1.md`
