# ADR 0111: Delinea Headless Chart Engine + `fret-chart` UI Integration

Status: Proposed
Scope: `ecosystem/delinea` (headless), `ecosystem/fret-chart` (UI), and the long-term boundary with `ecosystem/fret-plot`

## Context

Fret currently ships `fret-plot`, which targets an ImPlot/egui_plot-like **interactive cartesian plot** surface:
numeric axes, pan/zoom, hover/tooltip, and a small set of plot primitives (lines, bars, heatmaps, histograms).

We also want to eventually support **ECharts-class charts**:

- Many coordinate systems (cartesian, polar, radar, geo, parallel, calendar, …).
- A large set of non-series components (legend, tooltip, dataZoom, visualMap, axisPointer, brush, toolbox, …).
- A staged pipeline (data processing → layout → visual encoding → render) that supports incremental/progressive work.
- Declarative option/spec updates (`setOption`-style merges) plus runtime actions (`dispatchAction`-style interactions).

ECharts achieves this via a `GlobalModel` of `ComponentModel`/`SeriesModel`, a `CoordinateSystemManager`, and a
`Scheduler`/`Task` pipeline. Charming is a Rust library that provides a strongly typed ECharts **option builder**
and renders by executing ECharts JavaScript (browser/wasm/deno SSR). Charming is useful as a reference for
*spec ergonomics and completeness*, but it does not provide a native rendering engine.

For Fret, we want a native, portable implementation that renders via existing scene primitives (ADR 0002 / ADR 0097)
and remains `wgpu`/`winit` free in the ecosystem layer.

## Goals

- Provide a headless chart engine that can scale to an ECharts-like feature set without polluting `fret-plot`.
- Keep the engine portable (desktop + wasm) and independent of renderer backends (`fret-render` must remain swappable).
- Enable incremental/progressive work so “large charts” do not force pathological per-frame computation.
- Support stable IDs, diffing, and (later) animation without committing to ECharts JS internals.
- Keep theming aligned with Fret’s semantic token registry (ADR 0102) using namespaced chart tokens.

## Non-goals (P0)

- Full ECharts option compatibility in one leap.
- A faithful ECharts DOM/canvas renderer. Fret renders via `SceneOp::{Path,Quad,Text,ImageRegion}` (ADR 0097).
- A 3D chart renderer in the plot core. Correct 3D remains “viewport surface” (ADR 0098).

## Decision

### 1) Split charts into two crates: `delinea` (headless) and `fret-chart` (UI)

We introduce:

- `ecosystem/delinea` (crate name: `delinea`): a **headless chart engine** that owns:
  - spec/model merging and defaulting,
  - dataset and transforms,
  - coordinate systems,
  - layout of chart components,
  - visual encoding (palette, visualMap-style mappings),
  - mark generation (a renderer-agnostic “mark tree”),
  - hit-testing indices and interaction state reducers.

- `ecosystem/fret-chart` (crate name: `fret-chart`): the **Fret UI integration** that owns:
  - retained widget(s) (`ChartCanvas`),
  - mapping pointer/keyboard input into `delinea::Action`,
  - mapping `delinea` marks into Fret `SceneOp` primitives,
  - text measurement and font resolution (via Fret services),
  - theme token lookup using the semantic registry (ADR 0102).

This mirrors the successful separation in ECharts (engine vs renderer integration), but adapted to Fret’s layering
rules and retained invalidation model (ADR 0051).

### 1.1) Keep `delinea` free of compatibility baggage; isolate ECharts JSON compatibility

ECharts option JSON compatibility (including merge quirks, legacy aliases, and internal defaults) can be valuable,
but it tends to infect the entire engine with stringly-typed logic.

Decision:

- `delinea`’s canonical spec is `ChartSpec` (typed, stable, evolvable).
- If we want to accept ECharts option JSON, we add a separate adapter crate (e.g. `delinea-echarts`) that translates
  `EChartsOptionJson -> ChartSpec` and owns compatibility rules.

This preserves “clean code” while keeping a path to ECharts-flavored ingestion.

### 2) Keep `fret-plot` as the cartesian “plot core”; do not turn it into ECharts

`fret-plot` remains the ImPlot-aligned cartesian plot surface (ADR 0097/0099). ECharts-class charts live in
`delinea`/`fret-chart`.

Some features that appear in ECharts (e.g. pie, geo, radar) are **not cartesian plots** and should not be forced
into `fret-plot`. Conversely, `fret-plot` can continue to focus on scientific plotting and editor-like interaction
policies.

