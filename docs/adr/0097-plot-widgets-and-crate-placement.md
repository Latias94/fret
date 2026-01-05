# ADR 0097: Plot Widgets and Crate Placement (`fret-ui-plot`)

Status: Proposed

## Context

Fret targets editor-grade, cross-platform UIs, but it should also be viable for general-purpose apps.
In both cases, "plot / chart / implot-like" components are a common requirement (time series, scatter,
histograms, crosshair + tooltip, pan/zoom, selection).

Fret already has the renderer substrate required for a plot layer:

- `SceneOp::Path` + `PathService`: UI/components produce `PathCommand`s; the renderer tessellates via `lyon`
  with caching (see ADR 0080).
- `SceneOp::Text` + `TextService`: axes/ticks/labels/tooltips (see ADR 0006 / ADR 0029).
- SVG ops: icons/markers and general UI assets (`SceneOp::SvgMaskIcon` / `SceneOp::SvgImage`).

So the main question is not "can the renderer draw plots?", but rather where plot semantics and interaction
policy should live, and how to keep long-term portability (desktop-first now, wasm/WebGPU later).

## Decision

### 1) Add a dedicated plot crate in the ecosystem layer

Create `ecosystem/fret-ui-plot` (crate name: `fret-ui-plot`).

This crate is an ecosystem component/policy layer (like `fret-ui-docking` and `fret-ui-kit`), not part of
the framework kernel. It is expected to be extractable to a future `fret-components` repository per ADR 0037,
without changing `fret-core` / `fret-ui` contracts.

### 2) Plot emits only existing portable scene primitives

`fret-ui-plot` must build on top of existing portable primitives by emitting:

- `SceneOp::Path` (lines/areas/marker outlines)
- `SceneOp::Quad` (background, simple markers, simple grid lines as segments)
- `SceneOp::Text` (axes, ticks, labels, tooltips)
- transform/clip/layer/opacity stacks

It must not introduce:

- plot-specific `SceneOp`s (unless proven necessary and locked via a separate ADR)
- any `wgpu`/`winit` types or dependencies (see ADR 0093 hard layering rules)
- a dependency on `fret-render` (the renderer backend must stay swappable)

### 3) Responsibility split: semantics & interaction above, acceleration below

`fret-ui-plot` owns:

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

## Design Sketch (Recommended for Fret)

This ADR intentionally focuses on portable primitives and crate placement. For an initial API and
responsibility split that fits Fret's architecture, we recommend:

### 1) Retained-first UI integration (GPUI-aligned)

Start with a retained widget as the integration layer. This keeps plot state (hover/selection/cached paths)
in one place and avoids expanding the declarative element contract too early.

### 2) Headless helpers + thin scene emission

Structure the crate as:

- Headless modules: scales, tick generation, layout, decimation, hit testing, interaction math.
  - Input: data points, plot viewport, data bounds, style parameters, interaction state.
  - Output: a small set of computed values (ticks, transformed points, hit results).
- UI bridge modules: translate computed results into `SceneOp::{Path,Quad,Text}` using `PathService` and
  `TextService`, with caching keyed by model revision + viewport + scale factor + style.

This keeps "math and policy" portable while still using the renderer efficiently.

### 3) Performance model

The default rendering path should be CPU-driven and cache-friendly:

- Transform data points into plot-local logical pixels (`PlotTransform`).
- Apply decimation/downsampling bounded by plot width in device pixels (avoid "huge paths").
- Emit a single path per series where possible; keep axis labels and tick text cached.
- Use the decimated point set for hover hit testing to avoid per-pointer-event O(N) scans.

If/when this is insufficient, introduce renderer contract upgrades via separate ADRs (e.g. dashed strokes,
marker glyph atlases, polyline/line-strip semantics, or GPU line rendering).

### 4) Public API shape (P0/P1 direction)

Prefer a split between a generic substrate and small chart wrappers:

- `PlotCanvas` (or similar): owns layout, axes, interaction, and series painting hooks.
- `LineChart`, `ScatterPlot`, `BarChart`: thin wrappers that configure a `PlotCanvas` with one or more series.

This mirrors GPUI component's design and keeps high-entropy features isolated per chart type.

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

- Implement `fret-ui-plot` as a policy-heavy retained widget first (via `fret-ui`'s
  `unstable-retained-bridge`), to avoid expanding the declarative element contract prematurely.
- Start with a single-series line plot + axes + pan/zoom + tooltip.
- Validate correctness under clip/transform/strict ordering.
- Add decimation to avoid per-frame pathological tessellation on large datasets.

## References

- GPUI Component plot substrate: `repo-ref/gpui-component/crates/ui/src/plot/mod.rs`
- GPUI Component chart wrappers: `repo-ref/gpui-component/crates/ui/src/chart/area_chart.rs`
- GPUI Component `IntoPlot` derive macro: `repo-ref/gpui-component/crates/macros/src/derive_into_plot.rs`
- `egui_plot` reference (vendored): `repo-ref/egui_plot`
- ImPlot reference (vendored): `repo-ref/implot`
- ImPlot3D reference (vendored): `repo-ref/implot3d`
