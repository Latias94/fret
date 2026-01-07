# Axis Range and View Window Policy (v1)

This note defines the minimum, renderer-agnostic semantics needed to support
axis locking / zoom locking in `delinea` without coupling to any specific UI.

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
   - `ChartState.data_window_x[axis]` is ignored for that axis.
2. Otherwise, if `ChartState.data_window_x[axis]` is present, it becomes the effective X window.
3. Otherwise, the engine uses the full dataset range.

For Y:

1. If the Y axis range is `Fixed`, the engine uses it as the effective Y window.
   - `ChartState.data_window_y[axis]` is ignored for that axis.
2. Otherwise, if `ChartState.data_window_y[axis]` is present, it becomes the effective Y window.
3. Otherwise, the engine derives Y bounds from the dataset (restricted by the effective X window).

Partial locks (`LockMin`/`LockMax`) are applied as constraints after selecting the effective window.

In v1 the LOD stage clamps Y values to the effective Y window to avoid out-of-range
values dominating min/max selection.

## Patch/merge semantics

- `ChartPatch.axes: AxisOp::Upsert(AxisPatch { range: Some(...) })` updates an axis range.
- An axis range update is a layout/marks invalidation (not a structural change).
  It bumps `ModelRevisions.layout` (and therefore `ModelRevisions.marks`).

## Window math helpers (headless)

The headless layer exposes helpers to compute pan/zoom in data space without binding to UI:

- Pan: `DataWindow::pan_by_px(delta_px, viewport_span_px)`
- Zoom: `DataWindow::zoom_by_px(center_px, log2_scale, viewport_span_px)`

These helpers are intended to be used by UI adapters to produce
`SetDataWindowX/Y` or `SetViewWindow2D` actions.

## UI integration (recommended, not required)

The engine only defines semantics. A UI adapter (e.g. `fret-chart`) may map:

- Gestures:
  - Wheel: zoom X (update `data_window_x[axis]`).
  - Drag: pan X (update `data_window_x[axis]`).
- Box zoom:
  - Drag a rectangle: compute data windows and apply `SetViewWindow2D` for the target X/Y axes.
- Shortcuts:
  - `L`: toggle X axis lock (set `AxisRange::Fixed` to current visible range, or back to `Auto`).
  - `R`: reset view (clear `data_window_x[axis]`, set axis ranges to `Auto`).

The exact mapping is intentionally left to the UI layer.

## Future work

 - Per-axis windows (multiple X/Y axes).
- 2D box zoom (paired X/Y window updates).
- Non-linear scales (log/time) and tick generation.
- 3D: replace `DataWindowX` with generalized camera/view transforms.
