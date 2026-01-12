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
- Brush selection output: `docs/adr/0144-delinea-brush-selection-and-output-contract.md`
- Brush selection row-range fast path: `docs/adr/0145-delinea-brush-selection-to-row-selection-fast-path.md`
- Brush selection link events: `docs/adr/0146-delinea-link-events-for-brush-selection.md`
- VisualMap (data-driven styling): `docs/adr/0147-delinea-visualmap-and-data-driven-styling.md`

## Quick Manual Validation

The multi-axis harness (`apps/fret-examples/src/chart_multi_axis_demo.rs`) is the recommended
baseline for validating interaction semantics and span limits on both native and wasm.

### Native (desktop)

- Run the demo:
  - `cargo run -p fret-demo --bin fret-demo -- chart_multi_axis_demo`
- What to validate (P0):
  - X and Y axis band pan/zoom routing matches the active axis rules.
  - X span limits (`minSpan/maxSpan`) clamp interaction-derived zoom updates (wheel/slider/box zoom).
  - Y span limits clamp interaction-derived zoom updates (wheel/slider/box zoom) when `DataZoomYSpec` is configured.
  - When the current/base span is already outside the configured limits, interactions do not force it back into range
    (they only prevent moving further out of bounds).
  - BrushLink: `Alt + RMB drag` brush selection in one chart is mirrored into the other chart (same `LinkGroup`).

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

### Traceable P0 Scenario Checklist (Evidence-Backed)

This checklist is intentionally redundant with the sections below. Its purpose is to keep a
single “at a glance” view of:

- what we treat as P0-conformance semantics,
- where the behavior is defined (ADR),
- where it is implemented (evidence anchors),
- and what is still missing vs ECharts.

**S1 - DataZoom X inside + slider window writes** (`[x]`)

- ECharts reference: `repo-ref/echarts/src/component/dataZoom/AxisProxy.ts`, `repo-ref/echarts/src/component/dataZoom/dataZoomProcessor.ts`
- ADR(s): `docs/adr/0129-delinea-transform-pipeline-and-datazoom-semantics.md`, `docs/adr/0138-delinea-datazoom-component-composition-and-span-policy.md`
- Evidence: `ecosystem/delinea/src/engine/window.rs` (span limits), `ecosystem/delinea/src/engine/mod.rs` (interaction action routing), `ecosystem/fret-chart/src/retained/canvas.rs` (inside + slider gestures)
- Validation (desktop): `cargo run -p fret-demo --bin fret-demo -- chart_multi_axis_demo`
- Validation (wasm): `cargo run -p fretboard -- dev web --demo chart_multi_axis_demo`
- What to validate (P0):
  - Wheel on bottom X axis band zooms X only; span limits clamp zoom-in/out writes.
  - Drag on bottom X axis band pans X only.
  - X slider (bottom) supports pan drag + min/max handle drags + click-to-jump.
  - Locks (`Ctrl + LMB` toggle) gate slider + wheel writes for the locked axis.

**S2 - DataZoom Y + 2D zoom parity boundary (v1 divergence)** (`[~]`)

- ECharts reference: `repo-ref/echarts/src/component/dataZoom/*` (order-sensitive multi-dim filtering)
- ADR(s): `docs/adr/0136-delinea-datazoom-y-and-2d-semantics.md`, `docs/adr/0137-delinea-row-selection-and-filtering-contract.md`
- Evidence: `ecosystem/delinea/src/engine/mod.rs` (Y mapping windows), `ecosystem/delinea/src/transform/*` (RowSelection)
- Validation (desktop): `cargo run -p fret-demo --bin fret-demo -- chart_multi_axis_demo`
- What to validate (current v1 behavior):
  - Wheel on Y axis band zooms Y only; Y span limits (when configured) clamp interaction-derived writes.
  - 2D box zoom writes a paired window update (no sparse filtering materialization).
- Missing vs ECharts (high value):
  - Y-driven filtering semantics (and ordering rules when multiple dims are filtered),
  - ECharts-style “weakFilter/empty” behaviors for sparse selections,
  - 2D zoom interactions that can materialize sparse selections when needed.

**S3 - 2D box zoom writes an atomic paired window action** (`[x]`)

- ECharts reference: `repo-ref/echarts/src/component/dataZoom/*` + brush/interaction glue (behavioral reference only; implementation differs)
- ADR(s): `docs/adr/0136-delinea-datazoom-y-and-2d-semantics.md`
- Evidence: `ecosystem/fret-chart/src/retained/canvas.rs` (`Action::SetViewWindow2DFromZoom`)
- Validation (desktop): `cargo run -p fret-demo --bin fret-demo -- chart_multi_axis_demo`
- What to validate (P0):
  - `RMB drag` in plot starts a box zoom; the gesture respects axis-band active routing rules.
  - `Alt` expands horizontally; `Shift` expands vertically (unless `Shift` is used to start the gesture).
  - If either axis is zoom-locked or fixed, the gesture does not start.

