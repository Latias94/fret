# ADR 0096: Plot Widgets and Crate Placement (`fret-plot`)

Status: Accepted

## Context

Fret targets editor-grade, cross-platform UIs, but it should also be viable for general-purpose apps.
In both cases, "plot / chart / implot-like" components are a common requirement (time series, scatter,
histograms, crosshair + tooltip, pan/zoom, selection).

This ADR focuses on an ImPlot-like retained plot surface (`fret-plot`). For ECharts-style, dataset-driven
application charts, see ADR 0109 (`delinea` headless chart engine + `fret-chart` UI bridge).

Fret already has the renderer substrate required for a plot layer:

- `SceneOp::Path` + `PathService`: UI/components produce `PathCommand`s; the renderer tessellates via `lyon`
  with caching (see ADR 0080).
- `SceneOp::Text` + `TextService`: axes/ticks/labels/tooltips (see ADR 0006 / ADR 0029).
- SVG ops: icons/markers and general UI assets (`SceneOp::SvgMaskIcon` / `SceneOp::SvgImage`).

So the main question is not "can the renderer draw plots?", but rather where plot semantics and interaction
policy should live, and how to keep long-term portability (desktop-first now, wasm/WebGPU later).

## Decision

### 1) Add a dedicated plot crate in the ecosystem layer

Create `ecosystem/fret-plot` (crate name: `fret-plot`).

This crate is an ecosystem component/policy layer (like `fret-docking` and `fret-ui-kit`), not part of
the framework kernel. It is expected to be extractable to a future `fret-components` repository per ADR 0037,
without changing `fret-core` / `fret-ui` contracts.

### 2) Plot emits only existing portable scene primitives

`fret-plot` must build on top of existing portable primitives by emitting:

- `SceneOp::Path` (lines/areas/marker outlines)
- `SceneOp::Quad` (background, simple markers, simple grid lines as segments)
- `SceneOp::Text` (axes, ticks, labels, tooltips)
- `SceneOp::Image` / `SceneOp::ImageRegion` (data-aligned textures, icons, plot-space images)
- transform/clip/layer/opacity stacks

It must not introduce:

- plot-specific `SceneOp`s (unless proven necessary and locked via a separate ADR)
- any `wgpu`/`winit` types or dependencies (see ADR 0092 hard layering rules)
- a dependency on `fret-render` (the renderer backend must stay swappable)

### 3) Responsibility split: semantics & interaction above, acceleration below

`fret-plot` owns:

- data-to-geometry strategy (including downsampling/decimation and "avoid huge paths")
- hit testing, hover/tooltip, selection, pan/zoom policy (via action hooks + headless helpers)
- coordinate systems, axes, ticks, and text layout
- mapping results into `PathCommand` / `SceneOp::*`, reusing `PathService` caching as appropriate

`fret-render` owns:

- efficient, order-correct drawing of `Path/Text/SVG/Quad` (ADR 0002 / ADR 0009)

### 4) If we need richer paint semantics, upgrade contracts (do not hide it in the renderer)

If plot work requires configurable joins/caps/dashes, gradients/patterns, or AA strategy changes, prefer:

- extending ADR 0080 (Path v2), or introducing a separate paint abstraction ADR,
- and adding conformance tests to prevent renderer-only behavior drift.

### 5) Stable series identity is required

Plot interaction state must not be keyed by `Vec` indices. Series can be dynamically inserted/removed/reordered
(filtering, toggles, streaming data), and index-based identity will corrupt hover/pin/caches.

Instead, `fret-plot` uses stable series IDs:

- Each series has a `SeriesId`.
- Interaction state (hidden, pinned, hover, caches) is owned by the widget/canvas and keyed by `SeriesId`
  (model data stays “data-only”).

### 6) Multi-axis is explicit, not implicit

When a plot needs multiple Y scales, series must opt into a specific axis (e.g. `YAxis::Left` vs
`YAxis::Right`). The widget owns independent view state for each axis (e.g. `PlotState.view_bounds`
and `PlotState.view_bounds_y2`) while sharing the X range.

## Design and Performance

This ADR intentionally focuses on portable primitives and crate placement.
For the detailed retained-layer architecture and performance baseline, see:

- `docs/adr/0098-plot-architecture-and-performance.md`

## 3D Scope

This ADR covers 2D plots rendered via portable scene primitives.

If we want "real 3D" (depth-correct, GPU-friendly), we should treat Plot3D as an embedded viewport surface
and follow the existing engine viewport architecture. The UI-facing crate lives in
`ecosystem/fret-plot3d`; see ADR 0097.

## Overlays / annotations

Plot annotations (e.g. infinite reference lines, spans, callouts) are implemented as caller-owned overlays
stored in `PlotState`. See ADR 0104.

## Alternatives

### A) Implement plots in `fret-ui-shadcn`

Pros: fast iteration.

Cons: the shadcn layer is meant to stay "recipes/taxonomy"; plots are high-entropy and would make future
extraction harder.

### B) Put plots in `crates/` (kernel)

Cons: plot semantics and interaction policy are ecosystem concerns; moving them into the kernel raises the
long-term commitment cost and increases portability risk.

### C) Add plot-specific renderer/scene primitives

Cons: it locks high-entropy contracts too early; P0 plots are expressible via existing `Path/Quad/Text`.

### D) Immediate-mode plots (egui_plot / ImPlot-style)

Both `egui_plot` and ImPlot are immediate-mode libraries designed around their UI framework's rendering loop:
they rebuild plot geometry every frame and keep state via IDs in a global context.

Pros:

- very ergonomic for quick prototyping
- easy to "just draw" plots in a frame-based UI

Cons for Fret:

- fights retained caching (plot geometry and text shaping tend to be recomputed each frame)
- encourages high-frequency allocations and tessellation on large datasets
- state is typically stored in global contexts keyed by IDs, which is at odds with Fret's explicit model +
  widget-driven invalidation
- tight coupling to their painter abstractions makes it hard to reuse directly

We can still offer an ergonomic builder (frame-like API) on top of retained internals later, but the core
should remain retained and cache-driven.

## Follow-ups (P0)

- Implement `fret-plot` as a policy-heavy retained widget first (via `fret-ui`'s
  `unstable-retained-bridge`), to avoid expanding the declarative element contract prematurely.
- Start with a line plot that supports one or more series (the single-series case should stay
  frictionless), plus axes + pan/zoom + tooltip.
- Validate correctness under clip/transform/strict ordering.
- Add decimation to avoid per-frame pathological tessellation on large datasets.

## References

- GPUI Component plot substrate: `repo-ref/gpui-component/crates/ui/src/plot/mod.rs`
- GPUI Component chart wrappers: `repo-ref/gpui-component/crates/ui/src/chart/area_chart.rs`
- GPUI Component `IntoPlot` derive macro: `repo-ref/gpui-component/crates/macros/src/derive_into_plot.rs`
- `egui_plot` reference (vendored): `repo-ref/egui_plot`
- ImPlot reference (vendored): `repo-ref/implot`
- ImPlot3D reference (vendored): `repo-ref/implot3d`
- Plot3D rendering strategy: `docs/adr/0097-plot3d-rendering-strategy.md`
- Plot architecture and performance baseline: `docs/adr/0098-plot-architecture-and-performance.md`
