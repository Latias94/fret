# `delinea` / Apache ECharts Alignment Checklist

This document tracks feature and semantics alignment between `delinea` (headless chart engine) +
`fret-chart` (UI adapter) and Apache ECharts.

Goal: **ECharts-class behavior and extensibility**, not API or option-schema parity.

Status symbols:

- `[x]` implemented
- `[ ]` planned / missing
- `[~]` partial / needs revision

## References

- Apache ECharts (`repo-ref/echarts`, commit `09198192b`)
  - `src/component/dataZoom/dataZoomProcessor.ts` (axis proxy + order-sensitive filtering)
  - `src/component/dataZoom/AxisProxy.ts` (data window computation + filter modes)
  - `src/chart/*` (series behavior + large/progressive knobs)
- `delinea` ADR series:
  - Headless engine: `docs/adr/0128-delinea-headless-chart-engine.md`
  - Transform pipeline + X dataZoom: `docs/adr/0129-delinea-transform-pipeline-and-datazoom-semantics.md`
  - Axis scales + mapping: `docs/adr/0130-delinea-axis-scales-and-coordinate-mapping.md`
  - Marks contract: `docs/adr/0131-delinea-marks-identity-and-renderer-contract.md`
  - Large-data + progressive: `docs/adr/0132-delinea-large-data-and-progressive-rendering.md`
  - Interaction + hit testing: `docs/adr/0133-delinea-interaction-and-hit-testing-contract.md`
  - Multi-axis + layout: `docs/adr/0134-delinea-multi-axis-and-layout-contract.md`
  - Axis locks + shortcuts: `docs/adr/0135-delinea-axis-interaction-locks-and-shortcuts.md`
  - DataZoom Y + 2D semantics (v1 divergence): `docs/adr/0136-delinea-datazoom-y-and-2d-semantics.md`
  - Row selection + filtering: `docs/adr/0137-delinea-row-selection-and-filtering-contract.md`
  - DataZoom composition + span policy: `docs/adr/0138-delinea-datazoom-component-composition-and-span-policy.md`
  - Dataset storage + indices: `docs/adr/0140-delinea-dataset-storage-and-indices.md`

## Terminology Mapping (ECharts → `delinea`)

- `option` → `ChartSpec` (serializable durable config)
- `ecModel`/`SeriesModel`/`ComponentModel` → `ChartModel` (validated graph + computed metadata)
- `axisProxy` → `DataZoomXNode` / axis window policy helpers (`delinea::transform`)
- `dataZoom` inside/slider state → `ChartState.data_zoom_x` + `ChartState.data_window_y` (ephemeral view state)
- `DataStore` + `getRawIndex` → `RowSelection` + `RowSelection::get_raw_index`
- Rendered display objects (`zrender`) → `MarksOutput` (renderer-agnostic mark batches)

## P0: Decisions That Must Stay Stable (Avoid Future Rewrites)

- `[x]` Stable “raw index” identity across transforms (`RowSelection` + indices) (ADR 0140).
- `[x]` Separate durable config vs ephemeral view windows (ADR 0129 / ADR 0136).
- `[x]` Large-data strategy is explicit (budgeted stepping; progressive marks) (ADR 0132).
- `[x]` Interaction contract is mark-based and allocation-aware (ADR 0133).
- `[x]` Multi-axis routing is deterministic (active axis pair; region routing) (ADR 0134).

## Engine Architecture (Alignment Notes)

ECharts uses a staged pipeline and an axisProxy abstraction. One important property is that
**dataZoom filtering can be order-sensitive** when multiple dimensions are filtered
(`dataZoomProcessor.ts` documents “filter X, then reset/filter Y”).

`delinea` v1 intentionally diverges for performance:

- X dataZoom can filter rows (`FilterMode::Filter`) and drive selection.
- Y dataZoom is mapping-only in v1 (no row filtering) (ADR 0136).
- 2D zoom is expressed as a paired window write (`Action::SetViewWindow2D`) without introducing
  sparse selections (ADR 0136).

## Feature Checklist (ECharts-Class Cartesian Charts)

### Data model & transforms

