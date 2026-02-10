# ADR 1195: `delinea` Transform Lineage + Raw-Index Identity (v1)

Status: Proposed

## Context

`delinea` aims to be an ECharts-class, headless chart engine with stable, testable contracts.
To support:

- derived datasets (`fromDatasetIndex` / dataset chains),
- dataset transforms (`filter`, `sort`, ...),
- brushing / hit testing / axisPointer sampling,
- and future cross-chart linking,

we need a **stable notion of identity** for individual data rows that survives transform chains.

ECharts models this via `DataStore` + optional `_indices` (view → raw mapping) and exposes
`getRawIndex(i)`. Today, `delinea` already has the building blocks (`RowSelection` +
`get_raw_index`) but the v1 translator currently **eagerly clones and re-indexes** transformed
datasets, which breaks raw-index stability across chained transforms.

This ADR defines the v1 contract for:

- what “raw index identity” means,
- which transforms preserve it,
- what downstream outputs must report,
- and which behaviors are intentionally out of scope.

## Definitions

- **Raw dataset**: a dataset backed by a concrete `DataTable` in the host-provided `DatasetStore`.
- **Derived dataset**: a dataset whose rows are produced by applying transforms to an upstream
  dataset (no eager table cloning required).
- **View index**: the index into the *current* dataset view (after transforms).
- **Raw index**: the stable index into the upstream raw table that a view row corresponds to.
- **Lineage root**: the upstream raw dataset that ultimately owns the `DataTable`.

## Decision

### 1) Raw-index identity is defined relative to the lineage root

For any dataset `D`, each visible view row `i` has an optional raw index `raw(i)`:

- For raw datasets: `raw(i) = i` (identity mapping).
- For derived datasets built from a single upstream root with row-preserving transforms: `raw(i)`
  maps into that root’s raw table.

This mapping must be **deterministic** under the same inputs (dataset revision + transform specs).

### 2) v1 supported transforms must preserve a single-source raw index mapping

In v1, only row-preserving, single-source transforms participate in the raw-index contract:

- `filter` (subset; preserves upstream order),
- `sort` (reorders; preserves membership),
- `fromDatasetIndex` chaining (lineage composition).

Transforms that **combine** rows (aggregate), **expand** rows (explode), or **join** multiple
sources are out of scope for v1 and are not required to provide a single `raw(i)` mapping.

### 3) Deterministic ordering rules (required for stable tests and caches)

For v1 `sort`:

- Ordering is derived from the chosen sort key (a numeric field in v1).
- Non-finite values (`NaN`, `±inf`) are ordered **after** finite values for ascending order and
  **before** finite values for descending order (so “valid numeric data” stays grouped).
- Ties must be broken deterministically. v1 uses:
  1) the sort key,
  2) then the upstream `raw(i)` as a stable tiebreaker.

For v1 `filter`:

- Output order preserves upstream order (stable filter).

### 4) Downstream outputs must report raw-index identity, not view indices

Any engine output that identifies a data row (marks, hit tests, axisPointer samples, brush
participation) must expose row identity in terms of the **raw index of the lineage root**.

This aligns with ECharts’ `dataIndex` vs `dataIndexInside` distinction: downstream consumers should
not need to know whether a dataset is derived to interpret row identity.

### 5) Mutation interaction (v1)

The raw-index contract assumes v1’s dataset storage invariants (ADR 1140 / ADR 1149):

- Append-only mutations preserve existing raw indices and append new ones at the end.
- Replace mutations invalidate caches and may change historical raw indices (because the table is
  replaced).

Derived dataset mappings must therefore:

- reuse work under append-only mutations when safe,
- and fully invalidate under replace semantics.

## Consequences

- Dataset transforms can be modeled as index mappings (`RowSelection::Indices`) rather than
  materialized table clones.
- Hit testing and interactive outputs remain stable under transform refactors.
- Headless goldens can assert raw-index identity across chained transforms.

## Follow-ups

- ADR 1196: define the minimum dataset transform node set for v1 and the spec/model surface.
- ADR 1197: define cache keys and invalidation boundaries for dataset transform nodes.
