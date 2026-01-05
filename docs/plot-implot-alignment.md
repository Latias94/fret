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
- [x] Crosshair visible when cursor is inside plot
- [x] Cursor coordinate tooltip visible when cursor is inside plot
- [~] Series value readout (cursor X -> per-series Y; currently implemented via sorted-by-x interpolation)
- [x] Legend interaction: hide/solo/pin (basic)
- [ ] Cursor linking across plots (vertical cursor + per-series readout at X)
- [ ] Selection/query linking across plots (built on top of `LinkedPlotGroup`)
- [ ] Keyboard shortcuts matrix (document + align with ImPlot defaults)

## Data & identity

- [x] Stable series identity (`SeriesId` derived from label or explicit ID)
- [x] Zero-copy data adapters (slice, `Arc<[DataPoint]>`, getter-based)
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
- [x] Area / Shaded
- [ ] Stairs / Step
- [ ] Heatmap
- [ ] Error bars
- [ ] Candlesticks / OHLC

## Axes

- [x] Linear ticks (baseline)
- [ ] Log axis
- [ ] Time axis + formatting
- [ ] Axis formatters (custom label callbacks)
- [ ] Multi-axes (dual Y, etc.)
- [ ] Axis lock (lock X/Y pan/zoom)

## Styling

- [x] Theme-derived defaults
- [ ] Per-series style overrides beyond color (line style, marker style)
- [ ] Dashes / joins / caps (likely requires renderer/path contract follow-up)

## Demos

- [x] `plot_demo` (desktop + web)
- [x] `bars_demo` (desktop + web)
- [x] Desktop-only stress harness (large datasets)
- [ ] Linked plots demo (multiple plots with shared view/query)

## Next steps (recommended order)

1. Cursor linking UX (vertical cursor + per-series readout at X).
2. Add `Area/Shaded` and `Stairs/Step` plot layers.
3. Axis formatting (time + custom formatters).
4. Expand plot type set (heatmap, error bars, candlesticks).
