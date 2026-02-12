# ADR 0210: `delinea` AppendData Semantics + Incremental Caches (P0 Baseline)

Status: Proposed

## Context

ECharts supports `appendData` and incremental updates. For “commercial-grade” charting, the system
must handle:

- streaming datasets (high-frequency append-only ingestion),
- interactive view/window updates (pan/zoom/tooltip) without full rebuild,
- large selections without per-frame allocations or full scans.

`delinea` already has a strong baseline:

- dataset revisioning (`DataTable.revision`, ADR 0202),
- budgeted stepping (`WorkBudget`, ADR 0190),
- pixel-bounded LOD for large series (ADR 0194).

However, several internal caches currently treat *any* dataset `revision` change as a “replace”
and reset work to the beginning of the selection range. This is correct but not scalable for
streaming charts.

We need a stable rule that lets caches **extend** work on append-only mutations while still being
safe when datasets are replaced.

## Decision

### 1) Define a dataset mutation taxonomy at the API boundary

`delinea` distinguishes dataset mutations by *which API is used*:

- **Append-only (streaming)**:
  - `DataTable::append_row_f64`
  - `DataTable::append_columns_f64`
  - Contract: existing rows are not modified; only new rows are appended.
- **Replace**:
  - `DatasetStore::insert` replacing an existing table,
  - `DataTable::clear`, `push_column`, or any direct mutation that can change historical rows.
  - Contract: caches must assume historical data may have changed.

This ADR intentionally makes “append-only safety” an **explicit contract**. If callers mutate
historical rows directly, caches may fall back to the replace behavior.

### 2) Append-only changes may update caches incrementally

For caches that are built by scanning a row range, an append-only dataset update:

- must not force a rescan of the historical prefix,
- should scan only the new rows that became visible/eligible.

This is implemented by storing `row_count` and the last completed `end_limit` inside cache entries.

When a cache sees `revision` change:

- If `row_count` increased and the previous scan range was fully covered up to the old `end_limit`,
  it resumes scanning from `old_end_limit` and keeps the existing accumulated results.
- Otherwise it resets to a full rebuild from `start`.

### 3) Cache key growth should reuse prefix work (when possible)

Some cache keys include an explicit `end` (often derived from `row_count`). Under append-only
streams, the “same logical view” may lead to a strictly larger `end`. In that case, caches should
reuse the best available prefix entry and continue scanning from its completion point instead of
starting from `start`.

## Scope (v1)

This ADR applies to “scan-based caches” used by the transform pipeline:

- `DataViewStage` (`XFilter` indices)
- `OrdinalIndexStage` (ordinal-to-raw inverted index)

Follow-ups (P1) may extend the same pattern to:

- append-aware incremental mark rebuild for `RowSelection::All` (line/scatter/bar),
- append-aware bounds extension for monotonic X ranges,
- background benchmarking gates for `appendData` workloads.

## Consequences

- Streaming charts can update without paying O(N) for each append.
- The engine remains deterministic and allocation-aware.
- The behavior is explicit: correct incremental behavior requires using the append-only APIs.

## References

- ECharts incremental update: `repo-ref/echarts/src/data/SeriesData.ts` (`appendData`)
- Dataset identity and raw index contract: `docs/adr/0202-delinea-dataset-storage-and-indices.md`
- Budgeted stepping: `docs/adr/0190-delinea-headless-chart-engine.md`
- Large data + progressive: `docs/adr/0194-delinea-large-data-and-progressive-rendering.md`
