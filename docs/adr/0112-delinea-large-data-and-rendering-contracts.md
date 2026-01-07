# ADR 0112: Delinea Large-Data Pipeline, Zero-Alloc Baseline, and Rendering Prerequisites

Status: Proposed
Scope: `delinea` (headless engine), `fret-chart` (UI adapter), and required scene/paint contracts for chart parity

## Context

ADR 0111 establishes `delinea` as the headless chart engine and `fret-chart` as the Fret UI integration layer.

If the long-term goal includes ECharts-class capability, we must lock a **performance and memory baseline** early:

- Large datasets should remain interactive (pan/zoom/hover) without per-frame O(N) work.
- Core operations should avoid per-frame heap allocations (“zero-alloc in steady state”).
- Progressive / incremental work should be first-class (bounded work per frame).
- The engine must remain portable and renderer-agnostic, emitting marks that can be mapped to `SceneOp` primitives.

ECharts solves the large-data problem via a staged scheduler (`Task` / `Pipeline`) that supports progressive work,
and via data representations (`Source` / `SeriesData`) that minimize repeated parsing and enable fast filtering.

Fret currently provides a robust retained invalidation model (ADR 0051) and a prepared path primitive (ADR 0080),
but some paint semantics required by charts (dashes, gradients, richer stroke controls) are intentionally deferred.

This ADR locks the headless-side contracts needed to avoid “big future rewrites” when scaling chart breadth.

## Goals

- Define a practical “zero-alloc steady-state” contract for `delinea`.
- Define large-data strategies per primitive (line/scatter/bar/heatmap) that are bounded by viewport pixels.
- Define progressive execution contracts between `fret-chart` and `delinea`.
- Identify renderer/scene prerequisites (what we must upgrade later) without coupling `delinea` to `fret-render`.

## Non-goals

- A complete ECharts feature set immediately.
- Committing to ECharts JSON option compatibility inside `delinea` (translation can be a separate crate).
- Implementing full gradient/animation semantics in v1 (only lock the extension points).

## Decision

### 1) `delinea` must support “zero-alloc steady state” for interaction frames

Definition:

- After an initial warm-up (first layout, first mark build, first cache allocation), repeated interaction frames
  (pointer move, wheel zoom, drag pan) should perform **no heap allocations** in the common case.

Contract:

- `delinea` maintains internal arenas/caches and reuses scratch buffers across updates.
- `delinea` APIs avoid returning freshly allocated `Vec` in hot paths unless explicitly caller-provided.

Recommended API shape (non-normative):

- `ChartEngine::step(input, budget, scratch, out) -> StepResult`
  - `budget`: max work units or time slice for this frame.
  - `scratch`: caller-owned reusable buffers (or engine-owned, but stable across frames).
  - `out`: a snapshot of marks + hit-test indices (written into reusable storage).

`fret-chart` is responsible for requesting additional frames while `StepResult::unfinished == true`.

### 1.25) Cache ownership and invalidation must be explicit (to avoid duplicated work)

To match ECharts-class interactivity, we must avoid rebuilding the same intermediate artifacts repeatedly.
`delinea` should have explicit cache layers and ownership rules:

- **Engine caches** (inside `delinea`):
  - derived columns / transform results,
  - row masks / filtered views,
  - LOD outputs (decimated polylines, binned aggregates),
  - hit-test accelerators.
- **UI/renderer caches** (inside `fret-chart` and renderer services):
  - `PathService` prepared paths keyed by stable mark identity + style + transform invariants,
  - text shaping and glyph atlas caches (Fret text system),
  - optional rasterization caches (future; see ADR 0055 subtree replay and any future “surface cache”).

Invalidation must be driven by explicit revisions rather than ad-hoc “clear everything” fallbacks.

### 1.5) Determinism is a first-class requirement (cache keys, progressive shards, stable output)

Large-data behavior must be deterministic across frames, otherwise users will see “jitter” while panning/zooming
and caches will thrash.

Contract:

- Given the same `(spec, theme revision, viewport, state)`, the engine produces the same mark ordering and IDs.
- Any sampling strategy that drops points (scatter sampling, line LOD) must be deterministic:
  - either purely derived from data indices,
  - or uses a stable seed derived from `(ChartId, SeriesId, data revision)`.
- Progressive execution must be resumable without changing output ordering across slices.

### 2) Progressive execution is a first-class contract (Scheduler stages)

We adopt a staged pipeline conceptually aligned with ECharts:

