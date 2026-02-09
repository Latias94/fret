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
  - Headless engine: `docs/adr/1128-delinea-headless-chart-engine.md`
  - Transform pipeline + X dataZoom: `docs/adr/1129-delinea-transform-pipeline-and-datazoom-semantics.md`
  - Axis scales + mapping: `docs/adr/1130-delinea-axis-scales-and-coordinate-mapping.md`
  - Marks contract: `docs/adr/1131-delinea-marks-identity-and-renderer-contract.md`
  - Large-data + progressive: `docs/adr/1132-delinea-large-data-and-progressive-rendering.md`
  - Interaction + hit testing: `docs/adr/1133-delinea-interaction-and-hit-testing-contract.md`
  - Multi-axis + layout: `docs/adr/1134-delinea-multi-axis-and-layout-contract.md`
  - Axis locks + shortcuts: `docs/adr/1135-delinea-axis-interaction-locks-and-shortcuts.md`
  - DataZoom Y + 2D semantics (v1 divergence): `docs/adr/1136-delinea-datazoom-y-and-2d-semantics.md`
  - Row selection + filtering: `docs/adr/1137-delinea-row-selection-and-filtering-contract.md`
- DataZoom composition + span policy: `docs/adr/1138-delinea-datazoom-component-composition-and-span-policy.md`
- Dataset storage + indices: `docs/adr/1140-delinea-dataset-storage-and-indices.md`
- Brush selection output: `docs/adr/1144-delinea-brush-selection-and-output-contract.md`
- Brush selection row-range fast path: `docs/adr/1145-delinea-brush-selection-to-row-selection-fast-path.md`
- Brush selection link events: `docs/adr/1146-delinea-link-events-for-brush-selection.md`
- VisualMap (data-driven styling): `docs/adr/1147-delinea-visualmap-and-data-driven-styling.md`

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

- `[x]` Stable “raw index” identity across transforms (`RowSelection` + indices) (ADR 1140).
- `[x]` Separate durable config vs ephemeral view windows (ADR 1129 / ADR 1136).
- `[x]` Large-data strategy is explicit (budgeted stepping; progressive marks) (ADR 1132).
- `[x]` Interaction contract is mark-based and allocation-aware (ADR 1133).
- `[x]` Multi-axis routing is deterministic (active axis pair; region routing) (ADR 1134).

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
- ADR(s): `docs/adr/1129-delinea-transform-pipeline-and-datazoom-semantics.md`, `docs/adr/1138-delinea-datazoom-component-composition-and-span-policy.md`
- Evidence: `ecosystem/delinea/src/engine/window.rs` (span limits), `ecosystem/delinea/src/engine/mod.rs` (interaction action routing), `ecosystem/fret-chart/src/retained/canvas.rs` (inside + slider gestures), `ecosystem/fret-chart/src/echarts/mod.rs` (option adapter window seeding via `dataZoom.startValue/endValue` -> value-window actions, `dataZoom.start/end` -> `Action::SetAxisWindowPercent`)
- Validation (desktop): `cargo run -p fret-demo --bin fret-demo -- chart_multi_axis_demo`
- Validation (wasm): `cargo run -p fretboard -- dev web --demo chart_multi_axis_demo`
- What to validate (P0):
  - Wheel on bottom X axis band zooms X only; span limits clamp zoom-in/out writes.
  - Drag on bottom X axis band pans X only.
  - X slider (bottom) supports pan drag + min/max handle drags + click-to-jump.
  - Locks (`Ctrl + LMB` toggle) gate slider + wheel writes for the locked axis.

**S2 - DataZoom Y + 2D zoom parity boundary (v1 divergence)** (`[~]`)