**S4 - Category axis under zoom for non-bar series** (`[~]`)

- ECharts reference: category axis + dataZoom behavior (series sampling under ordinal transforms)
- ADR(s): `docs/adr/0130-delinea-axis-scales-and-coordinate-mapping.md`, `docs/adr/0140-delinea-dataset-storage-and-indices.md`
- Evidence: `ecosystem/delinea/src/engine/stages/ordinal_index.rs` (ordinal mapping), `ecosystem/delinea/src/engine/axis.rs` (ticks)
- Validation (existing coverage):
  - `cargo run -p fret-demo --bin fret-demo -- horizontal_bars_demo` (category Y axis + bar layout + axis pointer)
- Missing validation harness (recommended):
  - Add a `category_line_demo` (line/scatter over a category axis + dataZoom) to lock ordinal mapping invariants under zoom.
- Missing vs ECharts:
  - fully stable ordinal mapping semantics for line/scatter under zoom (not just bars/axis pointer),
  - conformance tests that lock “raw index ↔ ordinal index” invariants across transforms.

**S5 - Tooltip content parity + formatting hooks** (`[~]`)

- ECharts reference: tooltip formatter + axisPointer sampling behavior (series order, missing values, snapping rules)
- ADR(s): `docs/adr/0133-delinea-interaction-and-hit-testing-contract.md`, `docs/adr/0148-delinea-tooltip-formatting-contract.md`
- Evidence: `ecosystem/delinea/src/tooltip.rs`, `ecosystem/fret-chart/src/retained/tooltip.rs`, `ecosystem/delinea/src/engine/hit_test.rs`
- Validation (desktop): `cargo run -p fret-demo --bin fret-demo -- chart_demo`
- Validation (wasm): `cargo run -p fretboard -- dev web --demo chart_demo`
- Notes:
  - This is a *chart tooltip* (axisPointer-driven, data-derived, per-frame), not a generic UI tooltip primitive.
    It intentionally lives inside `fret-chart` rather than reusing the Radix/Shadcn tooltip surface.
- What to validate (P0 baseline):
  - Tooltip rows are stable and ordered by `series_order`.
  - Missing/unsampleable series show `-` instead of panicking or reordering rows.
  - When `axisPointer.snap=true` and `trigger=Axis`, the pointer aligns to a nearest sample on the trigger axis (stable away from the series stroke).
  - `axisPointer.triggerDistance` gates the snap marker (`axisPointer.hit`) only; the crosshair and tooltip remain available for `trigger=Axis`.
- What exists in v1:
  - `ChartSpec.tooltip: Option<TooltipSpecV1>` supports templates + decimals, including per-series overrides (adapter-side).
  - Tooltip marker swatches are rendered from the series palette (UI-side).
  - Tooltip lines support a two-column `label: value` layout (UI-side; current heuristic split on `": "`).
  - Delinea: `AxisPointerSpec.pointer_type=Shadow` (ECharts: `axisPointer.type="shadow"`) highlights the active category band (`AxisPointerOutput.shadow_rect_px`).
  - Delinea: `AxisPointerSpec.label.show=true` (ECharts: `axisPointer.label.show=true`) draws an axis value label on the trigger axis band (adapter-side; default is `false`). v1 supports a string template formatter via `AxisPointerSpec.label.template` (`{value}`, `{axis_name}`).
- Missing vs ECharts:
  - ECharts formatter parity (callback-style formatting, rich text/HTML markers, per-series overrides),
  - richer tooltip layout (structural columns, rich text/HTML) and additional snapping policies.

**S6 - Legend semantics (series visibility) + UI parity** (`[~]`)

- ECharts reference: legend selection model + event semantics
- ADR(s): (engine-level visibility is part of the core model contract; UI parity is adapter work)
- Evidence: `delinea::Action::SetSeriesVisible` + marks gating in `ecosystem/delinea/src/engine/stages/marks.rs`
- Validation (desktop): `cargo run -p fret-demo --bin fret-demo -- chart_multi_axis_demo`
- What to validate (P0 baseline):
  - `LMB click` on a legend row toggles that series visibility.
  - `LMB double-click` isolates the clicked series (hides all others).
  - When a series is already isolated, `LMB double-click` restores all series visibility.
  - `Shift + LMB click` toggles a contiguous range (anchor -> clicked) to match the clicked toggle target.
  - `RMB click` inside the legend panel restores all series visibility.
  - Hidden series do not participate in axisPointer primary selection and are excluded from axis-trigger tooltip rows.
