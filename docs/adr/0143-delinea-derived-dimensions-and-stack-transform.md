# ADR 0143: `delinea` Derived Dimensions (Calculated Columns) and Stack Transform (ECharts-Inspired)

Status: Proposed (P0 decision gate)

## Context

`delinea` is intended to scale to “application charts” in the ECharts class:

- dataset-driven specs (`dataset` + `encode`),
- composable transforms (filtering, stacking, aggregation),
- large data + progressive work (`WorkBudget`),
- deterministic headless interaction (hit testing, axis pointer, tooltips).

Today we already support stacking (`SeriesSpec.stack`) for `Line`/`Area`, but the implementation lives in
the marks/hit-test stage. This is a deliberate P0 shortcut (ADR 0129), but it creates long-term risks:

- **duplication**: marks, hit testing, tooltip sampling, and bounds can each re-implement stacking logic.
- **caching pain**: stacking becomes “render-time work”, hard to reuse and hard to budget.
- **ECharts drift**: ECharts treats stacking as a data-processor stage that writes calculated dimensions
  (see `processor/dataStack.ts`), which downstream stages consume consistently.

ECharts solves this with `SeriesData` calculation dimensions:

- `stackResultDimension` (rendered value: cumulative sum),
- `stackedOverDimension` (base value: previous series stack),
- it intentionally avoids writing to raw source data because stacking depends on legend selection
  and series ordering.

We want to lock a similar direction early so future chart features do not require a large rewrite.

## Relationship to Other ADRs

- ADR 0128: headless engine + `WorkBudget`.
- ADR 0129: transform pipeline ordering and `dataZoom` semantics (notes current stacking shortcut).
- ADR 0132: large-data strategy and progressive work.
- ADR 0137: row selection and indices contract.
- ADR 0140: dataset storage + index-based views.
- ADR 0141: missing values and segment policy (`connectNulls` direction).

## Decision

### 1) Introduce “derived dimensions” as an internal transform output

We standardize the concept of a **derived dimension** (calculated column) that is:

- computed from raw dataset columns + current model state,
- cached by revision keys,
- produced under `WorkBudget`,
- consumed by all downstream stages (marks, bounds, hit testing, axis-trigger tooltip).

Terminology (internal, not necessarily public API):

- **raw dimension**: an input dataset column (e.g. `encode.y`).
- **derived dimension**: a computed column (e.g. `stacked_y`, `stack_base_y`).

Derived dimensions are engine-owned caches, not part of user datasets.

### 2) Derived dimensions must be keyed and cacheable

Every derived dimension is identified by a stable key that includes:

- dataset id + dataset revision,
- transform kind (stack, bin, aggregate, etc),
- series/stack group identity and configuration (e.g. `StackId`, `StackStrategy`),
- visibility/legend state that affects the result,
- encoding references (which fields are inputs).

This makes invalidation explicit and local, and avoids “mystery recomputes” on hover.

### 3) Stacking becomes a transform producing derived Y channels

We define a stack transform that produces, per stacked series:

- `stacked_y`: the rendered Y value in data space (cumulative sum).
- `stack_base_y`: the baseline Y value in data space (previous series’ `stacked_y`, or 0).

Consumers must use:

- `stacked_y` for coordinate mapping, hit testing, and axis-trigger tooltip sampling (match ADR 0133).
- `stack_base_y` for area fills and bands (filled region between base and top).

The original raw `y` value remains accessible via the raw dataset column.

### 4) v1 scope: stack-by-index only (ECharts “isStackedByIndex”)

To keep P0 tractable and fast, v1 stacking supports only stack-by-index:

- stacked series must share:
  - the same dataset,
  - the same `encode.x` field (and thus the same row alignment),
  - compatible axes.

This matches the common “timeseries table” ingestion model and avoids expensive joins.

P1+ may add stack-by-dimension (ECharts `stackedByDimension`) for category bars and joins, but only after
we lock the required lookup/indexing strategy (see Follow-ups).

### 5) Stacking must respect legend/visibility state

Stack results depend on which series are currently visible. Therefore:

- derived stack columns are computed based on the “active stacked series list” after legend filtering,
- toggling series visibility invalidates derived stack columns for that stack group.

This matches the ECharts constraint (“Should not write on raw data, because stack series model list changes
depending on legend selection.”).

### 6) Derived dimensions are computed under budget and can be progressive

Derived dimension builders must:

- be resumable across `step()` calls,
- reuse buffers where possible to avoid per-frame allocations,
- keep deterministic results independent of budget chunk boundaries.

We allow “block mode” for certain transforms (similar to ECharts data processing stages being blocked
in stream), but the implementation must still be expressed via `WorkBudget` so wasm stays responsive.

## Consequences

- Stacking becomes a reusable, testable data transform rather than a rendering-only feature.
- Hit testing, bounds, and axis-trigger tooltip can share the same derived values.
- We can add more ECharts-style transforms (aggregation, binning, smooth) without scattering logic across
  stages.

Trade-offs:

- Derived columns can be large (`O(n)` per series). We must be deliberate about:
  - which transforms materialize columns vs produce indices,
  - progressive thresholds and cache invalidation keys.

## Follow-ups

P0:

- Refactor current stacking implementation to a derived-dimension transform stage, keeping output parity.
- Add tests that validate:
  - stacked_y and stack_base_y determinism across budgets,
  - `StackStrategy::{SameSign,All}` behavior,
  - legend visibility toggles invalidate and recompute correctly.

P1:

- Decide stack-by-dimension strategy:
  - monotonic X fast path (`lower_bound`),
  - hash map index (`x_value -> raw_index`),
  - sorted indices view (`RowSelection::Indices` that is monotonic in X),
  - and how it interacts with missing values (ADR 0141).
- Add derived dimensions for:
  - bar layout (category offsets),
  - error bars / candlesticks (OHLC), and
  - statistical transforms (histogram/boxplot).

## References

- ECharts stack processor (calculation dimensions and `modify`): `F:\\SourceCodes\\Rust\\fret\\repo-ref\\echarts\\src\\processor\\dataStack.ts`
- ECharts `DataStore.ensureCalculationDimension(...)`: `F:\\SourceCodes\\Rust\\fret\\repo-ref\\echarts\\src\\data\\DataStore.ts`
- ADR 0129: `docs/adr/0129-delinea-transform-pipeline-and-datazoom-semantics.md`
- ADR 0137: `docs/adr/0137-delinea-row-selection-and-filtering-contract.md`