- ECharts reference: `repo-ref/echarts/src/component/dataZoom/*` (order-sensitive multi-dim filtering)
- ADR(s): `docs/adr/1136-delinea-datazoom-y-and-2d-semantics.md`, `docs/adr/1137-delinea-row-selection-and-filtering-contract.md`
- Evidence: `ecosystem/delinea/src/engine/mod.rs` (percent-window routing + stage order), `ecosystem/delinea/src/engine/stages/filter_processor.rs` (Y percent extent derivation + Y filtering materialization + participation snapshot), `ecosystem/delinea/src/transform/data_zoom_y.rs` (Y filter node), `ecosystem/delinea/src/transform_graph/data_view.rs` (budgeted indices materialization), `ecosystem/delinea/src/view/mod.rs` (view selection/mask policy), `ecosystem/delinea/src/transform/*` (RowSelection), `ecosystem/delinea/src/engine/tests.rs` (`percent_y_extent_is_scoped_by_x_percent_window`, `data_zoom_y_filter_mode_filter_ignores_x_window_when_x_filter_mode_empty`, `data_zoom_y_filter_mode_filter_culls_band_rows_by_interval_intersection`, `data_zoom_x_filter_mode_empty_masks_scatter_marks_without_culling_row_selection`)
- Notes: The per-series participation contract (`ParticipationState::series_contract`) is the single source of truth for selection + mask consumption in marks and sampling.
- Validation (desktop): `cargo run -p fret-demo --bin fret-demo -- chart_multi_axis_demo`
- What to validate (current v1 behavior):
  - Wheel on Y axis band zooms Y only; Y span limits (when configured) clamp interaction-derived writes.
  - 2D box zoom writes a paired window update (no sparse filtering materialization).
  - When `DataZoomYSpec.filter_mode=Filter` is enabled, non-stacked scatter and line-family series (Line/Area) may materialize a sparse `RowSelection::Indices` filtered by the effective Y window (current: guarded by a view-size cap and disabled for stacked series; intersects with active X filter predicates when needed for non-monotonic inputs).
  - When X filtering is represented by an indices-backed carrier (large non-monotonic views), and Y filtering is materialized as indices, the engine applies X indices before materializing Y indices in the same frame (order-sensitive, ECharts-style).
  - When percent windows are used (`Action::SetAxisWindowPercent`), the derived Y percent domain is computed after X has affected the selection/domain (order-sensitive, ECharts-style “X before Y” behavior within a grid; v1 cartesian subset).
  - When `DataZoomXSpec.filter_mode=Empty` is enabled, X is represented as an empty mask (not a culling predicate). This can yield a non-empty selection while emitting zero marks when X and Y windows are disjoint.
  - When `DataZoomYSpec.filter_mode=Empty` is enabled, non-stacked line-family series (Line/Area/Band) may emit segment breaks for samples outside the Y window (masking preserves base row selection; tooltip/axisPointer sampling treats masked samples as `Missing`).
  - When both X and Y `filterMode=weakFilter` are enabled (one `dataZoom` per axis), non-stacked scatter and line-family series (Line/Area/Band) may materialize a sparse `RowSelection::Indices` implementing the ECharts `weakFilter` rule: filter only when **all** relevant dimensions are out-of-window on the **same** side (below/below or above/above). This is a v1 subset (cartesian XY only, size-capped, and budgeted via `DataViewStage`; for `Band`, the Y dimension is treated as an interval).
  - When the view is above the multi-dim `weakFilter` size cap, `delinea` currently degrades to per-axis filtering (effectively `Filter` behavior), because it does not materialize the indices-backed selection needed to preserve ECharts `weakFilter` semantics.
- Missing vs ECharts (high value):
  - Y-driven filtering semantics (ordering rules beyond the current per-grid X->Y indices materialization subset),
  - full ECharts-class `weakFilter` behavior across arbitrary dimension sets and axis types (beyond cartesian XY),
  - ECharts-style `empty` masking parity for stacked series and downstream outputs (brush export, transform ordering), beyond the current cartesian v1 subset.
  - 2D zoom interactions that can materialize sparse selections when needed.

**S3 - 2D box zoom writes an atomic paired window action** (`[x]`)

- ECharts reference: `repo-ref/echarts/src/component/dataZoom/*` + brush/interaction glue (behavioral reference only; implementation differs)
- ADR(s): `docs/adr/1136-delinea-datazoom-y-and-2d-semantics.md`
- Evidence: `ecosystem/fret-chart/src/retained/canvas.rs` (`Action::SetViewWindow2DFromZoom`)
- Validation (desktop): `cargo run -p fret-demo --bin fret-demo -- chart_multi_axis_demo`
- What to validate (P0):
  - `RMB drag` in plot starts a box zoom; the gesture respects axis-band active routing rules.
  - `Alt` expands horizontally; `Shift` expands vertically (unless `Shift` is used to start the gesture).
  - If either axis is zoom-locked or fixed, the gesture does not start.

**S4 - Category axis under zoom for non-bar series** (`[~]`)

- ECharts reference: category axis + dataZoom behavior (series sampling under ordinal transforms)
- ADR(s): `docs/adr/1130-delinea-axis-scales-and-coordinate-mapping.md`, `docs/adr/1140-delinea-dataset-storage-and-indices.md`
- Evidence: `ecosystem/delinea/src/engine/stages/ordinal_index.rs` (ordinal mapping), `ecosystem/delinea/src/engine/axis.rs` (category bands), `ecosystem/delinea/src/view/mod.rs` (band-edge mapping window under zoom), `ecosystem/delinea/src/engine/tests.rs` (`axis_pointer_shadow_rect_respects_category_band_edges_under_x_window`, `category_x_filter_culls_marks_for_non_monotonic_line_and_samples_first_duplicate`, `category_x_filter_materializes_indices_for_scatter_and_respects_base_row_range`)
- Validation (existing coverage):
  - `cargo run -p fret-demo --bin fret-demo -- horizontal_bars_demo` (category Y axis + bar layout + axis pointer)
- Validation (recommended):
  - `cargo run -p fret-demo --bin fret-demo -- category_line_demo` (category X axis + line/scatter + dataZoom)
