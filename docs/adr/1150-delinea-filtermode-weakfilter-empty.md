# ADR 1150: `delinea` FilterMode `WeakFilter` / `Empty` (ECharts Parity)

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

Contract in `delinea` (future-facing, multi-dimensional):

- When filtering is applied across multiple dimensions, keep rows that have values on both “sides” of the window
  across the participating dimensions (ECharts’ “leftOut && rightOut” logic).
- For the current v1 encoding model (single `x` field per series), `WeakFilter` is treated as equivalent to
  `Filter` until we introduce multi-dimensional axis filtering / multi-field encodes.

#### `FilterMode::Empty`

ECharts intent: preserve indices but treat out-of-window samples as missing.

Contract in `delinea`:

- The filtered view keeps a stable row/index space (no row dropping).
- Samples that fail the axis filter predicate are treated as “missing” for mark generation:
  - line/area/band: breaks segments at window boundaries (no synthetic connections across excluded samples),
  - scatter/bar: do not emit a mark for excluded samples.
- Bounds/LOD ignore missing samples (so `Empty` behaves like `Filter` for bounds, but not for index stability).

`Empty` relies on the missing-values and segment-break policy tracked in ADR 1141.

### 3) Implementation direction (P0 -> P1)

We will implement these modes without materializing new columns:

- Filtering carriers:
  - `RowSelection::{Range, Indices}` remain the primary view selection types.
  - `Empty` introduces a “missing sample” carrier (conceptually equivalent to mapping a value to `NaN` in ECharts),
    implemented as a per-series predicate applied during mark emission (and by splitting line-family marks into
    multiple polylines when needed).
- Budgeting:
  - Any indices/masks that require scanning must be built under `WorkBudget` and cached by dataset revision +
    transform parameters (ECharts `DataStore._indices` direction).

## Consequences

- We gain a stable path to ECharts-class filtering semantics (especially for non-monotonic data and multi-dimensional
  transforms) without coupling to UI code.
- We keep the data layer zero-copy and portable.
- We add complexity to mark emission for `Empty` (segment splitting), but this is deterministic and testable.

## Follow-ups

- Implement `FilterMode::{WeakFilter,Empty}` behavior in `delinea` (initially for X dataZoom).
- Add conformance tests:
  - `Empty` breaks line segments across excluded samples.
  - `WeakFilter` behavior once multi-dimensional filtering is introduced.
- Update `docs/delinea-echarts-alignment.md` to track parity explicitly.