- `[x]` Dataset + field indirection (`DatasetSpec` + `FieldSpec`) (ADR 0140).
- `[x]` `encode`-style mapping (series `x/y/y2` fields) (ADR 0128).
- `[x]` Row range gating (`SetDatasetRowRange`) for external virtualization (ADR 0137).
- `[x]` X filtering via `FilterMode` (`Filter` / `None`) (ADR 0129).
- `[~]` Multi-dimensional filtering with sparse selections (ECharts `weakFilter/empty`) (deferred; ADR 0137 follow-ups).

### Axes, scales, and grids

- `[x]` X/Y axes with explicit kind + placement (`AxisKind`, `AxisPosition`) (ADR 0130).
- `[x]` Value scales + mapping windows + axis ranges (ADR 0130).
- `[~]` Category axis with stable ordinal index mapping under zoom (works for bar/axis pointer; needs DataZoom Y workstream).
- `[ ]` Time axis tick strategy aligned with ECharts defaults (tracked in ADR 0139; implement + validate).

### Series types (cartesian)

- `[x]` Line
- `[x]` Area
- `[x]` Band (filled range between `y` and `y2`)
- `[x]` Bar (vertical + horizontal)
- `[x]` Scatter
- `[~]` Candlestick / OHLC (engine-level support TBD; `fret-plot` has a demo but is a separate stack)

### Components: tooltip / axisPointer / legend / dataZoom

- `[x]` Axis pointer (ECharts-like `trigger=item/axis`) (ADR 0133).
- `[~]` Tooltip content parity (series ordering, formatting hooks, value snapping) (in progress).
- `[~]` Legend semantics (series visibility) (engine supports `SetSeriesVisible`; UI parity TBD).
- `[x]` X dataZoom inside (wheel/drag zoom/pan) (ADR 0129).
- Evidence: `ecosystem/fret-chart/src/retained/canvas.rs` (axis-band pan, plot modifiers, and window writes).
- Demo: `apps/fret-examples/src/chart_multi_axis_demo.rs` (multi-axis interaction conformance harness; desktop + wasm).
- `[x]` X dataZoom slider (UI-only) in `fret-chart` (bottom X axis only).
- `[x]` Y inside zoom/pan (wheel on Y axis band; drag pan constrained via axis band or plot modifiers) (ADR 0136).
- `[x]` 2D box zoom that writes `SetViewWindow2D` for the active axis pair (ADR 0136).
  - Evidence: `ecosystem/fret-chart/src/retained/canvas.rs` (box zoom drag → `Action::SetViewWindow2D`).
- `[~]` 2D brush selection (selection-only, not a view window write).
- `[ ]` `minSpan/maxSpan` policies for view windows (ADR 0138 follow-up).

### Performance & large data

- `[x]` Explicit progressive stepping budget (ADR 0132).
- `[x]` No per-frame allocations in core stages (target; enforce via tests/benchmarks over time).
- `[~]` Series-specific LOD / downsampling strategies (scatter vs line vs bar) (needs a conformance harness).
- `[ ]` Append/update semantics (ECharts `appendData`) (deferred; likely needs dataset storage contract work).

### Styling & theming

- `[~]` Token-driven chart styling (tracked in `docs/adr/0142-fret-chart-theme-tokens-and-style-resolution.md`).
- `[ ]` VisualMap-style data-driven color mapping (ECharts `visualMap`) (future; depends on mark metadata + palette policy).

## Known Gaps vs ECharts (High Value)

- DataZoom Y + 2D zoom UX parity (inside + box zoom + reset behaviors).
- Category axis indexing under zoom for non-bar series.
- VisualMap (continuous/piecewise) and declarative color scales.
- Rich tooltip formatting and series-specific default formatting.

## Recommended Next Steps (P0 → P1)

1. P0: Audit 2D box zoom semantics (axis routing + lock gating) using `apps/fret-examples/src/chart_multi_axis_demo.rs` (desktop + wasm) (ADR 0134/0136).
2. P0: Implement DataZoom Y “inside” semantics in `fret-chart` (axis band + plot modifiers), with lock gating (ADR 0135/0136).
3. P0: Decide whether brush selection should be promoted to a view-window write (box-zoom style) or remain selection-only (ECharts brush parity).
4. P1: Introduce span constraints and a durable `DataZoomYSpec` only if slider UI or persisted defaults are required (ADR 0138).