- What exists in v1:
  - A built-in legend overlay in `fret-chart` (panel + swatch + hover highlight).
  - Visibility is wired through `delinea::Action::SetSeriesVisible` (headless model is authoritative).
- Missing vs ECharts:
  - scroll/overflow handling, multi-legend layout, and selection UX parity (invert, select-all/none),
  - conformance scenarios for legend <-> tooltip/axisPointer interactions.

**S7 - VisualMap (continuous + piecewise) multi-channel baseline** (`[~]`)

- ECharts reference: `repo-ref/echarts/src/component/visualMap/VisualMapModel.ts`, `repo-ref/echarts/src/component/visualMap/visualEncoding.ts`
- ADR(s): `docs/adr/0147-delinea-visualmap-and-data-driven-styling.md`
- Evidence: `ecosystem/delinea/src/engine/stages/marks.rs` (bucketed batches), `ecosystem/fret-chart/src/retained/canvas.rs` (controller UI)
- Validation (desktop):
  - `cargo run -p fret-demo --bin fret-demo -- chart_multi_axis_demo` (scatter visualMap + multi-axis controller band)
  - `cargo run -p fret-demo --bin fret-demo -- horizontal_bars_demo` (bar visualMap)
- What to validate (P0 baseline):
  - Continuous: drag inside range pans; drag handles resizes; click outside jumps.
  - Piecewise: click toggles buckets; `Shift+Click` range toggles; `RMB`/double click resets.
  - Channels: bucketed color, per-bucket opacity ramp, scatter radius multiplier, and optional stroke width range (v1: scatter + bar).
- Missing vs ECharts:
  - multiple VisualMaps targeting the same series (v1 restriction),
  - per-item attribute pipelines (GPU instancing) for rich multi-channel mapping without bucketization.

**S8 - LOD / downsampling strategies and conformance harness** (`[~]`)

- ECharts reference: `large`, `progressive`, sampling/decimation knobs per series type
- ADR(s): `docs/adr/0132-delinea-large-data-and-progressive-rendering.md`
- Evidence: `ecosystem/delinea/src/engine/lod/*` + stage budgets
- v1 invariants (P0 baseline):
  - Line-family (line/area/band): min/max-per-pixel bucketing over the plot width, emitting `<= 4 * plot_width_px`
    points for monotonic-X inputs (preserves spikes while staying pixel-bounded).
  - Scatter: exact mode for small datasets; large mode (`visible_len > 20_000`) switches to pixel-bounded LOD.
  - LOD outputs preserve index identity: `points.len() == data_indices.len()` and indices refer to raw rows.
- Tests (headless):
  - `ecosystem/delinea/src/engine/lod/minmax_per_pixel.rs` (unit invariants for finalize ordering + bounds)
  - `ecosystem/delinea/src/engine/tests.rs` (`scatter_large_mode_is_pixel_bounded`, `line_large_mode_is_pixel_bounded`)
- Conformance doc: `docs/delinea-lod-conformance.md`
- Validation harness (native, v1):
  - `cargo run -p fret-demo --bin chart_stress_demo`
  - Env knobs:
    - `FRET_CHART_STRESS_POINTS` (default: 1_000_000)
    - `FRET_CHART_STRESS_EXIT_AFTER_FRAMES`
- Missing vs ECharts:
  - explicit policies per series kind (line vs scatter vs bar),
  - a benchmark/conformance harness that locks frame-time and visual invariants.

**S9 - Append/update semantics (`appendData`)** (`[~]`)

- ECharts reference: `appendData` and incremental updates on `DataStore`
- ADR(s): `docs/adr/0140-delinea-dataset-storage-and-indices.md`, `docs/adr/0149-delinea-appenddata-and-incremental-caches.md`
- ADR(s): `docs/adr/0140-delinea-dataset-storage-and-indices.md` (append-only rule; v1 ingestion API)
- Evidence:
  - `ecosystem/delinea/src/data/mod.rs` (`DataTable::append_row_f64`, `DataTable::append_columns_f64`)
  - `ecosystem/delinea/src/engine/stages/data_view.rs` (append-only resume for XFilter index scans)
  - `ecosystem/delinea/src/engine/stages/ordinal_index.rs` (append-only resume for ordinal inverted index scans)
