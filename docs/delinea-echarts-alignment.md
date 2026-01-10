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

## Quick Manual Validation

The multi-axis harness (`apps/fret-examples/src/chart_multi_axis_demo.rs`) is the recommended
baseline for validating interaction semantics and span limits on both native and wasm.

### Native (desktop)

- Run the demo:
  - `cargo run -p fret-demo -- chart_multi_axis_demo`
- What to validate (P0):
  - X and Y axis band pan/zoom routing matches the active axis rules.
  - X span limits (`minSpan/maxSpan`) clamp interaction-derived zoom updates (wheel/slider/box zoom).
  - Y span limits clamp interaction-derived zoom updates (wheel/slider/box zoom) when `DataZoomYSpec` is configured.
  - When the current/base span is already outside the configured limits, interactions do not force it back into range
    (they only prevent moving further out of bounds).

### Web (wasm)

- Option A (recommended): use `fretboard` (wraps Trunk):
  - `cargo run -p fretboard -- dev web --demo chart_multi_axis_demo`
- Option B: run Trunk directly:
  - `cd apps/fret-demo-web`
  - `trunk serve`
  - open `http://127.0.0.1:8080/?demo=chart_multi_axis_demo`

See `apps/fret-demo-web/README.md` for prerequisites and the full list of demo names.

## Terminology Mapping (ECharts -> `delinea`)

- `option` -> `ChartSpec` (serializable durable config)
- `ecModel`/`SeriesModel`/`ComponentModel` -> `ChartModel` (validated graph + computed metadata)
- `axisProxy` -> `DataZoomXNode` / axis window policy helpers (`delinea::transform`)
- `dataZoom` inside/slider state -> `ChartState.data_zoom_x` + `ChartState.data_window_y` (ephemeral view state)
- `DataStore` + `getRawIndex` -> `RowSelection` + `RowSelection::get_raw_index`
- Rendered display objects (`zrender`) -> `MarksOutput` (renderer-agnostic mark batches)

## P0: Decisions That Must Stay Stable (Avoid Future Rewrites)

- `[x]` Stable “raw index” identity across transforms (`RowSelection` + indices) (ADR 0140).
- `[x]` Separate durable config vs ephemeral view windows (ADR 0129 / ADR 0136).
- `[x]` Large-data strategy is explicit (budgeted stepping; progressive marks) (ADR 0132).
- `[x]` Interaction contract is mark-based and allocation-aware (ADR 0133).
- `[x]` Multi-axis routing is deterministic (active axis pair; region routing) (ADR 0134).

## Conformance Scenarios (P0)

This section defines concrete behaviors that we treat as the baseline for refactors and new
features. The multi-axis harness (`apps/fret-examples/src/chart_multi_axis_demo.rs`) is the
reference demo for validating these scenarios on desktop + wasm.

### Active axis selection and routing

- Pointer movement over an axis band updates the active axis for that dimension:
  - Hover an X axis band -> active X axis becomes that band.
  - Hover a Y axis band -> active Y axis becomes that band.
- Interactions in the plot region act on the *active axis pair* (active X + active Y).
- When both a plot and an axis band are present under the pointer, axis bands take precedence for
  routing (axis region hit test happens before plot fallback).

### Pan (inside / axis band)

- Default chord: `LMB drag` pans.
- Dragging on the X axis band pans **X only** (active Y stays unchanged).
- Dragging on the Y axis band pans **Y only** (active X stays unchanged).
- Dragging in the plot region pans both axes by default.
- Plot pan constraints:
  - `Shift` constrains pan to **X only**.
  - `Ctrl` constrains pan to **Y only**.
- Pan lock gating:
  - If an axis is pan-locked, panning that axis has no effect.
  - If the axis range is fixed (`AxisRange::Fixed`), panning has no effect.

### Wheel zoom (inside / axis band)

- Wheel zoom applies to the axis under the pointer:
  - Wheel on X axis band -> zoom X only.
  - Wheel on Y axis band -> zoom Y only.
  - Wheel in plot -> zoom active axis pair (unless constrained).
- Plot zoom constraints:
  - `Shift` zooms **X only**.
  - `Ctrl` zooms **Y only**.
- Zoom lock gating:
  - If an axis is zoom-locked, wheel zoom and slider writes for that axis have no effect.
  - If the axis range is fixed (`AxisRange::Fixed`), zoom has no effect.
- Span limits (`minSpan/maxSpan`) for interaction-derived zoom writes:
  - If `DataZoomXSpec.min_value_span/max_value_span` are set, interactive zoom-in/out is clamped to those spans.
  - If the current/base span is already outside the limits, interactions do not force the span back into range
    (only prevent going further out of bounds).

### Box zoom (2D view window write)

