# ADR 0203: `delinea` Missing Values, Gaps, and Segment Policy (ECharts-Aligned)

Status: Proposed

## Context

To support “commercial-grade” charting, `delinea` needs a clear, deterministic policy for:

- missing values (e.g. `null`, `NaN`, `Infinity`),
- how missing values affect geometry (line/area/band/bar/scatter),
- how missing values affect interaction (tooltip, axisPointer, hit-testing),
- how missing values interact with LOD/progressive rendering.

ECharts has well-defined behavior for this, most notably:

- `connectNulls` in line-family charts,
- consistent “break the polyline at null” default behavior,
- downsampling that preserves `NaN` / “holes” instead of silently connecting across gaps.

Today, `delinea` relies on `f64` columns and treats non-finite values as “not a point” in many hot paths,
but the exact semantics are not specified and may vary by stage.

We want to lock this down early to avoid later rewrites of marks, hit-testing, and LOD.

## Relationship to Other ADRs

- ADR 0191: transform pipeline and dataZoom semantics.
- ADR 0194: large data + progressive rendering strategy.
- ADR 0195: interaction and hit-testing contract.
- ADR 0202: dataset storage + indices (raw store + views).

This ADR focuses specifically on **missing values and geometry segmentation**.

## Definitions

### Missing (v1)

A datum is considered **missing** for a numeric dimension when any of the following holds:

- the column value is not finite (`NaN`, `+/-Infinity`),
- the value is unavailable (column too short or out of range),
- the referenced column is not compatible with the requested numeric access path (e.g. not `f64`).

For v1, missing is represented as `NaN` in numeric code paths.

### Gap

A **gap** is a discontinuity in a series caused by one or more missing points.

### Segment

A **segment** is a contiguous run of valid points rendered as a single polyline (or a filled area/band region)
in plot space.

## Decision

### 1) Default: missing values break segments (ECharts default)

For `SeriesKind::{Line,Area,Band}`, missing values create a gap:

- geometry is split into segments,
- the renderer must not connect points across the gap,
- axis-trigger tooltip values for that series at the gap are absent (see below).

This matches ECharts with `connectNulls = false` (the default).

### 2) Optional: `connectNulls`-style behavior (P1)

We plan to add a per-series flag aligned with ECharts:

- `SeriesSpec.connect_nulls: bool` (default `false`).

When enabled:

- missing points are skipped and the polyline continues across the gap,
- LOD and hit-testing must behave as if the series were defined only on valid points.

This ADR does not implement the field yet; it locks the semantics and the naming direction.

### 3) Bars and scatter

- `SeriesKind::Bar`:
  - a missing Y value produces no bar for that row.
  - a missing X/category produces no bar.
- `SeriesKind::Scatter`:
  - missing X or Y produces no point mark.

No implicit interpolation is performed.

### 4) Stacking and missing values

For stacked line/area (ADR 0194, `SeriesSpec.stack`):

- missing values do not contribute to the stack baseline for that row,
- stacked output at that row is considered missing for that series,
- other series in the same stack may still be valid at that row.

This keeps stacking deterministic and prevents “phantom” fills created by substituting 0.

Follow-up (P1): if ECharts parity requires it, we can consider a configurable policy for how missing values
interact with stacks, but the default remains “missing breaks the stacked series at that row”.

### 5) Tooltip / axisPointer output

- `trigger=Item`:
  - missing points are not hittable; no `AxisPointerOutput.hit` is produced for that series.
- `trigger=Axis`:
  - if the sampled X has no valid point (or interpolatable neighbors) for a series, that series contributes
    no tooltip line for that axisPointer frame.

UI adapters may display a placeholder (e.g. `—`) for absent values, but the headless output stays explicit:
absence is represented by missing entries.

### 6) LOD / progressive rendering must preserve gaps

Downsampling (ADR 0194) must not erase gaps:

- if a bucket contains only missing values, it produces no output for that bucket,
- if valid values exist on both sides of a gap, the output must still encode a segment break unless
  `connect_nulls` is enabled.

This is required to avoid misleading visuals in time series with sparse data.

## Consequences

- Geometry and interaction become deterministic across all stages.
- We align with ECharts’ user expectations (default breaks, optional connect).
- Future features (candlestick, boxplot) can reuse the same “missing” model without surprising behavior.

Trade-offs:

- Preserving gaps can increase segment counts and slightly increase draw-call complexity.
- Some sampling/LOD shortcuts must carry a “segment break” signal through the pipeline.

## Follow-ups

P0/P1:

- Add a per-series `connect_nulls` switch and route it through marks + hit testing + LOD.
- Add unit tests covering:
  - gaps in line and area geometry (break vs connect),
  - axis-trigger tooltip output for missing values,
  - stacked series behavior under missing values.

## References

- ECharts `connectNulls` option: `F:\\SourceCodes\\Rust\\fret\\repo-ref\\echarts\\src\\chart\\line\\LineSeries.ts`
- ECharts polyline handling with gaps: `F:\\SourceCodes\\Rust\\fret\\repo-ref\\echarts\\src\\chart\\line\\poly.ts`
- ADR 0191: `docs/adr/0191-delinea-transform-pipeline-and-datazoom-semantics.md`
- ADR 0194: `docs/adr/0194-delinea-large-data-and-progressive-rendering.md`
- ADR 0195: `docs/adr/0195-delinea-interaction-and-hit-testing-contract.md`
- ADR 0202: `docs/adr/0202-delinea-dataset-storage-and-indices.md`
