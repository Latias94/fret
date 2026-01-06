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
- [x] Box zoom (Shift + drag)
- [x] Query selection (Alt + drag) stored in `PlotState.query`
- [x] Query/zoom drag shows range readout tooltip
- [x] Crosshair visible when cursor is inside plot
- [x] Mouse position readout (overlay or tooltip) when cursor is inside plot
- [~] Series value readout (cursor X -> per-series Y; currently implemented via sorted-by-x interpolation)
- [x] Tooltip/readout uses axis formatters (consistent units/time)
- [x] Legend interaction: hide/solo/pin (basic)
- [x] Cursor linking across plots (vertical cursor + per-series readout at X)
- [x] Selection/query linking across plots (built on top of `LinkedPlotGroup`)
- [ ] Keyboard shortcuts matrix (document + align with ImPlot defaults)

## Data & identity

- [x] Stable series identity (`SeriesId` derived from label or explicit ID)
- [x] Zero-copy data adapters (slice, `Arc<[DataPoint]>`, getter-based)
- [x] `f64` data domain (time axes / large coordinates)
- [x] Discontinuities via `None`/NaN/Inf break segments

## Performance baseline

- [x] CPU decimation bounded by viewport pixels (min/max per X bucket)
- [x] Cached paths keyed by `(SeriesId, model_revision, viewport_px, scale, view_bounds, style_key)`
- [ ] Separate decimation strategies per plot type (line vs bars vs points)
- [ ] Optional hit-test acceleration for monotonic-X series (binary search / interval trees)

## Plot types (P1)

- [x] Line plot
- [x] Scatter plot (marker crosses)
- [x] Bars plot (filled rectangles)
- [x] Area (fill to baseline)
- [x] Shaded band (fill between upper/lower series)
- [x] Stairs / Step
- [x] Heatmap (quad-based, portable)
- [ ] Error bars
- [ ] Candlesticks / OHLC

## Axes

- [x] Nice ticks (1/2/5 * 10^n)
- [x] Adaptive tick density (avoid label overlap)
- [~] Time axis + formatting (UTC seconds baseline)
- [x] Log axis (Log10 scale + ticks + interactions)
- [~] Log axis tick labels (major decades labeled; minor ticks are grid-only by default)
- [~] Axis formatters (custom label callbacks with stable cache key)
- [x] Axis thickness auto-fit (avoid clipped labels)
- [~] Multi-axes (Y2 supported; no Y3/Y4 yet)
- [x] Axis lock (lock X/Y pan/zoom)
- [x] Axis constraints (limits + min/max span)

## Styling

- [x] Theme-derived defaults
- [ ] Per-series style overrides beyond color (line style, marker style)
- [ ] Dashes / joins / caps (likely requires renderer/path contract follow-up)

## Demos

- [x] `plot_demo` (desktop + web)
- [x] `bars_demo` (desktop + web)
- [x] `area_demo` (desktop + web)
- [x] `heatmap_demo` (desktop + web)
- [x] `stairs_demo` (desktop + web)
- [x] `shaded_demo` (desktop + web)
- [x] `linked_cursor_demo` (desktop + web)
- [x] Desktop-only stress harness (large datasets)
- [x] Linked plots demo (covered by `linked_cursor_demo`)

## Next steps (recommended order)

1. Cursor linking UX (vertical cursor + per-series readout at X).
2. Axis formatting (time + custom formatters).
3. Expand plot type set (heatmap, error bars, candlesticks).