- Default chord: `RMB drag` in plot starts a 2D box zoom.
- Accessibility chord: `Shift + LMB drag` can also start a 2D box zoom.
- Box zoom writes a single atomic action (`Action::SetViewWindow2DFromZoom`) for the active axis pair.
- If either axis is zoom-locked or fixed (`AxisRange::Fixed`), the gesture does not start.
- Box zoom expansion modifiers (ImPlot-like):
  - `Alt` expands selection horizontally to the plot edge.
  - `Shift` expands selection vertically to the plot edge.
  - If `Shift` is the required modifier to *start* the gesture (e.g. `Shift + LMB`), it does not
    also apply vertical expansion.
- Cancellation: while a box zoom drag is active, a plain `LMB down` cancels the drag.

### Brush selection (selection-only)

- Chord: `Alt + RMB drag` in plot starts a brush selection.
- Brush selection does not write view windows (it is selection-only in v1).

### DataZoom sliders (UI-only, v1)

- X slider:
  - Rendered for the active bottom X axis (if present).
  - Drag inside the window pans the window.
  - Drag the min/max handles resizes the window.
  - Clicking outside the window jumps to that location and continues as a pan drag.
  - Window writes use `Action::SetDataWindowXFromZoom` (and therefore respect zoom locks and span limits).
- Y slider:
  - Rendered for the active Y axis (left or right) based on axis-band routing.
  - Drag semantics match the X slider (pan + min/max handles + jump-to-click).
  - Window writes use `Action::SetDataWindowYFromZoom` (and therefore respect zoom locks and Y span limits, if configured).
- Axis range locks (`AxisRange::LockMin` / `AxisRange::LockMax`) also gate slider interaction:
  - If either bound is locked, window panning via slider drag is disabled.
  - If `LockMin` is present, the min-handle drag is disabled.
  - If `LockMax` is present, the max-handle drag is disabled.

### Lock toggles and view reset/fit

- Pointer chord: `Ctrl + LMB down` toggles pan+zoom lock for the axis under the pointer
  (or for both axes when the pointer is in the plot region).
- Keyboard shortcuts (plain keys, no modifiers):
  - `R`: reset view windows for the active axis pair.
  - `F`: fit view windows to data extents for the active axis pair.
  - `M`: toggle X filter mode for the active X axis (v1 debugging/control hook).
  - `A`: clear brush selection and slider drag state.

### AxisPointer / tooltip stability

- When multiple hits are equally close (distance ties within float epsilon), the chosen hover hit is
  deterministic and prefers earlier `series_order` (stable tooltip/marker behavior across refactors).
- Axis-trigger tooltips keep a stable row set and order:
  - First row is always the trigger axis label/value.
  - Then one row per visible series in `series_order`.
  - If a series cannot be sampled at the current axis value (missing/NaN/out of range), its value is `-`.

## Engine Architecture (Alignment Notes)

ECharts uses a staged pipeline and an axisProxy abstraction. One important property is that
**dataZoom filtering can be order-sensitive** when multiple dimensions are filtered
(`dataZoomProcessor.ts` documents “filter X, then reset/filter Y”).

`delinea` v1 intentionally diverges for performance:

- X dataZoom can filter rows (`FilterMode::Filter`) and drive selection.
- Y dataZoom is mapping-only in v1 (no row filtering) (ADR 0136).
- 2D zoom is expressed as a paired window write (`Action::SetViewWindow2DFromZoom`) without introducing
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
- `[x]` Time axis tick strategy aligned with ECharts defaults (ADR 0139; UTC-only labels in v1).

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
- `[x]` dataZoom sliders (UI-only) in `fret-chart`.
  - X: bottom X axis only.
  - Y: active Y axis (left/right) based on axis-band routing.
- `[x]` Y inside zoom/pan (wheel on Y axis band; drag pan constrained via axis band or plot modifiers) (ADR 0136).
- `[x]` 2D box zoom that writes `SetViewWindow2DFromZoom` for the active axis pair (ADR 0136).
  - Evidence: `ecosystem/fret-chart/src/retained/canvas.rs` (box zoom drag -> `Action::SetViewWindow2DFromZoom`).
- `[~]` 2D brush selection (selection-only, not a view window write).
- `[x]` `minSpan/maxSpan` policies for interaction-derived view window writes (ADR 0138).
  - Evidence: `ecosystem/delinea/src/engine/mod.rs` (span clamp in interaction actions) + `ecosystem/delinea/src/engine/window.rs` (span limiter).
  - Notes: implemented as value-space `DataZoomXSpec.min_value_span/max_value_span` (no percent space).

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

## Recommended Next Steps (P0 -> P1)

1. P0: Audit 2D box zoom semantics (axis routing + lock gating) using `apps/fret-examples/src/chart_multi_axis_demo.rs` (desktop + wasm) (ADR 0134/0136).
2. P0: Implement DataZoom Y “inside” semantics in `fret-chart` (axis band + plot modifiers), with lock gating (ADR 0135/0136).
3. P0: Decide whether brush selection should be promoted to a view-window write (box-zoom style) or remain selection-only (ECharts brush parity).
4. P1: Introduce span constraints and a durable `DataZoomYSpec` only if slider UI or persisted defaults are required (ADR 0138).