1. **Data stage**: dataset parsing, transforms, filters, and derived columns.
2. **Layout stage**: coordinate system layout (grids/polar/etc), axes/ticks measurement, component placement.
3. **Visual stage**: palette assignment, visualMap mappings, emphasis/selection styles.
4. **Marks stage**: geometry generation and hit-test index generation.

Each stage:

- has an invalidation key (revision),
- can be partially executed under a budget,
- can resume on the next frame without recomputing completed work.

`fret-chart` provides a frame budget signal and receives “unfinished” feedback.

### 2.5) Budget units and fairness must be decided (time vs work units)

ECharts mixes “data length thresholds” (large mode) with progressive tasks that run over slices of data.
For `delinea`, we must decide:

- whether budgets are expressed as:
  - **time slices** (e.g. `Duration`), or
  - **work units** (e.g. “max points processed”), or
  - a hybrid (preferred; stable work units with optional time backstop).
- fairness rules across series/components:
  - avoid starvation (tooltip/axis updates should not lag behind huge series),
  - avoid “N charts kill the frame” (budget should be per chart instance).

Recommendation:

- Use work-unit budgets per stage (points processed, labels measured, marks emitted),
  and allow `fret-chart` to cap execution by time as a secondary guard.

### 3) Data representation must support fast filtering, slicing, and LOD without copying

We introduce `delinea::data` with a columnar-first internal representation:

- `DataTable`: columnar store (`Column<T>` for numeric/string/datetime/categorical).
- `RowRef` / `RowIndex`: cheap row addressing without materializing per-row structs.
- `DatasetId` / `TransformId`: stable identifiers for referential integrity.

Key requirements:

- Filtering yields a `RowMask` / `RowIndexList` view without copying columns.
- Series access is “by view”:
  - `SeriesView` references a `DataTable` + dimension indices + row set.
- Derived columns (transforms) are either:
  - lazily computed with caching, or
  - eagerly computed once per invalidation and stored as columns.

This mirrors ECharts’ `Source` / `SeriesData` split (format detection and dimension resolution once, then reuse).

### 3.5) “Spec update” semantics must be decided early (merge, replace, and stable identity)

ECharts’ `setOption` supports multiple update modes (merge, replaceMerge, notMerge, lazyUpdate) and relies heavily on
stable identity for animation and incremental updates.

Even if `delinea` does not implement ECharts JSON compatibility in v1, it must decide:

- How component/series instances are identified:
  - explicit IDs (preferred),
  - names (secondary),
  - index-based fallback (avoid for any stateful element).
- How partial updates apply:
  - merge fields into existing instances,
  - replace specific component families (ECharts `replaceMerge` concept),
  - reset-to-default vs keep-old semantics.

Recommendation:

- Require explicit IDs for all stateful instances in `ChartSpec` (series, axes, grids, dataZoom, visualMap).
- Provide a `ChartPatch` protocol that makes merge semantics explicit (per-field or per-component).
- Keep “ECharts compatibility translation” out of `delinea` core (a separate adapter crate can map ECharts rules).

### 3.75) Numeric types and normalization rules must be consistent (f32 vs f64, NaN/inf, domain clamps)

Charts frequently mix:

- large magnitudes (financial/time),
- small deltas (sensor data),
- log scales and clamped domains.

We must lock:

- the canonical internal numeric type for transforms and geometry math (`f64` recommended for headless math),
- how `NaN` / `±inf` are handled (skip, break segments, mark invalid),
- deterministic domain clamp rules (log/symlog constraints, empty-domain handling).

This prevents subtle cross-platform drift (especially wasm) and avoids “mystery discontinuities” in large datasets.

### 4) Large-data rendering is bounded by viewport pixels (not data length)

For each series kind, `delinea` must have an LOD strategy that bounds output complexity by viewport size.

#### Line / area (monotonic-X best path)

- Default: **min/max per pixel column** (two samples per x-pixel) to preserve spikes.
- Alternative: LTTB-like decimation for non-monotonic or sparse data, targeting `O(viewport_px_width)` points.
- Cache keys must include:
  - data revision,
  - visible domain window,
  - axis scale,
  - viewport width in device pixels,
  - series style keys that affect geometry.

#### Scatter

- Default: point budget per viewport (e.g. `max_points = k * viewport_px_area` with clamping).
- If count exceeds budget:
  - density map / binning fallback (heatmap-like),
  - or sampling with deterministic seed for stability while panning.

#### Bars / histograms

- For categorical bars, complexity is typically bounded by category count.
- For large category counts:
  - aggregate into bins at the axis tick resolution,
  - or apply a “min bar width” collapse strategy (bucket multiple categories into one visible bin).

