# ADR 0208: `delinea` VisualMap + Data-Driven Styling Contract (ECharts-Inspired)

Status: Proposed (P0 decision gate)

## Context

Apache ECharts exposes `visualMap` as the primary mechanism for **data-driven visuals**:

- continuous or piecewise mapping,
- `inRange` / `outOfRange` styling,
- mapping one or more data dimensions into visual channels (color, opacity, symbol size, etc.),
- a controller UI (color bar / piece selector) that drives interactive range updates.

`delinea` already has:

- a headless data model (`ChartSpec` + `DatasetSpec` + `FieldSpec`) (ADR 0190 / ADR 0202),
- a staged transform pipeline with large/progressive budgets (ADR 0191 / ADR 0194),
- a mark output contract with stable data indices per emitted primitive (ADR 0193),
- a UI adapter layer (`fret-chart`) with token-driven styling (ADR 0131).

However, without a locked VisualMap contract, we risk:

- ad-hoc per-series coloring that cannot scale to dashboards,
- UI-only mapping logic that cannot be tested headlessly,
- a future rewrite when we need WebGL/WebGPU-style per-item attribute pipelines.

This ADR defines a minimal, allocation-aware VisualMap contract that can reach ECharts-class behavior
without forcing `delinea` to depend on UI/theme systems.

## Relationship to Other ADRs

- ADR 0190: headless engine vs UI adapter boundary.
- ADR 0193: marks identity and renderer contract (data indices in the mark arena).
- ADR 0194: large data + progressive stepping budgets.
- ADR 0202: dataset storage and stable raw index identity.
- ADR 0131: `fret-chart` theme tokens and style resolution.

## Decision

### 1) VisualMap is an engine-level transform (not UI-only)

VisualMap evaluation (mapping data values to visual channels) is owned by `delinea` so that:

- it is deterministic and testable without rendering,
- it can participate in budgeting (no hidden O(N) work in the adapter),
- it can be reused across multiple UI adapters (desktop + web).

The UI adapter remains responsible for rendering:

- the VisualMap controller widget (slider/legend UI),
- the final paint representation (token resolution, font/layout choices).

### 2) Spec surface: `VisualMapSpec` with explicit targets + dimension source

`ChartSpec` gains:

- `visual_maps: Vec<VisualMapSpec>`

Each `VisualMapSpec`:

- has a stable id (`VisualMapId`),
- defines a *target set* of series/datasets (v1: `series: Vec<SeriesId>` or `dataset: DatasetId`),
- identifies the input dimension as either:
  - a dataset field (`FieldId`), or
  - a derived dimension (stack base, normalized value, etc.) once those are formalized.

The target shape is intentionally explicit to avoid ECharts-style “finder” ambiguity in a typed
engine.

### 3) Channels: start with `color` + `opacity` (extensible)

VisualMap can write a small set of channels in v1:

- `color` (primary, most common ECharts usage),
- `opacity` (often combined with `outOfRange` dimming; v1 supports a per-bucket ramp composed with `out_of_range_opacity`),
- `point radius multiplier` (scatter-only; bucketized batches, adapter multiplies its base radius).
- `stroke width range` (scatter + bar; bucketized batches, adapter renders borders).

Future extensions (P1+) may add:

- point size / symbol size,
- per-item stroke width for polylines (line/area),
- per-item z-order hints,
- label formatting.

### 4) Output contract: bounded “buckets” + stable data indices

`delinea` already emits stable `data_indices` / `rect_data_indices` alongside the mark geometry.
VisualMap must preserve that property.

Because the current `fret-render` path pipeline does not support per-item color instancing, `delinea`
v1 will model VisualMap as **bucketed batches**:

- VisualMap evaluation produces a bounded number of buckets per series (configurable in the spec).
- The marks stage emits separate mark nodes per bucket with a single resolved style key (e.g. a
  palette index / paint id) and a contiguous geometry range in the mark arena.
- The per-item data indices remain intact for hit testing and external consumers.

This design:

- keeps adapter rendering simple (one color per draw),
- keeps CPU cost predictable (`O(N)` scan + bounded bucket count),
- stays compatible with a future GPU-instanced backend by allowing the bucketization strategy to
  evolve (without changing the spec semantics).

### 5) Style resolution boundary: headless “style keys”, adapter resolves paints

`delinea` does not resolve theme tokens. Instead, VisualMap outputs:

- a headless `VisualStyleKey` (e.g. `PaletteIndex(u16)` or `RampStop(u16)`), and
- optional metadata for UI controllers (e.g. normalized stops).

`fret-chart` maps `VisualStyleKey` to actual colors using:

- theme tokens (`chart.palette.*`, `chart.visualmap.*`) (ADR 0131),
- and explicit user-provided colors when the spec includes them.

### 6) Interactive updates: VisualMap controller writes actions

Interactive visualMap controllers (continuous range slider / piecewise toggles) are expressed as
headless actions:

- `Action::SetVisualMapRange { visual_map, range: Option<(f64, f64)> }`
- `Action::SetVisualMapPieceMask { visual_map, mask: Option<u64> }` (v1: bucket bitmask; `None` means “all selected”)

These actions:

- update ephemeral state (similar to dataZoom windows),
- invalidate the visual encoding stage and marks as needed,
- are budgeted under `WorkBudget` like other progressive stages.

## Consequences

- VisualMap semantics become stable early, preventing future UI-only divergence.
- v1 buckets enable usable ECharts-like color mapping without requiring per-item GPU instancing.
- The contract leaves room for future backends (WebGPU instancing) without changing the spec.

## Follow-ups

P0:

- [x] Add `VisualMapSpec` + ids + action surface to `delinea` (v1: series binding + buckets + selected range state).
- [x] Add unit tests for deterministic bucket assignment and opacity composition.
- [~] Add unit tests for budgeted stepping over large data.
- [x] Add initial `fret-chart` adapter support for palette-driven marks (via `PaintId`) and opacity scaling.

P1:

- [~] Add a `fret-chart` VisualMap controller UI (continuous + piecewise) with improved affordances (labels, reset, drag gestures).
- [x] Extend mapping to scatter point radius multipliers and bar fill bucket coloring.
- Evaluate whether `MarkArena` should gain optional per-item attribute streams once a GPU-instanced
  path exists.

## References

- Apache ECharts VisualMap implementation:
  - `F:\\SourceCodes\\Rust\\fret\\repo-ref\\echarts\\src\\component\\visualMap\\VisualMapModel.ts`
  - `F:\\SourceCodes\\Rust\\fret\\repo-ref\\echarts\\src\\component\\visualMap\\visualEncoding.ts`
  - `F:\\SourceCodes\\Rust\\fret\\repo-ref\\echarts\\src\\component\\visualMapContinuous.ts`
  - `F:\\SourceCodes\\Rust\\fret\\repo-ref\\echarts\\src\\component\\visualMapPiecewise.ts`