### 3) Use a staged pipeline in `delinea` (Scheduler/Tasks), with optional progressive work

`delinea` adopts the key idea of ECharts’ scheduler:

- Work is split into stages (data → layout → visual → marks).
- Each stage can be invalidated independently by spec/state changes.
- Stages may run incrementally under a time/step budget.

`fret-chart` requests animation frames while `delinea` reports “unfinished work”, similar to existing Fret patterns
for continuous-frame leases (ADR 0034).

### 3.1) Components declare dependencies; scheduler executes in deterministic topo order

ECharts relies on implicit dependencies (axes → grid, dataZoom → axes + series, visualMap → series values, etc).
To avoid hidden coupling as the feature set grows:

- `delinea` components must declare which stages they participate in and what they depend on.
- the scheduler executes tasks in a deterministic order derived from the dependency graph.

This makes large refactors less likely when adding new component families.

### 4) Define a stable internal model and an action protocol

`delinea` exposes two primary state surfaces:

- `ChartState`: caller-owned, long-lived interactive preferences (zoom windows, selection, highlight, pinned items).
- `ChartOutput`: derived snapshot (layout + marks + hover/tooltip candidates + revised IDs).

Interactions are expressed as `Action` (inspired by `dispatchAction`), not raw input events:

- `Action::Pan`, `Action::Zoom`, `Action::SetDataWindow`, `Action::HoverAt`, `Action::Select`, …

`fret-chart` maps pointer/key input into these actions, allowing consistent behavior across platforms and keymaps.

### 5) Theming uses semantic token keys with a chart namespace

Chart theming uses semantic keys and a namespaced extension surface:

- Prefer `fret.chart.*` tokens for chart-specific styling (grid, axis, series defaults, tooltip, legend).
- Provide fallback aliases (e.g. `chart.*`) if we later want third-party reuse similar to existing `plot.*` patterns.

This aligns with ADR 0102: semantic-first keys + extensible namespaces.

## Architecture (Recommended)

### A) Data model: spec vs resolved model

- `ChartSpec`: declarative user intent (serializable; “what the chart should be”).
- `ChartPatch`: partial update (merge/replace semantics; “how the chart changed”).
- `ChartModel`: resolved structure:
  - defaults applied,
  - stable IDs assigned,
  - cross-references resolved (e.g. series → coord sys),
  - derived fast paths prepared.

The engine never keys long-lived state by vector indices. IDs are stable across reordering and filtering.

### B) Coordinate systems as pluggable engines

We model coordinate systems similarly to ECharts’ `CoordinateSystemManager`, but with explicit layout contracts:

- `CoordSysKind`: `CartesianGrid`, `Polar`, `Radar`, `Geo`, …
- `CoordSysLayout`: computed pixel rectangles/anchors inside the chart viewport.
- `CoordTransform`: data → axis space → pixels, and inverse mapping for picking.

The coordinate system layer is headless; it receives a viewport and a text measurer (for tick labels).

### C) Components and series produce marks (not renderer ops)

`delinea` outputs a renderer-agnostic mark tree:

- `MarkTree` contains `MarkNode`s, each with stable `MarkId`s.
- Mark kinds include:
  - geometry marks (polyline, polygon/area, rect/bar, symbol, image region),
  - text marks (axis labels, legends, tooltips),
  - group/layer nodes (z-ordering and clipping scopes).

`fret-chart` maps marks into `SceneOp` primitives (ADR 0097). If the mapping requires richer paint semantics
(dashes, joins/caps, gradients), we upgrade the contracts via dedicated ADRs rather than hiding renderer-only behavior.

### D) Scheduler and caching

We adopt a task graph with deterministic invalidation keys:

- `data_rev`: dataset changes, transform parameters, filter ranges.
- `layout_rev`: viewport size, text metrics, component layout constraints.
- `visual_rev`: theme revision, palette/visualMap changes.
- `mark_rev`: derived from (data/layout/visual) plus series-specific style.

Each stage can cache outputs keyed by these revisions and a stable ID set. Progressive stages can be resumed.

### E) Interaction + hit testing

`delinea` builds hit-testing indices as part of the output snapshot:

- series-level candidates (nearest point/segment, bar bucket, heatmap cell),
- component-level candidates (legend item, axis label, dataZoom handles).

`fret-chart` owns pointer capture and translates UI events into `Action`. `delinea` computes:

- hover targets,
- tooltip payloads,
- highlight/blur state (ECharts-style emphasis is a future step).

