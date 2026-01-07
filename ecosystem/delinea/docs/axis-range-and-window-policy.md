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
- `Fixed { min, max }`: engine treats the axis as locked and uses this range.

In v1, `Fixed` implies the axis is non-interactive in the headless engine:
view window updates (e.g. zoom/pan) are ignored for that axis.

### `DataWindowX`

`DataWindowX` is an ephemeral view window used to represent interactive zoom/pan
in data space. It is stored in `ChartState` (not in `ChartModel`) because it is
considered view-state rather than a durable chart option.

In v1 it is stored per X axis (`AxisId -> DataWindowX`) to keep the model ready
for multiple axes.

## Precedence rules (v1)

When producing marks (bounds + LOD + projection):

1. If the X axis range is `Fixed`, the engine uses it as the effective X window.
   - Bounds scanning is restricted to the fixed X range.
   - `ChartState.data_window_x[axis]` is ignored for that axis.
2. Otherwise, if `ChartState.data_window_x[axis]` is present, it becomes the effective X window.
3. Otherwise, the engine uses the full dataset range.

Y axis `Fixed` range is applied as a clamp on the computed bounds.

## Patch/merge semantics

- `ChartPatch.axes: AxisOp::Upsert(AxisPatch { range: Some(...) })` updates an axis range.
- An axis range update is a layout/marks invalidation (not a structural change).
  It bumps `ModelRevisions.layout` (and therefore `ModelRevisions.marks`).

## UI integration (recommended, not required)

The engine only defines semantics. A UI adapter (e.g. `fret-chart`) may map:

- Gestures:
  - Wheel: zoom X (update `data_window_x[axis]`).
  - Drag: pan X (update `data_window_x[axis]`).
- Shortcuts:
  - `L`: toggle X axis lock (set `AxisRange::Fixed` to current visible range, or back to `Auto`).
  - `R`: reset view (clear `data_window_x[axis]`, set axis ranges to `Auto`).

The exact mapping is intentionally left to the UI layer.

## Future work

- `LockMin` / `LockMax` (ImPlot-style partial locks).
- Per-axis windows (multiple X/Y axes).
- Y view windows (`DataWindowY`) and 2D box zoom.
- Non-linear scales (log/time) and tick generation.
- 3D: replace `DataWindowX` with generalized camera/view transforms.