#### Heatmap

- Represent as a grid aligned to pixels or bins; avoid per-cell marks when the grid exceeds the pixel budget.
- Prefer a single image/mesh-like mark representation (mapped to `SceneOp::ImageRegion` or a batched quad grid).

### 5) Hit testing must be scalable and layered

To keep hover/tooltip responsive on large data:

- Each series produces a hit-test accelerator:
  - monotonic X: segment index by x-interval (binary search to candidate segment range),
  - scatter: grid spatial hash (cell → point indices),
  - bars: category → bar bucket mapping.

The engine produces:

- a compact “best candidate” for the current pointer position,
- plus optional secondary candidates (for stacked series or multi-axis).

`fret-chart` only requests full tooltip payload materialization when needed (e.g. on stable hover),
to avoid string building on every pointer move.

### 5.5) Selection/highlight state must be modeled as reducers (not ad-hoc flags)

ECharts relies on a consistent global state model:

- highlight/downplay (emphasis/blur),
- select/unselect/toggleSelect,
- legend-driven visibility changes,
- visualMap-driven visual state,
- brush selections and link groups.

To avoid later rewrites, `delinea` should treat interaction state as reducer-driven:

- `ChartState` stores long-lived selections/highlights.
- `Action` is reduced into a new `ChartState` (pure-ish, deterministic).
- Visual encoding reads from `ChartState` to compute emphasis/blur/select styles.

This separation makes progressive rendering and caching tractable because “state changes” are explicit inputs to the
visual/mark stages.

### 6) Rendering prerequisites: lock extension points now, upgrade scene/paint contracts later

Fret’s current vector path contract (ADR 0080) intentionally fixes stroke joins/caps and has no dash pattern.
ECharts-class charts require at least:

- dashed strokes (grid lines, series styles),
- configurable joins/caps (visual fidelity),
- gradients (common in area, emphasis states, and modern themes),
- richer text/label layout (collision/overflow/rich text spans).

Decision:

- `delinea` mark styles must be modeled in a way that can **express** these semantics without changing the engine
  architecture later, even if `fret-chart` initially degrades them to solids.

Recommended modeling approach:

- `Paint` is a separate concept from geometry:
  - `FillPaint`: `Solid(Color)` | `LinearGradient(...)` | `RadialGradient(...)` | `Pattern(...)`
  - `StrokePaint`: `Solid(Color)` plus `StrokeStyle { width, join, cap, dash }`
- Marks reference `PaintId`s in a per-chart `PaintRegistry`.
- `fret-chart` maps supported paints to current `SceneOp` capabilities, and logs/flags degraded paints in debug.

Follow-up ADRs (not part of this file) should upgrade contracts:

- **Path v2**: dashed strokes + join/cap options (an extension to ADR 0080).
- **Paint abstraction**: gradients/patterns as a shared contract across UI and charts.
- **Rich text/label**: spans, background boxes, overflow, and collision policies.

### 6.5) Prepared geometry strategy must avoid pathological path churn

`SceneOp::Path` encourages reuse via `PathService` (ADR 0080). For charts, we must avoid:

- preparing a new path every frame for large polylines,
- preparing a new path for each hover highlight state,
- preparing separate paths for identical geometry across multiple charts.

Guidance:

- Base geometry should be prepared once per `(SeriesId, data_rev, lod_key, scale_key)`.
- Emphasis/hover/selection should ideally be expressed as **style changes** referencing the same geometry.
- When style requires different geometry (e.g. different stroke width affecting joins), treat it as a derived key and
  cache it explicitly, rather than “accidentally” rebuilding.

### 7) Text, formatting, and label layout must be treated as a subsystem (not incidental strings)

ECharts invests heavily in label behavior:

- numeric formatting (precision, SI units, percent),
- locale-aware formatting,
- rich text (inline styles/spans, background boxes),
- label overlap/collision avoidance,
- axis label rotation and truncation.

For Fret:

- `delinea` should model **formatters** as pure functions over typed values:
  - `FormatterId` references a registered formatter (closure or enum-based built-in).
  - formatter output should be reusable (cache by `(value, formatter id)` where possible).
- label layout should be its own stage or sub-stage:
  - it consumes measured text sizes from `fret-chart` and returns positioned text marks.
- tooltip content must avoid per-move string work:
  - compute the “tooltip payload” (typed values + references) first,
  - format to final strings only when showing tooltip.