- Missing vs ECharts:
  - Category data is currently represented as `f64` ordinals (indices). ECharts-style string/any-value
    categories require a clearer adapter/engine mapping strategy and explicit constraints.
  - Duplicate-category sampling policy (first/last/nearest/aggregate) is currently a v1 implicit rule
    (biased towards first-in-selection). It needs to be specified and made consistent across more
    series kinds (Area/Band/stacked) and sampling paths.
  - Performance and consistency: the coordination between indices materialization and ordinal index
    caching can still be improved (avoid redundant index builds; reuse ordinal mapping in more
    sampling paths instead of scanning).

**S5 - Tooltip content parity + formatting hooks** (`[~]`)

- ECharts reference: tooltip formatter + axisPointer sampling behavior (series order, missing values, snapping rules)
- ADR(s): `docs/adr/1133-delinea-interaction-and-hit-testing-contract.md`, `docs/adr/1148-delinea-tooltip-formatting-contract.md`
- Evidence: `ecosystem/delinea/src/tooltip.rs`, `ecosystem/delinea/src/engine/hit_test/mod.rs`, `ecosystem/fret-chart/src/retained/tooltip.rs`, `ecosystem/fret-chart/src/declarative/tooltip_overlay.rs`
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
  - Axis-trigger sampling policies are stable and series-kind aware:
    - Line/Area/Band: linear interpolation for monotonic X inputs; nearest-sample fallback for non-monotonic inputs.
    - Scatter: nearest-sample selection on X (no interpolation).
    - Bar: category-ordinal lookup (stable index mapping), respecting stacked value when applicable.
- What exists in v1:
  - `ChartSpec.tooltip: Option<TooltipSpecV1>` supports templates + decimals, including per-series overrides (adapter-side).
    - `TooltipSpecV1.item_axis_line` controls whether item-trigger tooltips include the axis row (`hide` is the default, ECharts-aligned).
  - Tooltip marker swatches are rendered from the series palette (UI-side).
  - Tooltip lines support a two-column `label: value` layout (UI-side; shared helper `ecosystem/fret-chart/src/tooltip_layout.rs`).
  - Tooltip text is structurally typed for stable styling across adapters:
    - `TooltipTextLineKind` (`AxisHeader` / `SeriesRow` / `Body`) + `value_emphasis` for value weight, and `is_missing` for de-emphasis.
    - Evidence: `ecosystem/fret-chart/src/retained/tooltip.rs`, `ecosystem/fret-chart/src/{retained/canvas.rs,declarative/tooltip_overlay.rs}`.
  - Declarative overlay: axisPointer shadow/crosshair + snap marker + tooltip bubble is rendered by `ecosystem/fret-chart/src/declarative/tooltip_overlay.rs` (state is snapshotted during render; paint reads only the snapshot).
  - Delinea: `AxisPointerSpec.pointer_type=Shadow` (ECharts: `axisPointer.type="shadow"`) highlights the active category band (`AxisPointerOutput.shadow_rect_px`).
  - Delinea: `AxisPointerSpec.label.show=true` (ECharts: `axisPointer.label.show=true`) draws axis value labels (adapter-side; default is `false`).
    - Axis trigger: one label on the trigger axis band.
    - Item trigger: labels for both X and Y axes (best-effort), and tooltip placement avoids label rects.
    - v1 supports a string template formatter via `AxisPointerSpec.label.template` (`{value}`, `{axis_name}`).
  - Tooltip placement is confine-first and avoids axisPointer label rects (shared placement helper):
    - Evidence: `ecosystem/fret-chart/src/tooltip_layout.rs` (`place_tooltip_rect`), used by both retained and declarative tooltip overlays.
  - Axis-trigger sampling reads from the current view selection (DataZoom/filter/selection) and is allocation-aware:
    - monotonic ranges use a bounded `lower_bound` interpolation path,
    - non-monotonic ranges use nearest-scan with a hard cap (`MAX_UNSORTED_AXIS_SCAN_POINTS`) to avoid O(n) work on huge datasets,
    - for very large non-monotonic X views, the engine can build a budgeted "nearest X" index to recover near-O(log n) sampling (`NearestXIndexStage`).
      The stage supports append-only resume and prefix reuse when the request end grows.
- Missing vs ECharts:
  - Rich text / HTML tooltip content and marker layout parity (ECharts `richText` / DOM tooltip).
  - Tooltip composition parity for complex series payloads (multi-dim values, multiple value axes, custom marker shapes).
  - More complete axisPointer label policies (only-X label for item trigger, label collision/stacking policy, per-axis styling options).
  - Additional snapping policies and richer structural tooltip layouts beyond the current two-column rows.

**S6 - Legend semantics (series visibility) + UI parity** (`[x]`)

