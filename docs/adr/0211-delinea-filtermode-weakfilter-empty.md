# ADR 0211: `delinea` FilterMode `WeakFilter` / `Empty` (ECharts Parity)

Status: Proposed

## Context

`delinea` currently supports a minimal subset of Apache ECharts `dataZoom.filterMode`:

- `FilterMode::Filter`
- `FilterMode::None`

This is sufficient for monotonic-X “plot-like” workflows, but it leaves two ECharts modes undefined:

- `weakFilter`
- `empty`

In ECharts, these modes exist because “dropping rows” is not always a correct carrier:

- Multi-dimensional filtering may need to preserve segments that cross the view window.
- Some series types and transform combinations (stacking, category bars, linked selections) need stable
  indices so derived values remain consistent even when data is “out of view”.

Reference implementation (ECharts):

- `repo-ref/echarts/src/component/dataZoom/AxisProxy.ts` (`filterData`, `filterMode`)

Fret-specific constraints:

- `delinea` must remain deterministic and testable.
- The solution must be portable (desktop + wasm).
- Large data performance must remain bounded (budgeted work, allocation-aware).

## Decision

### 1) Extend `FilterMode` with ECharts-aligned variants

We extend `delinea::spec::FilterMode` to include:

- `Filter` (existing)
- `None` (existing)
- `WeakFilter` (new)
- `Empty` (new)

This is a durable chart semantics decision: it must live in the chart spec/model, not in widget-local policy.

### 2) Define semantics in terms of transform output + missing-value policy

`FilterMode` affects:

- which raw rows participate in bounds / LOD / marks,
- how line-family marks handle discontinuities,
- whether downstream transforms observe a stable “view index” space.

We standardize the meaning of each mode:

#### `FilterMode::None`

- No filtering is applied for the data window.
- Bounds/LOD remain global (subject to dataset row gating).

#### `FilterMode::Filter`

- Rows that do not satisfy the axis filter predicate are removed from the filtered view.
- Bounds/LOD/marks/axisPointer sampling are computed on the filtered view.
- The filtered view may be represented as:
  - a continuous `RowRange` when monotonic-X heuristics allow, or
  - an indices-backed `RowSelection::Indices` when monotonic slicing is not available and caching is beneficial.

This matches ECharts `filter` (selectRange / filterSelf).

#### `FilterMode::WeakFilter`

ECharts intent: do not drop data that may be needed to preserve continuity when multiple dimensions are involved.

Contract in `delinea`:

- **1D (X-only) semantics (v1 baseline):** `WeakFilter` is treated as equivalent to `Filter`.
  - This keeps the option surface stable while we iterate on multi-dimensional parity.
- **Multi-dimensional semantics (v1 subset, size-capped):** when a series has both an active X filter and an
  active Y filter, and both dimensions are configured as `WeakFilter`, the engine may materialize an
  indices-backed selection under `WorkBudget` to express a joint participation contract.
  - This is currently implemented as an **intersection-style** indices carrier (not full ECharts `leftOut/rightOut`
    continuity rules yet).
  - Materialization is gated by series kind (cartesian, non-stacked) and view-size caps to keep v1 bounded.
  - If the indices carrier is pending or skipped (budget/cap), the engine falls back to the window/mask carriers
    for marks + sampling.

#### `FilterMode::Empty`

ECharts intent: preserve indices but treat out-of-window samples as missing.

Contract in `delinea`:

- The filtered view keeps a stable row/index space (no row dropping).
- `Empty` is expressed as a typed per-series mask (`SeriesEmptyMask`) computed during view rebuild, so marks
  emission and axisPointer/tooltip sampling share the same missing-sample policy.
- Samples that fail the axis filter predicate are treated as “missing” for mark generation:
  - line/area/band: breaks segments at window boundaries (no synthetic connections across excluded samples),
  - scatter/bar: do not emit a mark for excluded samples.
- Bounds/LOD ignore missing samples (so `Empty` behaves like `Filter` for bounds, but not for index stability).

`Empty` relies on the missing-values and segment-break policy tracked in ADR 0203.

### 3) Implementation direction (P0 -> P1)

We will implement these modes without materializing new columns:

- Filtering carriers:
  - `RowSelection::{Range, Indices}` remain the primary view selection types.
  - `Empty` introduces a “missing sample” carrier (conceptually equivalent to mapping a value to `NaN` in ECharts),
    represented as a typed per-series mask (`SeriesEmptyMask`) and consumed consistently by:
    - marks emission (line-family segment splitting, and non-emission for point-like marks),
    - axisPointer/tooltip sampling (masked samples return `Missing`).
- Budgeting:
  - Any indices/masks that require scanning must be built under `WorkBudget` and cached by dataset revision +
    transform parameters (ECharts `DataStore._indices` direction).

## Consequences

- We gain a stable path to ECharts-class filtering semantics (especially for non-monotonic data and multi-dimensional
  transforms) without coupling to UI code.
- We keep the data layer zero-copy and portable.
- We add complexity to mark emission for `Empty` (segment splitting), but this is deterministic and testable.

## Follow-ups

- Validate `FilterMode::{WeakFilter,Empty}` semantics across both X and Y dataZoom, including size-cap behavior.
- Extend `Empty` parity to stacked series (and define the row/index stability contract for stacked value sampling).
- Expand multi-dimensional `WeakFilter` from the current v1 subset (intersection indices carrier) toward ECharts-class
  continuity rules (`leftOut && rightOut`) and document the resulting segment-break contract for line-family marks.

## Amendments

- 2026-01-13 (ECharts replica workstream): v1 gained a size-capped multi-dimensional `WeakFilter` subset via
  budgeted indices materialization in the filter processor step plan (`XYWeakFilter -> ... -> YIndices`).
  This should be treated as an optimization carrier and an intermediate contract on the path to full parity.

## Evidence (implementation anchors)

- Filter step plan + ordering: `ecosystem/delinea/src/engine/stages/filter_processor.rs`
- Budgeted indices views: `ecosystem/delinea/src/transform_graph/data_view.rs`
- Indices application and size caps: `ecosystem/delinea/src/transform/data_zoom_y.rs`,
  `ecosystem/delinea/src/transform_graph/y_indices.rs`
- Unified participation contract consumers: `ecosystem/delinea/src/engine/mod.rs` (marks + axisPointer + brush export)
