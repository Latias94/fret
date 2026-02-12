# ADR 0104: Plot Overlays and Annotations (2D, `fret-plot`)

Status: Accepted

## Context

ImPlot and egui_plot provide a rich set of plot "annotations" that are not part of the underlying
series model:

- infinite reference lines (vertical/horizontal),
- spans (filled ranges),
- callouts/labels,
- draggable markers and guides.

For Fret, we want these features while keeping:

- retained, cache-friendly geometry generation (ADR 0098),
- portable rendering primitives (`SceneOp::{Path,Quad,Text,ImageRegion}`; ADR 0096),
- a clean, composable API that can be shared by all 2D plot types.

## Decision

### 1) Overlays are caller-owned and stored in `PlotState`

`PlotState` is the right place for plot-local, app-controlled state and decorations. Overlays should be:

- explicit (no global context),
- composable (works with all plot canvases),
- stable across frames (so caching remains effective).

So `fret-plot` adds:

- `PlotState::overlays: PlotOverlays`
- `PlotOverlays` contains per-overlay collections (starting with reference lines).

### 2) P0 overlay surface: static overlays (InfLines, Tags, PlotText)

P0 supports static, non-interactive overlays:

- `InfLineX { x: f64, color: Option<Color>, width: Px }`
- `InfLineY { y: f64, axis: YAxis, color: Option<Color>, width: Px }`
- `PlotImage { image: ImageId, rect: DataRect, axis: YAxis, ... }` (data-aligned RGBA images)
- `TagX { x: f64, label: Option<String>, show_value: bool, color: Option<Color> }`
- `TagY { y: f64, axis: YAxis, label: Option<String>, show_value: bool, color: Option<Color> }`
- `PlotText { x: f64, y: f64, axis: YAxis, text: String, ... }`

Rendering uses the current `PlotTransform` to map data coordinates into the local plot viewport and emits
`SceneOp::Quad` / `SceneOp::Text` / `SceneOp::ImageRegion` primitives clipped by the plot region.

Theming is token-driven (see `docs/plot-theme-tokens.md`), with optional annotation tokens:

- `fret.plot.annotation.background` / `plot.annotation.background`
- `fret.plot.annotation.border` / `plot.annotation.border`
- `fret.plot.annotation.text` / `plot.annotation.text`
- `fret.plot.annotation.stroke` / `plot.annotation.stroke`
- `fret.plot.annotation.padding` / `plot.annotation.padding`
- `fret.plot.annotation.radius` / `plot.annotation.radius`

### 3) Interaction is layered (P1+)

To keep the baseline clean, interaction is a follow-up:

- P1: hover affordances + tooltip integration (e.g. "line label at axis edge").
- P1: draggable points, reference lines, and rectangles (opt-in per overlay), with outputs reported via `PlotOutputSnapshot::drag`.
- P2: spans (filled X/Y ranges), text callouts, and arbitrary polyline annotations.

Interactive overlays remain caller-owned, with widget-produced outputs flowing through `PlotOutput`
(mirroring selection/query behavior).

## Relationship to Other ADRs

- ADR 0096: plot crate placement and portable rendering constraints.
- ADR 0098: retained plot architecture and caching baseline.
- `docs/audits/implot-alignment.md`: feature checklist (including `inf_lines_demo`).

This ADR only defines the overlay/annotation contract. It does not change the rendering substrate or
the series model contract.

## 3D Notes

This ADR covers 2D plot overlays rendered via portable scene primitives.

For Plot3D (ADR 0097), annotations are expected to be implemented either:

- as 2D overlays around the viewport surface (labels, UI chrome), or
- as 3D guides inside the viewport (requires Plot3D-specific rendering and hit testing).

## References

- ImPlot: `ImPlot::DragPoint`, `ImPlot::DragLineX`, `ImPlot::DragLineY`, `ImPlot::DragRect`, `ImPlot::PlotInfLines`
- egui_plot annotations: `HLine`, `VLine`, marker APIs
- GPUI component plot substrate: `repo-ref/gpui-component/crates/ui/src/plot/mod.rs`