- ECharts reference: legend selection model + event semantics
- ADR(s): (engine-level visibility is part of the core model contract; UI parity is adapter work)
- Evidence: `delinea::Action::SetSeriesVisible` + marks gating in `ecosystem/delinea/src/engine/stages/marks.rs`
- Validation (desktop): `cargo run -p fret-demo --bin fret-demo -- chart_multi_axis_demo`
- What to validate (P0 baseline):
  - `LMB click` on legend selector buttons (`All` / `None` / `Invert`) updates the visibility set accordingly.
  - `LMB click` on a legend row toggles that series visibility.
  - `LMB double-click` isolates the clicked series (hides all others).
  - When a series is already isolated, `LMB double-click` restores all series visibility.
  - `Shift + LMB click` toggles a contiguous range (anchor -> clicked) to match the clicked toggle target.
  - `RMB click` inside the legend panel restores all series visibility.
  - `Ctrl+A` (when the pointer is in the legend panel): select all series.
  - `Ctrl+Shift+A` (when the pointer is in the legend panel): select none.
  - `Ctrl+I` (when the pointer is in the legend panel): invert selection.
  - Hidden series do not participate in axisPointer primary selection and are excluded from axis-trigger tooltip rows.
- What exists in v1:
  - A built-in legend overlay in `fret-chart` (panel + swatch + hover highlight) for retained and declarative charts.
  - Visibility is wired through `delinea::Action::SetSeriesVisible` (headless model is authoritative).
  - Basic overflow handling: the legend panel height is clamped to the plot height, and the wheel scrolls the legend.
  - Selector affordance: `All` / `None` / `Invert` selector buttons at the top of the legend panel.
  - Shared legend selection logic is factored into `ecosystem/fret-chart/src/legend_logic.rs` to keep retained and declarative behavior aligned.
- Missing vs ECharts:
  - multi-legend layout and full selector schema parity (ECharts `legend.selector` options + styling),
  - conformance scenarios for legend <-> tooltip/axisPointer interactions,
  - keyboard shortcut parity for the declarative legend overlay (retained canvas supports `Ctrl+A` / `Ctrl+I` / etc when the pointer is in the legend).

**S7 - VisualMap (continuous + piecewise) multi-channel baseline** (`[~]`)

- ECharts reference: `repo-ref/echarts/src/component/visualMap/VisualMapModel.ts`, `repo-ref/echarts/src/component/visualMap/visualEncoding.ts`
- ADR(s): `docs/adr/1147-delinea-visualmap-and-data-driven-styling.md`
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
- ADR(s): `docs/adr/1132-delinea-large-data-and-progressive-rendering.md`
- Evidence: `ecosystem/delinea/src/spec/mod.rs` (`SeriesSpec.lod`), `ecosystem/delinea/src/engine/lod/*` + stage budgets
- v1 invariants (P0 baseline):
  - Line-family (line/area/band): min/max-per-pixel bucketing over the plot width, emitting `<= 4 * plot_width_px`
    points for monotonic-X inputs (preserves spikes while staying pixel-bounded).
  - Scatter: exact mode for small datasets; large mode (default: `visible_len > 20_000`) switches to pixel-bounded LOD.
    The threshold and progressive cap are configurable via `SeriesSpec.lod` (v1 subset).
  - Bar: exact mode (one rect per visible row); no pixel-bounded LOD yet (performance is budgeted but output size is not).
  - LOD outputs preserve index identity: `points.len() == data_indices.len()` and indices refer to raw rows.
- Tests (headless):
  - `ecosystem/delinea/src/engine/lod/minmax_per_pixel.rs` (unit invariants for finalize ordering + bounds)
  - `ecosystem/delinea/src/engine/tests.rs` (`scatter_large_mode_is_pixel_bounded`, `scatter_large_threshold_can_force_large_mode`, `scatter_progressive_can_force_multiple_steps`, `line_large_mode_is_pixel_bounded`, `lod_scatter_large_mode_is_budget_invariant`, `lod_line_large_mode_is_budget_invariant`, `lod_bar_mode_is_budget_invariant`)
- Conformance doc: `docs/delinea-lod-conformance.md`
- Validation harness (native, v1):
  - `cargo run -p fret-demo --bin chart_stress_demo`
  - Env knobs:
    - `FRET_CHART_STRESS_POINTS` (default: 1_000_000)
    - `FRET_CHART_STRESS_EXIT_AFTER_FRAMES`
- Missing vs ECharts:
  - Full option-schema parity is not a goal; however, per-series LOD knobs are still partial:
    `SeriesSpec.lod` exists (v1 subset, scatter wired), but line/bar/stacked scenarios need more coverage.
  - A benchmark/conformance harness that can gate frame-time regressions on CI (richer invariants than
    the current manual stress demo).

**S9 - Append/update semantics (`appendData`)** (`[~]`)