- Validation (headless): `cargo nextest run -p delinea` (see `data_view_stage_invalidates_indices_on_data_revision_change`)
- Missing vs ECharts (high value):
  - append-aware incremental mark rebuild for `RowSelection::All` (avoid rebuilding the entire mark tree on stream append),
  - append-aware incremental bounds extension (monotonic X fast path),
  - streaming-focused benchmarks and CI gates for regressions.

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
- When `LinkGroup` is configured, brush selection changes emit a `LinkEvent::BrushSelectionChanged`
  event, enabling ECharts-like `brushLink` behavior (see ADR 0146).
- In the multi-axis harness, the demo runs two charts in the same link group:
  - Brushing in one chart updates the selection in the other chart.
  - Clearing brush selection in one chart clears it in the other chart.

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
- VisualMap controllers (v1):
  - Rendered when the model has at least one `VisualMapSpec`.
  - Control UI lives in a dedicated right-side band (outside the plot clip).
  - Continuous:
    - Drag inside the selected window pans the range; drag the min/max handles resizes.
    - Clicking outside the window jumps the selection and continues as a pan drag.
    - Writes `Action::SetVisualMapRange` into `ChartState.visual_map_range`.
  - Piecewise:
    - Clicking a bucket toggles its selection (inRange/outOfRange).
    - Writes `Action::SetVisualMapPieceMask` into `ChartState.visual_map_piece_mask`.

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
- `[~]` 2D brush selection (rect selection output; does not write view windows).
  - Headless state/output: `delinea::ChartState.brush_selection_2d` / `delinea::ChartOutput.brush_selection_2d`
  - Action: `delinea::Action::SetBrushSelection2D` / `delinea::Action::ClearBrushSelection` (ADR 0144)
  - Derived fast path: `delinea::ChartOutput.brush_x_row_ranges_by_series` (X-only contiguous selection; ADR 0145)
  - UI gesture (current default): `Alt + RMB drag` (ImPlot-style)
- `[x]` `minSpan/maxSpan` policies for interaction-derived view window writes (ADR 0138).
  - Evidence: `ecosystem/delinea/src/engine/mod.rs` (span clamp in interaction actions) + `ecosystem/delinea/src/engine/window.rs` (span limiter).
  - Notes: implemented as value-space `DataZoomXSpec.min_value_span/max_value_span` (no percent space).

### Performance & large data

- `[x]` Explicit progressive stepping budget (ADR 0132).
- `[x]` Progressive stepping does not rely on pointer-driven invalidation.
  - Evidence: `ecosystem/fret-chart/src/retained/canvas.rs` (requests animation frames while unfinished) +
    `crates/fret-ui/src/tree/paint.rs` (clears stale paint cache entries when caching is disabled for a node).
- `[x]` No per-frame allocations in core stages (target; enforce via tests/benchmarks over time).
- `[~]` Series-specific LOD / downsampling strategies (scatter vs line vs bar) (needs a conformance harness).
- `[~]` Append/update semantics (ECharts `appendData`) (append-only APIs exist; incremental cache extension still pending).

### Styling & theming

- `[~]` Token-driven chart styling (tracked in `docs/adr/0142-fret-chart-theme-tokens-and-style-resolution.md`).
- `[~]` VisualMap-style data-driven color mapping (ECharts `visualMap`) (scatter + bar v1 buckets).
  - Evidence: `docs/adr/0147-delinea-visualmap-and-data-driven-styling.md`, `ecosystem/delinea/src/engine/stages/marks.rs`, `ecosystem/fret-chart/src/retained/canvas.rs`.
  - Notes: v1 includes continuous + piecewise controller UI, scatter/bar bucket coloring, per-bucket opacity ramps, scatter point radius mapping, and optional stroke width ranges; per-item attribute pipelines are future work.

## Known Gaps vs ECharts (High Value)

- DataZoom Y + 2D zoom UX parity (inside + box zoom + reset behaviors) (S2).
- Category axis indexing under zoom for non-bar series (S4).
- Tooltip snapping + axisPointer sampling policies (S5).
- Legend UI parity + selection UX (S6).
- VisualMap: multiple maps per series and per-item attribute pipelines (S7).
- Series-specific LOD / downsampling policies + harness (S8).
- Append/update semantics (ECharts `appendData`) (S9).

## Recommended Next Steps (P0 -> P1)

1. P0: Legend widget + selection model UX (single/multi select, invert, reset) (S6).
2. P0: Tooltip snapping policies and axis-trigger tooltip UX (crosshair + stable rows away from points) (S5).
3. P0: VisualMap: multiple maps per series and a plan for per-item attributes/instancing (S7).
4. P1: Category axis indexing under zoom for non-bar series (lock ordinal invariants with a dedicated demo) (S4).
5. P1: LOD / downsampling policies + conformance harness (S8).
6. P1: Append/update semantics (ECharts `appendData`) (S9).
