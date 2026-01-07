# Axis Interaction Alignment (ImPlot-style) — `fret-chart` + `delinea`

This document describes the current P0 interaction policy for `ChartCanvas` (the `fret-chart` retained widget)
and how it maps to `delinea`’s headless semantics.

The goal is to provide an ImPlot-like “muscle memory” baseline without committing to ImPlot API compatibility.

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

Box zoom selection modifiers (ImPlot style):

- Hold `Alt`: expand selection horizontally to the plot edges (zoom Y only)
- Hold `Shift`: expand selection vertically to the plot edges (zoom X only)

Note: if a modifier is required to start the drag gesture (e.g. `Shift + LMB drag`), it is treated as part of the
gesture chord and does not implicitly apply edge expansion.

## Axis locks (P0)

`ChartCanvas` currently supports UI-level axis locks:

- Lock pan: prevents panning the corresponding axis while dragging.
- Lock zoom: prevents zooming the corresponding axis via wheel or box zoom.

These locks are **pure UI policy** (local to the widget) and do not modify the chart spec/model.
For persistent axis constraints, use `delinea::AxisRange` in the spec/model:

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
- Pan: `Action::SetDataWindowX/Y { axis, window: Some(DataWindow) }`
- Zoom (wheel/box): same as above, with windows computed from pixel-space interaction
- Reset: `Action::SetDataWindowX/Y { axis, window: None }`

## Hover / crosshair / tooltip (P0)

Current behavior is intentionally ImPlot-like:

- Crosshair + tooltip are only shown when the pointer is inside the plot rect and close to a sampled data point.
- Crosshair position uses the pointer location.
- Hover marker uses the nearest sampled point (after LOD).
- Tooltip currently shows `series id`, `x`, `y` and uses the same numeric formatting as axis ticks.

## Known limitations

- Only a single primary X/Y axis is supported (no multi-axis / multi-grid yet).
- Axis rendering is minimal (no grid lines, titles, legends, or rich formatting yet).
- Crosshair is currently rendered as solid lines (no dash pattern yet).
- Fit/reset requires focus (currently acquired by clicking the canvas).