### F) Stable layering and z-order (ECharts `z`/`z2`/`zlevel` analogue)

Charts need predictable layering:

- grid/axes behind series,
- overlays (crosshair/brush) above series,
- tooltip above everything.

`delinea` should output marks grouped by a small set of stable `LayerId`s and per-layer ordering keys. `fret-chart`
maps these to Fret’s scene layer stack and clip scopes (ADR 0019 / ADR 0079).

## ECharts alignment strategy (How to “eventually fully align”)

Full alignment should be treated as a long-term program, not a P0 contract. We propose:

1) **Feature parity first**, with a native spec:
   - Implement coordinate systems, components, and series behavior in `delinea`.
   - Keep the internal model stable and testable.

2) **ECharts option compatibility as a translation layer** (optional follow-up):
   - A separate crate (e.g. `delinea-echarts`) can parse ECharts option JSON and translate into `ChartSpec`.
   - This keeps `delinea` clean (no “stringly typed” compatibility burden inside the engine core).

This approach preserves “clean code” while still allowing us to accept ECharts-flavored inputs when desired.

## Phased implementation (recommended)

### P0 (bootstrap)

- Cartesian grid only (`xAxis/yAxis`, categorical + linear).
- Series: line, bar, scatter, area.
- Components: title, legend, tooltip (basic), axisPointer (crosshair), dataZoom (inside).
- Output marks: `Path/Quad/Text` only.
- Deterministic IDs + basic hit testing.

### P1 (dashboard-grade)

- visualMap continuous/piecewise (color mapping + legend integration).
- brush/select regions + selection outputs.
- progressive rendering for large scatter/line.
- richer label layout and collision avoidance.

### P2 (ECharts-class breadth)

- polar/radar/geo/parallel/calendar.
- markLine/markArea/markPoint equivalents (some may map to `fret-plot` overlays conceptually).
- animation and transitions (stable mark diff + interpolations).

## Alternatives considered

### A) Extend `fret-plot` into a full chart engine

Rejected: it blurs the plot/chart boundary and makes the plot core high-entropy, harming long-term maintainability.

### B) Embed ECharts JavaScript (Charming-style) for rendering

Rejected for Fret: it couples output to JS engines/DOM or SSR JS runtimes, and it bypasses Fret’s renderer/scene
contracts and portability goals.

### C) Put charts in `crates/` (kernel)

Rejected: chart semantics and interaction policy are ecosystem-level concerns; committing them as kernel contracts
too early increases long-term compatibility burden.

## Consequences

- We can evolve a high-level chart system without destabilizing `fret-plot`.
- We gain a path to ECharts breadth via a staged pipeline and a component registry.
- We will likely need follow-up ADRs for richer paint semantics (dashes/gradients) and mark-level animation.

## Fret substrate assessment (readiness and likely upgrades)

### What we already have (sufficient to bootstrap)

- Portable drawing primitives: `SceneOp::{Path,Quad,Text,ImageRegion}` (ADR 0097).
- Retained invalidation model suitable for incremental engines (ADR 0051).
- A semantic theme/token registry with namespaced extension keys (ADR 0102).
- Desktop + wasm demo harnesses (runner split; web runner exists in `apps/`).

### What will likely require contract or renderer work for ECharts-class parity

- **Stroke semantics**: dashed strokes, configurable caps/joins/miter limits, and consistent pixel snapping.
- **Gradients/patterns**: common in area charts and emphasis/hover states.
- **Label layout**: collision avoidance, overflow policies, rich text spans/backgrounds.
- **Mark-level animation**: stable element keys + diff + interpolation. This is larger than “request animation frames”.
- **Progressive rendering budgets**: a first-class budget signal from UI to engine, and engine feedback when work remains.

These should be handled via focused ADRs that extend scene/paint contracts with conformance tests, rather than
embedding renderer-specific behavior in the chart engine.

## References

- ECharts source (scheduler/model/coord sys): `repo-ref/echarts/src/`
- Charming (typed option builder): `repo-ref/charming/`
- Plot crate placement and primitive constraints: `docs/adr/0097-plot-widgets-and-crate-placement.md`
- Plot architecture and performance baseline: `docs/adr/0099-plot-architecture-and-performance.md`
- Plot3D viewport-surface strategy: `docs/adr/0098-plot3d-rendering-strategy.md`
- Semantic theme keys and extensible token registry: `docs/adr/0102-semantic-theme-keys-and-extensible-token-registry.md`
