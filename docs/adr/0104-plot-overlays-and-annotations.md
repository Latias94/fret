# ADR 0104: Plot Overlays and Annotations (2D, `fret-plot`)

Status: Proposed

## Context

ImPlot and egui_plot provide a rich set of plot "annotations" that are not part of the underlying
series model:

- infinite reference lines (vertical/horizontal),
- spans (filled ranges),
- callouts/labels,
- draggable markers and guides.

For Fret, we want these features while keeping:

- retained, cache-friendly geometry generation (ADR 0099),
- portable rendering primitives (`SceneOp::{Path,Quad,Text}` only; ADR 0097),
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

### 2) P0 overlay surface: static infinite reference lines (InfLines)

P0 supports static, non-interactive reference lines:

- `InfLineX { x: f64, color: Option<Color>, width: Px }`
- `InfLineY { y: f64, axis: YAxis, color: Option<Color>, width: Px }`

Rendering uses the current `PlotTransform` to map data coordinates into the local plot viewport and emits
`SceneOp::Quad` rectangles clipped by the plot region.

### 3) Interaction is layered (P1+)

To keep the baseline clean, interaction is a follow-up:

- P1: hover affordances + tooltip integration (e.g. "line label at axis edge").
- P1: draggable reference lines (opt-in per overlay).
- P2: spans (filled X/Y ranges), text callouts, and arbitrary polyline annotations.

Interactive overlays remain caller-owned, with widget-produced outputs flowing through `PlotOutput`
(mirroring selection/query behavior).

## Relationship to Other ADRs

- ADR 0097: plot crate placement and portable rendering constraints.
- ADR 0099: retained plot architecture and caching baseline.
- `docs/plot-implot-alignment.md`: feature checklist (including `inf_lines_demo`).

This ADR only defines the overlay/annotation contract. It does not change the rendering substrate or
the series model contract.

## 3D Notes

This ADR covers 2D plot overlays rendered via portable scene primitives.

For Plot3D (ADR 0098), annotations are expected to be implemented either:

- as 2D overlays around the viewport surface (labels, UI chrome), or
- as 3D guides inside the viewport (requires Plot3D-specific rendering and hit testing).

## References

- ImPlot: `ImPlot::DragLineX`, `ImPlot::DragLineY`, `ImPlot::PlotInfLines`
- egui_plot annotations: `HLine`, `VLine`, marker APIs
- GPUI component plot substrate: `repo-ref/gpui-component/crates/ui/src/plot/mod.rs`