- ECharts reference: `appendData` and incremental updates on `DataStore`
- ADR(s): `docs/adr/1140-delinea-dataset-storage-and-indices.md`, `docs/adr/1149-delinea-appenddata-and-incremental-caches.md`
- ADR(s): `docs/adr/1140-delinea-dataset-storage-and-indices.md` (append-only rule; v1 ingestion API)
- Evidence:
  - `ecosystem/delinea/src/data/mod.rs` (`DataTable::append_row_f64`, `DataTable::append_columns_f64`)
  - `ecosystem/delinea/src/transform_graph/data_view.rs` (append-only resume for XFilter index scans)
  - `ecosystem/delinea/src/engine/stages/ordinal_index.rs` (append-only resume for ordinal inverted index scans)
  - `ecosystem/delinea/src/engine/stages/nearest_x_index.rs` (append-only resume + prefix reuse for nearest-X index scans)
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
  event, enabling ECharts-like `brushLink` behavior (see ADR 1146).
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
- `ChartOutput.hover` is gated by `AxisPointerSpec.trigger_distance_px` and matches `AxisPointerOutput.hit`:
  - `trigger=Item`: `hover` is `None` when the pointer is too far from a mark (same as `axis_pointer=None`).
  - `trigger=Axis`: `axis_pointer` can remain active while `hover`/marker hit is `None`.
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
- AxisPointer/tooltip sampling respects X filtering even for non-monotonic X (never samples out-of-window points).
  - Evidence: `ecosystem/delinea/src/engine/mod.rs`, `ecosystem/delinea/src/engine/tests.rs` (`axis_pointer_tooltip_respects_y_empty_mask_under_x_weakfilter_for_scatter_series`)
- AxisPointer/tooltip sampling respects `FilterMode::Empty` masks even when the underlying row selection is non-empty.
  - Evidence: `ecosystem/delinea/src/engine/tests.rs` (`axis_pointer_tooltip_respects_y_empty_mask_for_scatter_series`, `axis_pointer_tooltip_respects_y_empty_mask_for_band_series`, `axis_pointer_tooltip_respects_y_empty_mask_under_x_weakfilter_for_scatter_series`, `axis_pointer_tooltip_respects_x_empty_mask_when_marks_are_empty_but_selection_is_not`, `axis_pointer_tooltip_respects_x_empty_mask_under_y_filtered_selection_for_line_series`, `axis_pointer_tooltip_respects_x_empty_mask_under_y_filtered_selection_for_band_series`, `axis_pointer_item_trigger_returns_none_when_marks_are_empty_under_x_empty_mask`, `axis_pointer_item_trigger_returns_none_when_line_marks_are_empty_under_x_empty_mask`, `axis_pointer_item_trigger_returns_none_when_band_marks_are_empty_under_x_empty_mask`, `axis_pointer_item_trigger_is_suppressed_for_y_empty_masked_line_samples`, `axis_pointer_item_trigger_does_not_hit_clamped_y_empty_gap_for_line_series`, `scatter_large_mode_does_not_hit_y_empty_masked_outlier`, `bar_item_trigger_does_not_hit_y_empty_masked_outlier`, `axis_pointer_shadow_rect_is_emitted_for_category_axis_when_bar_is_y_empty_masked`, `data_zoom_x_filter_mode_empty_masks_bar_marks_without_culling_row_selection`, `axis_pointer_tooltip_respects_x_empty_mask_for_bar_when_marks_are_empty_but_selection_is_not`, `axis_pointer_item_trigger_returns_none_for_bar_under_x_empty_mask_when_marks_are_empty`, `axis_pointer_item_trigger_returns_none_for_stacked_bar_under_x_empty_mask_when_marks_are_empty`, `axis_pointer_item_trigger_returns_none_for_horizontal_bar_under_x_empty_mask_when_marks_are_empty`, `axis_pointer_axis_trigger_emits_shadow_and_missing_tooltip_for_stacked_bar_under_x_empty_mask`, `axis_pointer_axis_trigger_emits_shadow_and_missing_tooltip_for_horizontal_bar_under_x_empty_mask`, `data_zoom_x_filter_mode_empty_keeps_axis_windows_stable_when_line_marks_are_empty`, `data_zoom_x_filter_mode_empty_keeps_axis_windows_stable_when_band_marks_are_empty`, `data_zoom_x_filter_mode_empty_keeps_axis_windows_stable_when_scatter_lod_marks_are_empty`)
- Multi-dimensional filtering is intentionally a v1 subset:
  - the engine owns an explicit per-grid step plan (`XYWeakFilter -> XRange -> XIndices -> YPercent -> YIndices`),
  - indices materialization is budgeted and size-capped,
  - and this is not yet full ECharts parity (see ADR 1150 + `ecosystem/delinea/src/engine/stages/filter_processor.rs`).
- Y dataZoom is mapping-first in v1: it always has a window/mapping carrier, but it may materialize
  sparse indices selections under size caps when `DataZoomYSpec.filter_mode` opts into filtering
  semantics (ADR 1136 amendments; ADR 1150).
- 2D zoom is expressed as a paired window write (`Action::SetViewWindow2DFromZoom`) without introducing
  sparse selections (ADR 1136).

## Feature Checklist (ECharts-Class Cartesian Charts)

### Data model & transforms

