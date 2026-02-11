# Axis Interaction Alignment (ImPlot-style) — `fret-chart` + `delinea`

This document describes the current P0 interaction policy for `ChartCanvas` (the `fret-chart` retained widget)
and how it maps to `delinea`’s headless semantics.

The goal is to provide an ImPlot-like “muscle memory” baseline without committing to ImPlot API compatibility.

For ECharts alignment and durable contracts, prefer the ADR series under `docs/adr/1128-1147-*` and
`docs/delinea-echarts-alignment.md`.

## Coordinate conventions

- Screen space: Fret logical pixels, origin at the top-left of the widget bounds.
- Plot space: `ChartCanvas` reserves axis bands and padding; series rendering and interactions are computed inside the plot rect.
- Data space: `delinea::DataWindow` is **increasing to the right** for X and **increasing upward** for Y.
- Mapping:
  - `x_data = x_window.min + t * (x_window.max - x_window.min)`, where `t = x_px / width_px`.
  - `y_data = y_window.min + t * (y_window.max - y_window.min)`, where `t = y_px_from_bottom / height_px`.

## Default gestures (P0)

The defaults are intentionally aligned with ImPlot:

- Pan: `LMB drag`
- Box zoom: `RMB drag`
- Box zoom (alt): `Shift + LMB drag` (accessibility alternative)
- Wheel zoom: `Mouse wheel`
  - Hold `Shift`: zoom X only
  - Hold `Ctrl`: zoom Y only
  - Wheel on the X axis band: zoom X only
  - Wheel on the Y axis band: zoom Y only

- Reset view: `LMB double-click`
  - On plot area: resets both axes to auto (clears `DataWindow` overrides)
  - On X/Y axis band (temporary heuristic): resets the corresponding axis only

- Fit view to data: `F` (focused canvas)
- Reset view to auto: `R` (focused canvas)
- Clear brush selection: `A` (focused canvas)
- Toggle X filter mode: `M` (focused canvas)
  - `Filter` (default): X window filters bounds/LOD and slices rows (best performance, auto-scales Y to visible X).
  - `None`: X window does not filter bounds/LOD and does not slice rows (keeps global Y scale, more expensive).
  - When active, the plot shows a small in-canvas indicator: `Y bounds: global (M)`.

Box zoom selection modifiers (ImPlot style):

- Hold `Alt`: expand selection horizontally to the plot edges (zoom Y only)
- Hold `Shift`: expand selection vertically to the plot edges (zoom X only)

Note: if a modifier is required to start the drag gesture (e.g. `Shift + LMB drag`), it is treated as part of the
gesture chord and does not implicitly apply edge expansion.

## Multi-axis routing (P0)

When multiple X/Y axes are present, `ChartCanvas` follows the “active axis pair” contract (ADR 0196):

- Hovering an axis band updates the active axis for that dimension.
- Interactions in the plot region target the active axis pair (X + Y), unless constrained by modifiers.
- Axis-band hit tests take precedence over plot-region fallback.
- Y slider UI targets the active Y axis (left/right) based on axis-band routing.

## Axis locks (P0)

`ChartCanvas` supports per-axis interaction locks:

- Lock pan: prevents panning the corresponding axis while dragging.
- Lock zoom: prevents zooming the corresponding axis via wheel or box zoom.

These locks live in `delinea::ChartState.axis_locks` and gate **interaction-derived** actions only
(see ADR 0197). For persistent axis constraints, use `delinea::AxisRange` in the spec/model:

- `AxisRange::Fixed { min, max }` disables interaction on that axis.
- `AxisRange::LockMin { .. }` / `LockMax { .. }` clamp interaction updates.

### Toggle gesture

- `Ctrl + LMB` toggles axis pan+zoom lock.

Axis targeting is based on the rendered layout:

- Pointer inside the X axis band toggles/targets X.
- Pointer inside the Y axis band toggles/targets Y.
- Pointer inside the plot rect toggles/targets both axes.

## Mapping to `delinea` actions

All UI input is mapped into headless actions:

- Hover: `Action::HoverAt { point: widget-local px }` (logical pixels)
- Toggle locks: `Action::ToggleAxisPanLock` / `Action::ToggleAxisZoomLock`
- Pan: `Action::PanDataWindowXFromBase` / `Action::PanDataWindowYFromBase`
- Wheel zoom: `Action::ZoomDataWindowXFromBase` / `Action::ZoomDataWindowYFromBase`
- Box zoom: `Action::SetViewWindow2DFromZoom` (plot-region drag) and `Action::SetDataWindow*FromZoom` (axis slider)
- Brush selection: `Action::SetBrushSelection2D` / `Action::ClearBrushSelection` (selection output; does not zoom)
- Reset: `Action::SetViewWindow2D { x: None, y: None }` (plot) or `Action::SetDataWindowX/Y { window: None }` (single axis)

## Hover / crosshair / tooltip (P0)

Current behavior is intentionally ImPlot-like for *gestures*, but the tooltip/axisPointer model is
ECharts-inspired and is controlled by `delinea::AxisPointerSpec`:

- `trigger=Axis` (default):
  - Crosshair + tooltip are shown while the pointer is in the plot rect. Hovering an axis band also
    drives the axisPointer (the hover point is clamped into the plot rect).
  - The tooltip shows an axis row followed by one row per visible series (stable `series_order`).
  - The crosshair is rendered as a single line (vertical for X-trigger, horizontal for Y-trigger).
  - The hover marker dot is only shown when a close-enough hit exists (gated by `trigger_distance_px`).
- `trigger=Item`:
  - Crosshair + tooltip are only shown when a close-enough hit exists (gated by `trigger_distance_px`).
  - The crosshair is rendered as a full crosshair (X+Y).
- `snap=true`:
  - Crosshair position and axis-trigger tooltip values may snap to the nearest hit point when one exists.

## Known limitations

- Multi-grid layout is not implemented yet; the current widget assumes a single plot rect.
- Axis rendering is intentionally minimal (no grid lines, titles, or rich formatting yet).
- Crosshair is currently rendered as solid lines (no dash pattern yet).
- Fit/reset requires focus (currently acquired by clicking the canvas).
- Brush selection highlight is UI-only in v1:
  - A selection rect is rendered in plot space.
  - Series not on the selected axis pair are dimmed.
  - Series on the selected axis pair are re-painted inside the selection rect via clip-based highlighting
    (outside the rect is dimmed).
