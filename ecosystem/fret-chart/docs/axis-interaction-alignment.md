# Axis Interaction Alignment (ImPlot-style) — `fret-chart` + `delinea`

This document describes the current P0 interaction policy for `ChartCanvas` (the `fret-chart` retained widget)
and how it maps to `delinea`’s headless semantics.

The goal is to provide an ImPlot-like “muscle memory” baseline without committing to ImPlot API compatibility.

## Coordinate conventions

- Screen space: Fret logical pixels, origin at the top-left of the widget bounds.
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

Because axes are not rendered yet in `fret-chart`, the current P0 hit targets are a heuristic:

- Left 24px band: toggles Y axis lock
- Bottom 24px band: toggles X axis lock
- Elsewhere: toggles both axes

This will be replaced by explicit axis layout regions once we render axes and labels in `fret-chart`.

## Mapping to `delinea` actions

All UI input is mapped into headless actions:

- Hover: `Action::HoverAt { point: window-local px }`
- Pan: `Action::SetDataWindowX/Y { axis, window: Some(DataWindow) }`
- Zoom (wheel/box): same as above, with windows computed from pixel-space interaction
- Reset: `Action::SetDataWindowX/Y { axis, window: None }`

## Known limitations

- No axis rendering yet (ticks/labels), so axis-region detection for lock toggles is temporary.
- Box zoom currently uses the widget bounds as the plot area (no padding/margins).
- Fit/reset requires focus (currently acquired by clicking the canvas).