- `[x]` Dataset + field indirection (`DatasetSpec` + `FieldSpec`) (ADR 1140).
- `[x]` `encode`-style mapping (series `x/y/y2` fields) (ADR 1128).
- `[x]` Row range gating (`SetDatasetRowRange`) for external virtualization (ADR 1137).
- `[x]` X filtering via `FilterMode` (`Filter` / `None` / `WeakFilter` / `Empty`) (ADR 1129 / ADR 1150).
- `[~]` `FilterMode::{WeakFilter,Empty}` surface + v1 subset semantics (ADR 1150); multi-dimensional parity still pending.

### Axes, scales, and grids

- `[x]` X/Y axes with explicit kind + placement (`AxisKind`, `AxisPosition`) (ADR 1130).
- `[x]` Value scales + mapping windows + axis ranges (ADR 1130).
- `[~]` Category axis with stable ordinal index mapping under zoom (works for bar/axis pointer; needs DataZoom Y workstream).
- `[x]` Time axis tick strategy aligned with ECharts defaults (ADR 1139; UTC-only labels in v1).

### Series types (cartesian)

- `[x]` Line
- `[x]` Area
- `[x]` Band (filled range between `y` and `y2`)
- `[x]` Bar (vertical + horizontal)
- `[x]` Scatter
- `[~]` Candlestick / OHLC (engine-level support TBD; `fret-plot` has a demo but is a separate stack)

### Components: tooltip / axisPointer / legend / dataZoom

- `[x]` Axis pointer (ECharts-like `trigger=item/axis`) (ADR 1133).
- `[~]` Tooltip content parity (series ordering, formatting hooks, value snapping) (overlay UX implemented; formatter parity still pending).
- `[x]` Legend semantics (series visibility) + baseline UI parity (selector + isolate + range toggle).
- `[x]` X dataZoom inside (wheel/drag zoom/pan) (ADR 1129).
- Evidence: `ecosystem/fret-chart/src/retained/canvas.rs` (axis-band pan, plot modifiers, and window writes).
- Demo: `apps/fret-examples/src/chart_multi_axis_demo.rs` (multi-axis interaction conformance harness; desktop + wasm).
- `[x]` dataZoom sliders (UI-only) in `fret-chart`.
  - X: bottom X axis only.
  - Y: active Y axis (left/right) based on axis-band routing.
- `[x]` Y inside zoom/pan (wheel on Y axis band; drag pan constrained via axis band or plot modifiers) (ADR 1136).
- `[x]` 2D box zoom that writes `SetViewWindow2DFromZoom` for the active axis pair (ADR 1136).
  - Evidence: `ecosystem/fret-chart/src/retained/canvas.rs` (box zoom drag -> `Action::SetViewWindow2DFromZoom`).
- `[~]` 2D brush selection (rect selection output; does not write view windows).
  - Headless state/output: `delinea::ChartState.brush_selection_2d` / `delinea::ChartOutput.brush_selection_2d`
  - Action: `delinea::Action::SetBrushSelection2D` / `delinea::Action::ClearBrushSelection` (ADR 1144)
  - Derived fast path: `delinea::ChartOutput.brush_x_row_ranges_by_series` (X-only contiguous selection; ADR 1145)
  - UI gesture (current default): `Alt + RMB drag` (ImPlot-style)
- `[x]` `minSpan/maxSpan` policies for interaction-derived view window writes (ADR 1138).
  - Evidence: `ecosystem/delinea/src/engine/mod.rs` (span clamp in interaction actions) + `ecosystem/delinea/src/engine/window.rs` (span limiter).
  - Notes: implemented as value-space `DataZoomXSpec.min_value_span/max_value_span` (no percent space).

### Performance & large data

- `[x]` Explicit progressive stepping budget (ADR 1132).
- `[x]` Progressive stepping does not rely on pointer-driven invalidation.
  - Evidence: `ecosystem/fret-chart/src/retained/canvas.rs` (requests animation frames while unfinished) +
    `crates/fret-ui/src/tree/paint.rs` (clears stale paint cache entries when caching is disabled for a node).
- `[x]` No per-frame allocations in core stages (target; enforce via tests/benchmarks over time).
- `[~]` Series-specific LOD / downsampling strategies (scatter vs line vs bar) (needs a conformance harness).
- `[~]` Append/update semantics (ECharts `appendData`) (append-only APIs exist; incremental cache extension still pending).

### Styling & theming

