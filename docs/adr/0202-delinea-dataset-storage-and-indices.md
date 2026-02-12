# ADR 0202: `delinea` Dataset Storage, Indices, and Zero-Copy Strategy (ECharts-Inspired)

Status: Accepted (P0)

## Context

`delinea` aims to support “commercial-grade” charts:

- large datasets (100k–10M points),
- interactive pan/zoom + hover without frame hitches,
- streaming / incremental updates.

Today, `delinea` uses `DataTable` as a minimal, owned, columnar store:

- columns are `Vec<T>` (`Column::{F64,I64,U64,Bool,String}`),
- filtering/windowing is primarily expressed as a row range (ADR 0191),
- some large-data behavior is handled in marks/LOD stages (ADR 0194).

This works for P0, but it leaves a key gap compared to ECharts: **a shared, index-addressable data store**
that can represent filtered/selected/downsampled views without copying columns.

ECharts addresses this with `DataStore`:

- typed-array “chunks” per dimension,
- `_indices` for filtered views,
- `getRawIndex(i)` to map view indices back to raw storage,
- `appendData(...)` only on raw data (before indices are created).

We want to lock down an equivalent direction early, so future features (stacking transforms, brush
selection, progressive rendering, streaming) do not require a large rewrite of our data layer.

## Relationship to Other ADRs

- ADR 0190: overall headless engine architecture and `WorkBudget`.
- ADR 0191: transform pipeline + dataZoom semantics.
- ADR 0194: large data + progressive rendering strategy.
- ADR 0199: row selection and filtering contract.

This ADR focuses specifically on **dataset storage** and **index-based views**.

## Decision

### 1) Keep columnar storage as the foundation

We keep the core stance: hot paths operate on contiguous numeric columns.
`DataTable` remains a valid ingestion format for v1.

### 2) Introduce an internal “DataStore + indices” model (ECharts-inspired)

We standardize an internal model for dataset access that supports both:

- **raw storage** (identity indexing),
- **view storage** (filtered/selected/downsampled) via an index mapping.

Key concepts (terminology, not necessarily public API):

- `raw_count`: number of rows in underlying storage.
- `count`: number of rows visible in the current view.
- `indices: Option<IndexArray>`: when present, maps `view_index -> raw_index`.
- `get_raw_index(view_index)`: returns `raw_index` (identity when `indices=None`).

This model is the required substrate for:

- selection/brush (ADR 0199),
- transform outputs that should not materialize new columns,
- progressive rendering and chunking (ADR 0194),
- fast `indexOfRawIndex`-style operations (hover, linking).

### 3) Prefer index-based transforms over column materialization

When a transform can be expressed as “keep these rows”, it should produce indices, not new columns.

Examples:

- `dataZoom` filtering (`filterMode=filter`) -> indices (or a compact row-range fast path when monotonic).
- brush selection -> indices.
- filtering transforms -> indices.

Materializing new columns is reserved for transforms that truly compute new values (stacking, binning,
aggregation), and even then we should consider adding computed columns as additional dimensions rather
than cloning existing ones.

### 4) Streaming rule: “append only on raw data”

We adopt the same spirit as ECharts (“append on raw storage”), but the concrete shape differs:

- In `delinea`, `DataTable` is always the raw store.
- Indexed views are **ephemeral**, engine-owned caches (e.g. `RowSelection::Indices`) keyed by dataset
  revision + transform parameters (ADR 0190 / ADR 0191 / ADR 0199).

Therefore:

- appending rows is always performed on the raw store (`DataTable`),
- caches that depend on row indices must treat a dataset revision bump as invalidation and rebuild
  under `WorkBudget`,
- view indices never “own” storage, so they cannot go stale without a revision mismatch.

#### v1 ingestion API (contract surface)

For v1, we treat the following APIs as the stable, recommended mutation surface for streaming:

- `DataTable::append_row_f64(&mut self, row: &[f64])`
- `DataTable::append_columns_f64(&mut self, columns: &[&[f64]])` (column-major batch append)

Both:

- update `row_count` deterministically,
- bump `revision` so all dependent caches can invalidate,
- preserve “raw index identity” (existing rows keep their raw indices; new rows are appended at the end).

Direct mutation of `DataTable.columns` (pushing into vectors, truncation, etc.) is not treated as a
stable contract even if it is technically possible in Rust today. If a consumer needs to perform a
non-append mutation, it should do so via a dedicated “replace” helper (future work) so the engine
can treat it as a full invalidation event with clear semantics.

### 5) Zero-copy direction: shared column backing is allowed

To enable:

- cheap cloning of datasets/views,
- sharing a dataset across multiple charts,
- future integration with Arrow/Polars,

we explicitly allow (future) column backends beyond `Vec<T>`, e.g. `Arc<[T]>`/`Box<[T]>`/borrowed buffers.

This ADR does not mandate the exact Rust type yet, but it locks the invariant:

- “A column can be shared without copying and can expose `&[T]` to hot paths.”

If/when we evolve `Column`, we should avoid painting ourselves into an API corner (e.g. consider
`#[non_exhaustive]` early, and keep construction helpers rather than pushing users to match on variants).

### 6) Revision and invalidation remain first-class

The engine must continue to use revisions (`Revision`) to keep invalidation precise:

- dataset mutation (append/replace) bumps a dataset revision,
- transforms produce derived revisions,
- view windows (pan/zoom) are separate from data revisions.

This is required to keep hover/pointer updates from forcing full recompute.

## Consequences

- We can represent filtering/selection as indices, avoiding column copies.
- We align with ECharts’ proven “raw store + indexed views” model, which is known to scale.
- We make future streaming and progressive rendering decisions compatible with wasm constraints
  (single-threaded baseline, deterministic chunking).

Trade-offs:

- Indices introduce an extra level of indirection for `get(dim, i)` when filtering is active.
- Some transforms will still need materialized columns (stack/aggregate); we must be deliberate about
  which steps allocate.

## Follow-ups

P0/P1:

- Keep indices-based selection (`RowSelection::Indices`) and budgeted caching as the standard path for
  non-contiguous transforms.
- Consider introducing a dedicated internal store/view type (even if `DataTable` remains public) when we
  add derived dimensions (stack/aggregate) and need shared computed columns.
- Add unit tests that validate:
  - view indices are monotonic (when required),
  - `get_raw_index` mapping is correct,
  - `append` invalidation behavior matches the contract.

## References

- ECharts `DataStore` (chunks + `_indices` + `getRawIndex` + `appendData`):
  `repo-ref/echarts/src/data/DataStore.ts`
- ECharts `SeriesData` (view operations and downsampling helpers):
  `repo-ref/echarts/src/data/SeriesData.ts`
- ECharts `Source` (source formats and dimension discovery):
  `repo-ref/echarts/src/data/Source.ts`
- ADR 0190: `docs/adr/0190-delinea-headless-chart-engine.md`
- ADR 0191: `docs/adr/0191-delinea-transform-pipeline-and-datazoom-semantics.md`
- ADR 0194: `docs/adr/0194-delinea-large-data-and-progressive-rendering.md`
- ADR 0199: `docs/adr/0199-delinea-row-selection-and-filtering-contract.md`
