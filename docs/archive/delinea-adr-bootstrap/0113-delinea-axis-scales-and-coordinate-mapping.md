# ADR 0130: `delinea` Axis Scales + Coordinate Mapping Contract (Value/Category v1)

Status: Proposed

## Context

`delinea` is a headless, ECharts-inspired chart engine (ADR 0128). Today we support a narrow set of
cartesian 2D, line-family charts (`Line/Area/Band`) with a value axis model.

To unlock the next chart taxonomy step (especially `bar`, `stack`, and most ECharts demo-style charts),
we need to lock down a **stable axis scale and mapping contract** that:

- supports both **continuous** (`value`) and **discrete** (`category`) axes,
- defines how `dataZoom` and axis locks affect the visible **window** vs the full **domain**,
- yields deterministic tick positions and labels,
- is compatible with large-data strategies (binary search on monotonic axes, min/max-per-pixel LOD).

This ADR is intentionally scoped to cartesian 2D. Non-cartesian coordinate systems (polar/radar/geo)
remain out of scope.

Related context:

- ECharts has explicit scale types (`interval`, `ordinal`, `time`, `log`) in `src/scale/*`.
- Our ImPlot-like retained plot surface (`fret-plot`) has its own scale contract (ADR 0098),
  but `delinea` needs an engine-level contract tied to dataset-driven charts.

## Relationship to Other ADRs

- ADR 0128: `delinea` headless chart engine.
- ADR 0129: transform pipeline + dataZoom semantics.
- ADR 0096 / ADR 0098: ImPlot-like plot widgets (different goals and API surface).

## Decision

### 1) Axis scale kind is explicit in `AxisSpec`

Introduce an explicit scale kind:

- v1: `Value`, `Category`
- v2 (follow-up): `Time`, `Log`, custom user scales

This avoids “implicit axis type” drift, and keeps future extensions additive.

### 2) Split axis **domain** (data extent) from axis **window** (view extent)

For each axis we define:

- **Domain**: the full data extent implied by the current model after transforms
  (e.g. min/max of relevant series, or category count).
- **Window**: the current visible extent after applying `AxisRange` constraints and `dataZoom`.

Rules:

- Domain is computed from transformed data and is stable until data/transforms change.
- Window is derived from Domain + axis constraints + interactive components (e.g. `dataZoom`).
- `FilterMode::Filter` affects the domain because it changes the visible dataset.
- `FilterMode::None` does not affect the domain; only the window changes.

This matches the mental model in ECharts: “data extent” vs “dataZoom window”.

### 3) Coordinate mapping is a first-class engine API

`delinea` exposes a scale-aware mapping API used by:

- marks generation (series layout),
- ticks and label placement,
- axis pointer / tooltip sampling,
- hit testing.

The engine defines these operations per axis:

- `data_to_unit(value) -> f32` (unit is `[0, 1]` in axis direction, before pixel mapping)
- `unit_to_data(u) -> value` (when meaningful; e.g. `Category` maps to nearest index)
- `unit_to_px(u, plot_rect) -> f32` and `px_to_unit(px, plot_rect) -> f32`

The goal is not to expose a public “math trait” API yet, but to ensure the engine is structured around
a single mapping implementation instead of scattered per-feature conversions.

### 4) `Category` axis uses band semantics (required for bars)

`Category` is defined as an ordinal scale over an ordered list of categories.

v1 rules:

- Category positions are **center-of-band** by default.
- The band width is derived from the visible category count and plot rect size.
- Bar layout is defined against the category band, not against arbitrary pixel heuristics.

Category ordering:

- If `AxisSpec.categories` is present, it is authoritative.
- Otherwise categories are derived from the bound dataset field in **first-appearance order**
  (deterministic across runs for the same data).

This keeps initial behavior simple while still allowing explicit category lists for UI-driven or
application-driven ordering.

### 5) Tick generation and label formatting are scale-owned

Tick generation and tick label formatting are scale-owned responsibilities:

- `Value`: “nice” major ticks based on a target count and pixel budget, with stable rounding rules.
- `Category`: major tick per category by default, with label decimation based on pixel budget.

Formatting is performed in the headless layer (as structured text, not styled text), so the UI adapter
only applies fonts/colors/layout policy.

## Consequences

- We can add `bar` and `stack` without redesigning axis math later.
- Existing features (axis pointer `trigger=axis`, tooltip aggregation, dataZoom) must be refactored to
  use the unified mapping API to avoid future inconsistencies.
- A deterministic category ordering strategy must be implemented (explicit list or first-appearance).

## Follow-ups

P0 implementation work implied by this ADR:

1. Add `AxisScaleKind` to `AxisSpec` (v1: `Value`, `Category`).
2. Add a unified axis mapping API in the engine, and refactor existing axis-pointer + ticks to use it.
3. Implement category axis layout + label decimation.
4. Add `SeriesKind::Bar` (rect marks) using category band semantics.

P1 follow-ups:

- Add `Time` and `Log` scales (explicitly; no implicit guessing).
- Add multi-grid / multi-axis layout contracts (see ADR 0131 / ADR 0134 follow-ups).

## References

- ECharts scales: `F:\\SourceCodes\\Rust\\fret\\repo-ref\\echarts\\src\\scale\\Interval.ts`,
  `F:\\SourceCodes\\Rust\\fret\\repo-ref\\echarts\\src\\scale\\Ordinal.ts`
- ADR 0109: `docs/archive/delinea-adr-bootstrap/0190-delinea-headless-chart-engine.md`
- ADR 0110: `docs/archive/delinea-adr-bootstrap/0191-delinea-transform-pipeline-and-datazoom-semantics.md`
- ADR 0098: `docs/adr/0098-plot-architecture-and-performance.md`