- `[~]` Token-driven chart styling (tracked in `docs/adr/0142-fret-chart-theme-tokens-and-style-resolution.md`).
- `[~]` VisualMap-style data-driven color mapping (ECharts `visualMap`) (scatter + bar v1 buckets).
  - Evidence (engine): `docs/adr/1147-delinea-visualmap-and-data-driven-styling.md`, `ecosystem/delinea/src/engine/stages/marks.rs`, `ecosystem/delinea/src/spec/mod.rs` (`VisualMapSpec`).
  - Evidence (translator): `ecosystem/fret-chart/src/echarts/mod.rs` (`translate_visual_maps_v1`) + `ecosystem/fret-chart/tests/echarts_headless_goldens.rs` (golden: `goldens/echarts-headless/v1/visualmap-scatter-opacity-and-size.json`).
  - Notes: v1 includes continuous + piecewise controller UI, scatter/bar bucket coloring, per-bucket opacity ramps, scatter point radius mapping, and optional stroke width ranges; per-item attribute pipelines are future work.
  - Notes (ECharts adapter v1): supports `visualMap.type`, `seriesIndex` (or dataset-wide when single dataset), `dimension` (index/name), `min/max` (or inferred from data), `range`, `inRange.opacity`, `outOfRange.opacity`, and `inRange.symbolSize` (mapped to `point_radius_mul_range` using `symbolSize / 10.0` as a heuristic).

## Known Gaps vs ECharts (High Value)

- Order-sensitive multi-dimensional filtering semantics beyond the current v1 subset (ECharts `dataZoomProcessor` ordering, per-axis composition) (S2).
- General transform graph with cached node outputs + derived columns (ECharts-class dataset transforms).
- `[~]` Multi-grid layout (multiple independent grids in one chart).
  - Notes: v1 has `GridSpec` + `AxisSpec.grid` in the engine model, and the ECharts translator binds `gridIndex`.

    - Current (v1): the retained UI adapter hosts a single `ChartEngine` instance and supplies per-grid plot
      viewports, allowing the engine to emit deterministic per-grid marks and routing surfaces.
    - Remaining gap (high value): consolidate the UI adapter around **global** controllers (one legend +
      one tooltip/axisPointer overlay for the multi-grid surface) instead of per-grid duplication (see
      workstream notes, section “Global controllers (B)”).
  - Evidence:
    - `ecosystem/fret-chart/src/echarts/mod.rs` (translates `gridIndex`)
    - `ecosystem/fret-chart/src/retained/multi_grid.rs` (retained multi-grid host)
    - `ecosystem/fret-chart/src/retained/canvas.rs` (per-grid plot viewport patching via `grid_override`)
    - `apps/fret-examples/src/echarts_multi_grid_demo.rs` (demo)
    - Workstream tracker: `docs/workstreams/delinea-engine-contract-closure-v1.md` (M1 contract + acceptance gates)
- Category axis indexing under zoom for non-bar series (S4).
- VisualMap: multiple maps per series and per-item attribute pipelines (S7).
- Series-specific LOD / downsampling policies + conformance harness (S8).
- Append/update semantics (ECharts `appendData`) (S9).
- Tooltip rich text / HTML parity (S5) (deferred; current work focuses on infra stability).

## Recommended Next Steps (P0 -> P1)

Workstream tracker for engine-level contract closure (multi-grid + transform lineage + incremental updates):
`docs/workstreams/delinea-engine-contract-closure-v1.md`.

1. P0: Expand the existing “filter processor” stage (ECharts `dataZoomProcessor` analogue) to cover the remaining order-sensitive multi-dim composition gaps and to emit a unified participation contract (selection + masks) (S2).
2. P0: Add a general transform graph (cached nodes + derived columns) and migrate DataZoom/filtering to it incrementally (ECharts-class dataset transforms).
3. P0: Multi-grid layout + deterministic routing rules (UI adapter + engine layout) (ADR 1134 follow-ups).
   - Note: v1 now uses per-grid plot viewports in a single chart instance; the remaining target is
     global controllers (shared legend + tooltip/axisPointer overlay) and cross-grid linking semantics.
4. P0/P1: Incremental dataset updates + stable partial recompute (append/update + cache invalidation boundaries) (S9 / ADR 1149).
5. P1: VisualMap per-item attribute pipelines (beyond bucketization) and multi-map targeting semantics (S7).
6. P1: Tooltip rich text / HTML parity and richer formatter surfaces (S5 / ADR 1148).

## ECharts Replica P0 Backlog (Option -> Engine Baseline)

This section is a **workable P0 execution plan** for the `fret-chart` ECharts translator and the
headless engine contracts. It is intentionally narrower than the full ECharts option schema: the
goal is to establish a stable “Option -> Engine” spine that scales to more series and components.

### P0-0: Translator test harness (golden-ish, but allocation-free)

- Goal: a repeatable way to validate that a given ECharts JSON option produces stable `delinea`
  outputs (marks + axis windows), without requiring GPU rendering.
- Evidence target:
  - `ecosystem/fret-chart/src/echarts/mod.rs` (translator entrypoint + unit tests)
  - `ecosystem/fret-chart/tests/echarts_headless_goldens.rs` (headless Option -> Engine golden harness)
  - `goldens/echarts-headless/v1/*.json` (checked-in headless output snapshots)
  - `ecosystem/delinea/src/engine/tests.rs` (engine-level invariants)
