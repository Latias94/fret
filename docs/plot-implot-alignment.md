# Plot / ImPlot Alignment Checklist

This document tracks feature and behavior alignment between `fret-ui-plot` and the baseline UX of
ImPlot. The goal is not strict API compatibility, but to converge on the parts of ImPlot that have
proven to be ergonomic and predictable for editor-grade UIs.

Status symbols:

- `[x]` implemented
- `[ ]` planned / missing
- `[~]` partial / needs revision

## Scope

- 2D plots only (`fret-ui-plot`).
- 3D is tracked separately (see ADR 0098 / Plot3D demos).

## Current architecture (Fret)

- Input state: `PlotState` (caller-owned, optional)
- Output state: `PlotOutput` (widget-written snapshot, optional)
- Multi-plot linking: `LinkedPlotGroup`

This contract is documented in ADR 0099.

## Interaction & UX

- [x] Pan (left drag)
- [x] Zoom (mouse wheel, `Shift`/`Ctrl` axis modifiers)
- [x] Box zoom (RMB drag; optional `Shift+LMB` drag)
- [x] Query selection (Alt + drag) stored in `PlotState.query`
- [x] Query/zoom drag shows range readout tooltip
- [x] Crosshair visible when cursor is inside plot
- [x] Mouse position readout (overlay or tooltip) when cursor is inside plot
- [x] Series value readout (cursor X -> per-series Y; sorted-by-x interpolation, else view-sampled/budgeted fallback)
- [x] Tooltip/readout uses axis formatters (consistent units/time)
- [x] Legend interaction: hide/solo/pin (basic)
- [x] Cursor linking across plots (vertical cursor + per-series readout at X)
- [x] Linked cursor readout defaults to overlay (configurable tooltip)
- [x] Linked cursor readout supports pinned/hover filtering
- [x] Selection/query linking across plots (built on top of `LinkedPlotGroup`)
- [x] Line hover uses nearest segment distance (not just sampled points)
- [~] Keyboard shortcuts matrix (documented; partially aligned with ImPlot defaults)

See also `docs/plot-axis-interactions.md` for axis-region routing, fit behavior, and lock shortcuts.

### Keyboard shortcuts & mouse gestures

This table describes the default mapping used by `fret-ui-plot` today, with a side-by-side reference
to ImPlot's default `ImPlotInputMap`.

| Action | ImPlot default | `fret-ui-plot` default | Notes |
| --- | --- | --- | --- |
| Pan | `LMB drag` | `LMB drag` | Matches ImPlot. |
| Pan (axis-only) | Drag axis region | Drag axis region | X axis drag pans X-only; Y axis drag pans the corresponding Y axis only. |
| Zoom | `Wheel` | `Wheel` | In the plot region: `Shift` = X-only, `Ctrl` = Y-only. ImPlot has no axis-only modifiers by default. |
| Zoom (axis-only) | Wheel on axis region | Wheel on axis region | X axis wheel zooms X-only; Y axis wheel zooms the corresponding Y axis only. |
| Box select / zoom | `RMB drag` | `RMB drag` and `Shift+LMB drag` | `RMB drag` follows ImPlot. `Shift+LMB` is kept as an accessibility-friendly alternative. |
| Box select expand (horizontal) | Hold `Alt` | Hold `Alt` (RMB box zoom only) | Expands selection to plot edges on X. |
| Box select expand (vertical) | Hold `Shift` | Hold `Shift` (RMB box zoom only) | Expands selection to plot edges on Y. |
| Cancel box select | `LMB press` | `LMB press` (when RMB selecting) or `Esc` | `Esc` cancels any active drag. |
| Fit / reset view | `LMB double-click` | `LMB double-click` | Double-click in plot region fits all axes; double-click on an axis fits that axis. `R` remains as an explicit "reset everything" shortcut (also clears hidden/pinned/query). |
| Clear query selection | N/A (app-owned) | `Q` | `PlotState.query` is application-controlled state. |
| Restore legend visibility | N/A | `A` | Clears hidden/pinned series. |

The default mapping is configurable via `PlotCanvas::input_map(PlotInputMap)`.

## Data & identity

- [x] Stable series identity (`SeriesId` derived from label or explicit ID)
- [x] Zero-copy data adapters (slice, `Arc<[DataPoint]>`, getter-based)
- [x] `f64` data domain (time axes / large coordinates)
- [x] Discontinuities via `None`/NaN/Inf break segments

## Performance baseline

- [x] CPU decimation bounded by viewport pixels (min/max per X bucket)
- [x] Cached paths keyed by `(SeriesId, model_revision, viewport_px, scale, view_bounds, style_key)`
- [x] Separate decimation strategies per plot type (polyline vs point bucket sampling)
- [x] Optional hit-test acceleration for monotonic-X series (windowed slice scan)

## Plot types (P1)

- [x] Line plot
- [x] Scatter plot (markers)
- [x] Bars plot (filled rectangles)
- [x] Grouped bars (bar groups)
- [x] Stacked bars (positive/negative stacks)
- [x] Histogram plot
- [x] Area (fill to baseline)
- [x] Shaded band (fill between upper/lower series)
- [x] Stairs / Step
- [x] Heatmap (quad-based, portable)
- [x] Error bars
- [x] Candlesticks / OHLC

## Axes

- [x] Nice ticks (1/2/5 * 10^n)
- [x] Adaptive tick density (avoid label overlap)
- [x] Time axis + formatting (relative seconds or Unix UTC, calendar-aligned UTC ticks)
- [x] Log axis (Log10 scale + ticks + interactions)
- [x] Log axis tick labels (major decades labeled; minor ticks are grid-only by default)
- [x] Axis formatters (custom label callbacks with stable cache key)
- [x] Axis thickness auto-fit (avoid clipped labels)
- [x] Multi-axes (Y2/Y3/Y4)
- [x] Axis lock (lock X/Y pan/zoom)
- [x] Axis constraints (limits + min/max span)

## Styling

- [x] Theme-derived defaults
- [~] Per-series style overrides beyond color (stroke width; scatter marker radius/shape; error bars marker shape)
- [ ] Dashes / joins / caps (likely requires renderer/path contract follow-up)

## Demos

- [x] `plot_demo` (desktop + web)
- [x] `bars_demo` (desktop + web)
- [x] `grouped_bars_demo` (desktop + web)
- [x] `stacked_bars_demo` (desktop + web)
- [x] `area_demo` (desktop + web)
- [x] `heatmap_demo` (desktop + web)
- [x] `histogram_demo` (desktop + web)
- [x] `stairs_demo` (desktop + web)
- [x] `shaded_demo` (desktop + web)
- [x] `linked_cursor_demo` (desktop + web)
- [x] Desktop-only stress harness (large datasets)
- [x] Linked plots demo (covered by `linked_cursor_demo`)

## Next steps (recommended order)

1. Per-series style overrides (line/marker style beyond color).
2. Dashes / joins / caps (may require renderer/path contract follow-up).
3. Context menu parity (optional; not tracked yet).
