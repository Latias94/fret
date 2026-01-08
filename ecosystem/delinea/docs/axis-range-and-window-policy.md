# Axis Range and View Window Policy (v1)

This note defines the minimum, renderer-agnostic semantics needed to support:

- durable axis constraints (`AxisRange`),
- interactive view windows (`DataWindowX` / `DataWindowY`),
- interaction gating (pan/zoom locks) via `AxisInteractionLocks`,

in `delinea`, without coupling to any specific UI.

## Goals

- Make axis range behavior explicit and deterministic in the headless engine.
- Keep the model stable for future 2D/3D expansion and "chart" features.
- Support large datasets by keeping bounds/LOD incremental and window-aware.

## Concepts

### `AxisRange`

`AxisRange` lives in the chart specification/model and describes the visible
data-space range of an axis:

- `Auto`: engine derives a suitable range from the dataset (and optional view window).
- `LockMin { min }` / `LockMax { max }`: one bound is fixed, the other bound can still follow view windows and/or data.
- `Fixed { min, max }`: both bounds are fixed.

In v1, `Fixed` fully overrides view windows for that axis. Partial locks override
only the locked bound.

### `AxisInteractionLocks` (pan/zoom locks)

`AxisInteractionLocks` lives in `ChartState` (view-state), keyed per axis:

- `pan_locked`: ignore pan actions for that axis.
- `zoom_locked`: ignore zoom actions for that axis.

This is intentionally separate from `AxisRange`:

- `AxisRange` is a data-space constraint and affects the **effective window** used for layout/LOD.
- `AxisInteractionLocks` is an interaction policy and only affects whether certain actions are applied.

### `DataWindowX` / `DataWindowY`

`DataWindowX` / `DataWindowY` are ephemeral 1D view windows used to represent
interactive zoom/pan in data space. They are stored in `ChartState` (not in
`ChartModel`) because they are considered view-state rather than durable chart
options.

The underlying data structure is `DataWindow { min, max }`.

In v1 they are stored per axis (`AxisId -> DataWindowX` and `AxisId -> DataWindowY`)
to keep the model ready for multiple axes.

## Precedence rules (v1)

When producing marks (bounds + LOD + projection):

1. If the X axis range is `Fixed`, the engine uses it as the effective X window.
   - Bounds scanning is restricted to the fixed X range.
   - `ChartState.data_zoom_x[axis].window` is ignored for that axis.
2. Otherwise, if `ChartState.data_zoom_x[axis].window` is present, it becomes the effective X window.
3. Otherwise, the engine uses the full dataset range.

For Y:

1. If the Y axis range is `Fixed`, the engine uses it as the effective Y window.
   - `ChartState.data_window_y[axis]` is ignored for that axis.
2. Otherwise, if `ChartState.data_window_y[axis]` is present, it becomes the effective Y window.
3. Otherwise, the engine derives Y bounds from the dataset (restricted by the effective X window).

Partial locks (`LockMin`/`LockMax`) are applied as constraints after selecting the effective window.

In v1 the LOD stage clamps Y values to the effective Y window to avoid out-of-range
values dominating min/max selection.

## View slicing (P0)

The engine also derives `SeriesView.row_range` to avoid processing rows that cannot contribute
to the visible output (important for large datasets).

Row slicing uses the same precedence rules as marks:

- If the X axis range is `Fixed`, the fixed window is used for slicing and `ChartState.data_zoom_x[axis].window` is ignored.
- Otherwise, `ChartState.data_zoom_x[axis].window` (when present) is used for slicing.

## Patch/merge semantics

- `ChartPatch.axes: AxisOp::Upsert(AxisPatch { range: Some(...) })` updates an axis range.
- An axis range update is a layout/marks invalidation (not a structural change).
  It bumps `ModelRevisions.layout` (and therefore `ModelRevisions.marks`).

## Window math helpers (headless)

The headless layer exposes helpers to compute pan/zoom in data space without binding to UI:

- Pan: `DataWindow::pan_by_px(delta_px, viewport_span_px)`
- Zoom: `DataWindow::zoom_by_px(center_px, log2_scale, viewport_span_px)`

These helpers are intended to be used by UI adapters to produce
`SetDataWindowX/Y`, `SetViewWindow2D`, or higher-level interaction actions (see below).

## Interaction actions (headless, recommended)

To keep input semantics consistent across UIs, the headless engine defines action meanings.

Pan/zoom actions:

- `Action::PanDataWindowXFromBase` / `Action::PanDataWindowYFromBase`
- `Action::ZoomDataWindowXFromBase` / `Action::ZoomDataWindowYFromBase`
- `Action::SetDataWindowXFromZoom` / `Action::SetDataWindowYFromZoom` (box zoom)

Guard rules:

1. If `AxisRange` is `Fixed`, pan/zoom actions are no-ops for that axis.
2. If `AxisInteractionLocks.pan_locked` is true, pan actions are no-ops.
3. If `AxisInteractionLocks.zoom_locked` is true, zoom actions are no-ops.
4. Otherwise, the resulting window is clamped via `AxisRange::LockMin/LockMax` constraints.

## UI integration (recommended, not required)

The engine only defines semantics. A UI adapter (e.g. `fret-chart`) may map:

- Gestures:
  - Wheel: zoom X (update `data_zoom_x[axis].window`).
  - Drag: pan X (update `data_zoom_x[axis].window`).
- Box zoom:
  - Drag a rectangle: compute data windows and apply `SetViewWindow2D` for the target X/Y axes.
- Shortcuts (suggested defaults, UI policy):
  - `L`: toggle pan+zoom lock for the hovered axis region (X/Y/plot = both axes).
  - `Shift+L`: toggle pan lock only.
  - `Ctrl+L`: toggle zoom lock only.
  - `R`: reset view (clear `data_zoom_x[axis].window` / `data_window_y[axis]`, keep axis ranges at `Auto`).

The exact mapping is intentionally left to the UI layer.

## Future work

 - Per-axis windows (multiple X/Y axes).
- 2D box zoom (paired X/Y window updates).
- Non-linear scales (log/time) and tick generation.
- 3D: replace `DataWindowX` with generalized camera/view transforms.