This keeps pointer-move hot paths allocation-free while still enabling rich UX later.

### 8) Layering and z-order need a stable mapping to Fret scene layers

ECharts uses `z` / `z2` / `zlevel` concepts (zrender) to separate compositing layers and order within layers.

`delinea` should output explicit mark grouping:

- `LayerId` (small integer) for coarse layering (background, grid, series, overlays, tooltip).
- per-layer stable ordering keys for deterministic draw order.

`fret-chart` maps these to Fret’s scene layer stack (ADR 0079) and clip stacks (ADR 0019/0078).

### 9) Component dependency graph and topological execution order should be explicit

ECharts has implicit dependencies like:

- `xAxis/yAxis` depend on `grid`,
- `dataZoom` depends on axes/coord sys and filters series data,
- `visualMap` depends on series values and can affect legend/tooltip.

`delinea` should represent component dependencies explicitly:

- each component declares:
  - which coordinate systems it targets,
  - which series it affects (by ID or query),
  - which stages it participates in (data/layout/visual/marks).
- the scheduler executes component tasks in a deterministic topological order.

This prevents “hidden coupling” where adding a new component later forces a refactor of stage ordering.

### 10) Cross-chart coordination should be an explicit feature (ECharts `connect` / groups)

ECharts supports linking interactions across multiple chart instances (shared axisPointer, linked brushing, etc).
Editor-grade UIs also frequently require synchronized charts (time-series stacks, linked cursors).

Decision:

- `delinea` should define a **link protocol** early:
  - `LinkGroupId` identifies a synchronization group.
  - `LinkEvent` expresses domain window changes, cursor/hover positions, selection/brush extents.
- Linking is optional and implemented at the engine boundary:
  - `ChartOutput` emits link events.
  - the host (app or `fret-chart`) routes link events to other chart instances as `Action`s.

This keeps the engine deterministic and avoids hidden global state.

### 11) Time, media/timeline, and locale should be treated as engine inputs (not globals)

ECharts supports timeline/media options and locale-aware formatting. Even if we do not implement those features in
P0, we should define the inputs now to avoid implicit globals later:

- `LocaleId` and formatting conventions (decimal separators, month names, etc).
- `TimeZone` / time base (UTC vs local), and time tick policies.
- “Responsive” spec variants (media queries) as an explicit layer in `ChartSpec` or as app-controlled patches.

The goal is to ensure that the same spec renders identically given the same engine inputs, across desktop and wasm.

### 12) Observability and performance tooling must exist from day one

ECharts has a mature pipeline and makes performance tradeoffs per-series (large mode, progressive thresholds).
For `delinea`, we should lock:

- stage timing telemetry (data/layout/visual/marks),
- allocation counters (debug builds),
- cache hit/miss counters (LOD caches, label caches),
- a stress harness that replays scripted interactions (pan/zoom/hover) over large datasets.

This prevents “performance regressions by accident” while the feature set expands.

### 13) Web/WASM constraints must be reflected in the design

Even though Fret aims to support wasm, chart engines often accidentally rely on:

- threads,
- OS fonts enumeration,
- filesystem caches,
- or JS timers with high precision.

`delinea` should assume:

- single-threaded execution is the baseline,
- all “platform services” (text measurement, image decoding) are provided by `fret-chart` as explicit inputs,
- progressive budgets are expressed in a way that is stable under wasm timing variability.

### 14) Memory layout and ownership: avoid per-mark heap allocations

ECharts builds many ephemeral graphic elements, but amortizes work via pipelines and internal stores.
For `delinea`, we should prefer:

- arena-backed mark storage (`Vec` capacity reserved once, reused),
- IDs instead of owned `String` in hot paths (`StringId`/interning),
- small, copyable structs for mark headers; large payloads stored in side tables (SoA-friendly).

The UI adapter (`fret-chart`) should similarly:

- avoid allocating new strings for tooltip/labels on every move,
- reuse layout buffers and only rebuild when revisions change.

### 15) Concurrency policy should be explicit (single-thread baseline; optional parallel stages later)

Even on desktop, we should not assume background threads at P0 because wasm is single-threaded by default.

Decision:

- The engine runs correctly in a single-threaded model.
- If later we add parallelism, it should happen as an internal optimization (e.g. parallel data transforms) behind
  explicit feature gates, without changing the public contracts.

## P0 checklist (what we must decide before writing large amounts of code)

The following items should be considered “P0 decision gates” for `delinea`:

