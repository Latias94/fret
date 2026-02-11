# ADR 0253: `delinea` Minimal Dataset Transform Node Set (v1)

Status: Proposed

## Context

ECharts supports a rich `dataset.transform` pipeline with derived datasets. In `delinea` v1 we
intentionally support a **narrow, declarative subset** that is:

- engine-owned (no translator-side eager table cloning),
- deterministic and budget-aware,
- compatible with raw-index identity (ADR 0252),
- and sufficient to unlock “real-world” transform chains in headless goldens.

Today, `fret-chart`’s ECharts translator supports a small subset of transforms, but it applies them
by cloning rows into a new table. This ADR defines the v1 engine-level surface we want instead.

## Decision

### 1) Dataset lineage becomes a first-class spec/model concept

`ChartSpec.datasets[*]` is extended to describe dataset lineage:

- Raw datasets are backed by a host `DataTable` in `DatasetStore`.
- Derived datasets reference an upstream dataset and carry a transform chain.

In v1, derived datasets are **single-parent** (one upstream dataset).

### 2) v1 supported dataset transforms (ECharts-aligned subset)

v1 supports the following dataset transforms (applied in order):

1) **Filter (numeric field)**: keep rows whose key satisfies one or more predicates:
   - `gte`, `gt`, `lte`, `lt`, `eq`, `ne`
2) **Sort (numeric field)**: reorder rows by a key:
   - `order = asc | desc`

Additionally, `fromDatasetIndex` chaining is represented by dataset lineage (`derived.from`).

Out of scope in v1:

- expression-based computed columns,
- multi-field sort,
- groupBy/aggregate,
- join/lookup across multiple datasets,
- transforms that change schema.

### 3) Schema/fields behavior (v1)

In v1, dataset transforms do not change schema. Therefore:

- Derived datasets inherit the same logical fields as their upstream dataset.
- Series encoding (`SeriesEncode`) continues to bind via `FieldId` as today.

This is intentionally strict to avoid early schema drift. When we introduce computed columns, we
will add explicit “derived field” nodes with stable identity.

### 4) Transform execution model (high level)

The engine represents each derived dataset as:

- a lineage root `DataTable` (from `DatasetStore`),
- plus a `RowSelection` mapping (`view_index -> raw_index`) computed by the transform chain.

This keeps raw-index identity stable (ADR 0252) and avoids cloning.

## Consequences

- The translator can emit dataset lineage + transforms declaratively.
- The engine becomes the source of truth for transform semantics and caching.
- Headless goldens can cover chained transforms without allocating new tables.

## Follow-ups

- ADR 0254: cache key + invalidation contract for dataset transform nodes.
- Add headless goldens that assert raw-index stability across filter+sort chains.