- Suggested tests:
  - “can build engine and run to completion” (already exists for minimal cases + goldens)
  - “option change only affects expected revision family” (future: spec vs visual vs data)
  - To update goldens:
    - `$env:FRET_UPDATE_GOLDENS='1'; cargo test -p fret-chart -F echarts --test echarts_headless_goldens`

### P0-1: `dataset + dimensions + encode` baseline (cartesian2d only)

- Goal: support the ECharts “real” data path: `dataset.source` + `series.encode` mapping, so that
  new series types do not require ad-hoc `series.data` parsing.
- Scope:
  - `dataset`: one or many datasets (at least `dataset[0]`).
  - `source`: numeric table (array-of-arrays); string categories are P1.
  - `series.datasetIndex` + `series.encode.{x,y}` (index or name).
- Engine mapping:
  - `dataset -> delinea::DatasetSpec + DataTable`
  - `encode -> delinea::SeriesEncode` (`FieldId` bindings)
- Code anchors:
  - `ecosystem/fret-chart/src/echarts/mod.rs` (dataset parsing + encode mapping)
  - `ecosystem/delinea/src/spec/mod.rs` (`DatasetSpec`, `FieldSpec`, `SeriesEncode`)
- Missing vs ECharts:
  - `[~]` `dataset.transform` (ECharts dataset transforms): v1 supports a small eager translator subset:
    - `filter` (numeric dimension + `gte/gt/lte/lt/eq/ne`)
    - `sort` (numeric dimension + `order=asc/desc`)
    - `fromDatasetIndex` chaining for derived datasets
    - Evidence: `ecosystem/fret-chart/src/echarts/mod.rs`, `ecosystem/fret-chart/tests/echarts_headless_goldens.rs`, `goldens/echarts-headless/v1/dataset-transform-*.json`
    - Known gap: raw-index identity across dataset transforms is not yet modeled as an ECharts-class `DataStore` graph; derived datasets currently re-index rows (needs an engine-level transform graph contract).
  - `source` object rows, `sourceHeader`, and type inference (P1)

### P0-2: Multi-axis binding via indices (`xAxisIndex` / `yAxisIndex`)

- Goal: support multiple axes and deterministic binding for series (still cartesian2d).
- Scope:
  - `xAxis` / `yAxis` as arrays.
  - `series.xAxisIndex` / `series.yAxisIndex` (default 0).
- Engine mapping:
  - allocate `AxisId` per ECharts axis index; bind `SeriesSpec.{x_axis,y_axis}` accordingly.
- Code anchors:
  - `ecosystem/fret-chart/src/echarts/mod.rs` (axis array parsing + index -> `AxisId` mapping)
  - `ecosystem/delinea/src/engine/model/mod.rs` (reference validation and routing)

### P0-3: LOD/progressive knobs baseline (per-series surface)

- Goal: ECharts-inspired knobs should flow through Option -> Spec -> Model -> Marks consistently.
- Scope:
  - `series.large`, `series.largeThreshold`, `series.progressive`, `series.progressiveThreshold`
    (subset; not option-schema parity).
- Evidence:
  - Translator: `ecosystem/fret-chart/src/echarts/mod.rs`
  - Engine: `ecosystem/delinea/src/spec/mod.rs` (`SeriesSpec.lod`), `ecosystem/delinea/src/engine/stages/marks.rs`
  - Conformance: `docs/delinea-lod-conformance.md`

### P0-4: Transform graph (ECharts-class, but minimal nodes)

- Goal: stop baking filter/dataZoom semantics into bespoke code paths; move to a cached transform graph.
- Status (current):
  - `TransformGraph` exists as a minimal “derived output cache” surface, currently hosting:
    - X axis data extents caching (used by percent->value mapping).
    - Y percent extents scoped by X selection/filter (order-sensitive “X before Y” semantics per grid).
    - Incremental indices view caching (`DataViewStage`) is owned by `TransformGraph` (implemented in `transform_graph/data_view.rs`).
    - Filter plan scaffolding and the latest filter plan output snapshot are owned by `TransformGraph` (see `transform_graph/filter_plan.rs`, `transform_graph/filter_plan_output.rs`).
    - Y indices materialization node (with caching for no-op cases) is owned by `TransformGraph` (see `transform_graph/y_indices.rs`).
    - X range slicing node (monotonic fast path; cached) is owned by `TransformGraph` (see `transform_graph/x_range.rs`).
    - Participation contract (`selection` + `empty_mask` + `data_revision`) is now derived from the transform graph filter plan output snapshot (see `engine/stages/filter_processor.rs` -> `ParticipationState::rebuild_from_plan_output`).
- Minimum viable nodes:
  - selection/slice node (monotonic fast path + non-monotonic index selection)
  - derived columns (computed fields)
  - (optional) sort/index helpers for nearest-X sampling
- Code anchors:
  - `ecosystem/delinea/src/transform/*`
  - `ecosystem/delinea/src/transform_graph/data_view.rs` (current incremental indices builder)
  - `ecosystem/delinea/src/transform_graph/mod.rs` (cached extents + Y percent extents caching)