1. **Stage graph and invalidation keys** (data/layout/visual/marks) and what changes dirty which stage.
2. **Budget contract** (time-slice vs work-unit based) and how `fret-chart` requests more frames.
3. **DataTable representation**: numeric types, categorical encoding, datetime, and transform caching policy.
4. **LOD defaults**: per-series budgets and deterministic behavior (avoid “jitter” while panning).
5. **Hit-test indices**: which accelerators are required for which series types.
6. **Paint/Style modeling**: how to represent future dashes/gradients without rewriting mark generation.
7. **Spec update semantics**: merge/replace rules, stable IDs, and what qualifies as a breaking change.
8. **State reducers**: how selection/highlight/visibility changes flow through visual encoding.
9. **Label/formatting subsystem**: formatter IDs, measurement flow, collision policy, and caching boundaries.
10. **Layer mapping**: how marks map to scene layers/clips deterministically.
11. **Component dependency graph**: declarations, topo ordering, and stage participation.
12. **Cross-chart linking**: link events, group routing, and synchronization semantics.
13. **Locale/time/media inputs**: deterministic formatting and time axis behavior across platforms.
14. **Observability**: stage timings, allocation counters, cache metrics, and stress harness shape.
15. **WASM constraints**: platform services as explicit inputs; no hidden globals.
16. **Cache ownership**: engine vs UI/renderer cache boundaries and invalidation keys.
17. **Budget units**: stage work units and fairness across components/series.
18. **Numeric rules**: canonical numeric type and NaN/inf/domain clamp behavior.
19. **Memory layout**: mark storage strategy, string interning, and “no per-mark heap alloc” rule.
20. **Concurrency policy**: single-thread baseline and how parallelism would be introduced later.

## Consequences

- `delinea` can scale to large data with predictable costs and without “accidental allocations”.
- `fret-chart` can drive progressive execution using the existing animation-frame scheduling mechanisms.
- Renderer upgrades can be introduced incrementally via ADRs without destabilizing `delinea`’s core architecture.

## Implications for `fret-plot` (optional extensions to reduce future duplication)

`fret-plot` and `delinea` solve overlapping problems (LOD, hit testing, axes/ticks), but they target different UX
baselines (`fret-plot` is ImPlot-aligned cartesian plotting; `delinea` is a multi-component chart engine).

We should avoid forcing them to merge, but we can reduce future refactors by aligning a few contracts early:

1) **Shared headless primitives (preferred)**:

- Extract reusable algorithms into a small headless module/crate (either under `delinea` or a shared `fret-viz-core`):
  - tick generation, scale transforms (linear/log/time),
  - LOD/decimation primitives (min/max per pixel, LTTB-like),
  - hit-test accelerators for monotonic X and scatter grids.

This lets `fret-plot` improve performance without adopting the full chart engine model.

2) **Series interfaces for large data**:

`fret-plot` already anticipates view-range sampling in ADR 0099. To keep it competitive on large datasets:

- ensure `SeriesData::sample_range` (or equivalent) is a first-class API,
- allow chunked/streaming sources without copying into contiguous `Vec<DataPoint>`,
- keep “zero-alloc steady-state” for hover/pan/zoom by reusing scratch buffers.

3) **Paint semantics alignment**:

Once Path/Paint v2 lands (ADR 0113), `fret-plot` should adopt the same style model so:

- dashed reference lines and grid styles do not require segment workarounds,
- chart and plot visuals remain consistent under shared themes.

4) **Prepared path reuse strategy**:

`fret-plot` currently benefits from `PathService` caching. As we add more styles (dashes/joins/caps),
we should ensure `fret-plot` caches “geometry vs paint” separately (same as the guidance for charts):

- reuse geometry for emphasis/hover style changes,
- only rebuild when geometry truly changes (LOD/window/scale).

None of these require a redesign of `fret-plot`. They are small alignment points to prevent “two incompatible
performance stacks” from emerging.

## References

- ECharts scheduler and task pipeline: `repo-ref/echarts/src/core/Scheduler.ts`, `repo-ref/echarts/src/core/task.ts`
- ECharts option/data pipeline: `repo-ref/echarts/src/model/OptionManager.ts`, `repo-ref/echarts/src/data/Source.ts`, `repo-ref/echarts/src/data/SeriesData.ts`
- Charming (typed option surface): `repo-ref/charming/charming/src/lib.rs`
- Vector path contract (joins/caps fixed, no dashes in v1): `docs/adr/0080-vector-path-contract.md`
- Plot architecture and large-data baseline (ImPlot-aligned): `docs/adr/0099-plot-architecture-and-performance.md`
