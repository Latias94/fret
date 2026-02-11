# ImPlot Alignment (Fret Plot)

This document tracks feature parity between:

- ImPlot (reference in `repo-ref/implot`)
- `fret-plot` (2D, retained)
- `fret-plot3d` (3D, retained; currently minimal)
- For Plot3D-specific parity tracking: `docs/audits/implot3d-alignment.md`.

The goal is not API compatibility. The goal is to match UX and core capabilities with clean,
token-driven theming and retained rendering/cache.

## Status Legend

- ✅ Implemented
- 🟡 Partial / different UX
- ❌ Not implemented

## Plot Items (ImPlot)

| ImPlot item | Fret equivalent | Status | Notes |
| --- | --- | --- | --- |
| `PlotLine` | `LinePlotCanvas` / `LineSeries` | ✅ | Supports multi-axis (Y1..Y4) and time/log formatting. |
| `PlotScatter` | `ScatterPlotCanvas` / `ScatterSeries` | ✅ | No dedicated demo yet (`scatter_demo.rs` missing). |
| `PlotStairs` | `StairsPlotCanvas` | ✅ | Demo: `apps/fret-examples/src/stairs_demo.rs`. |
| `PlotShaded` | `ShadedPlotCanvas` / `ShadedSeries` | ✅ | Demo: `apps/fret-examples/src/shaded_demo.rs`. |
| `PlotBars` | `BarsPlotCanvas` / `BarSeries` | ✅ | Demo: `apps/fret-examples/src/bars_demo.rs`. |
| `PlotBarGroups` | `BarsPlotCanvas` / `CategoryBarSeries` | ✅ | Demos: grouped/stacked bars. |
| `PlotErrorBars` | `ErrorBarsPlotCanvas` / `ErrorBarsSeries` | ✅ | Demo: `apps/fret-examples/src/error_bars_demo.rs`. |
| `PlotStems` | `StemsPlotCanvas` / `StemsSeries` | ✅ | Demo: `apps/fret-examples/src/stems_demo.rs`. |
| `PlotInfLines` | `PlotOverlays` (`InfLineX` / `InfLineY`) | ✅ | Demo: `apps/fret-examples/src/inf_lines_demo.rs`. |
| `PlotHeatmap` | `HeatmapPlotCanvas` | ✅ | Demo: `apps/fret-examples/src/heatmap_demo.rs`. |
| `PlotHistogram` | `HistogramPlotCanvas` / `HistogramSeries` | ✅ | Demo: `apps/fret-examples/src/histogram_demo.rs`. |
| `PlotCandlestick` | `CandlestickPlotCanvas` / `CandlestickSeries` | ✅ | Demo: `apps/fret-examples/src/candlestick_demo.rs`. |
| `PlotPieChart` | (none) | ❌ | Likely belongs to a charting layer, not a cartesian plot core. |
| `PlotDigital` | (none) | ❌ | Could be modeled as a step/segment series; clarify UX + sampling. |
| `PlotImage` | `PlotOverlays::images` (`PlotImage`) | ✅ | Renders `ImageId` as a data-aligned rect via `SceneOp::ImageRegion` (layers: below/above grid). |
| `PlotText` | `PlotOverlays::text` (`PlotText`) | 🟡 | Implemented as a caller-owned overlay (ADR 0104). No rich text/callouts yet. |
| `PlotHistogram2D` | `Histogram2DPlotCanvas` / `Histogram2DPlotModel` | ✅ | Implemented as a grid-backed plot (bins -> quads) with shared colormap + colorbar. |
| `PlotDummy` | (none) | ❌ | Not needed; can be handled by layout/legend policies if required. |

## Interactions & UX (ImPlot)

| Feature | Fret status | Notes / pointers |
| --- | --- | --- |
| Pan / zoom | ✅ | Implemented in retained canvas. |
| Axis pan/zoom lock | ✅ | `x_axis_pan_locked`, `x_axis_zoom_locked`, `y*_axis_*_locked`. |
| Box selection modifiers | ✅ | Matches ImPlot defaults (Alt/Shift expand-to-edge). |
| Linked plots / shared cursor | 🟡 | We have linking infra + demos (e.g. `linked_cursor_demo`), but no first-class subplot grid yet. |
| Legend interaction (hide, highlight, pin) | 🟡 | Implemented with retained caching; policy differs from ImPlot in details. |
| Tags (`TagX` / `TagY`) | ✅ | Implemented via `PlotOverlays::{tags_x,tags_y}` (ADR 0104). |
| Annotations | 🟡 | `PlotOverlays::text` exists; follow-ups include callouts, arrows, and editable anchors. |
| Drag tools (`DragPoint` / `DragLineX` / `DragLineY` / `DragRect`) | 🟡 | Implemented via `PlotOverlays` + `PlotOutputSnapshot::drag` (demo: `apps/fret-examples/src/drag_demo.rs`). Modifiers: `Shift` constrains inside-drags (point/rect) to X-only or Y-only; `Alt` snaps to nearest axis tick. |
| Subplots (`BeginSubplots`) | ❌ | Likely a UI-kit/layout concern, not plot core; needs design. |
| Aligned plots (`BeginAlignedPlots`) | ❌ | Could be implemented by sharing axis layout constraints across canvases. |
| Colormaps | 🟡 | Heatmap supports `ColorMapId` + an in-plot colorbar; no public registry UI yet. |
| SymLog axis scale | ❌ | `AxisScale` currently supports `Linear` and `Log10` only. |

## What We Should Add Next (Proposed Priorities)

P0 (high leverage for “commercial-grade” plot UX):

- Tags + text annotations (plot-space primitives, rendered as overlays).
- Drag tools (point/line/rect), with snapping and modifier-driven constraints.

P1 (broad capability expansion):

- 2D histogram (`PlotHistogram2D`) with a clear binning API and a streaming-friendly path.
- Colormap API surface (registry, sampling, and legend integration).
- Scatter demo + more “combinators” (multiple layers in one canvas).

P2 (charting vs plotting boundary):

- Pie chart / categorical charts: consider a separate “chart” surface, not the core plot.
- Digital plots: decide if it is a specialized series or a chart primitive.
