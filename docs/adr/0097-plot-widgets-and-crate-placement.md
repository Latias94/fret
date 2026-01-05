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

- data → geometry strategy (including downsampling/decimation and "avoid huge paths")
- hit testing, hover/tooltip, selection, pan/zoom policy (via action hooks + headless helpers)
- coordinate systems, axes, ticks, and text layout
- mapping results into `PathCommand` / `SceneOp::*`, reusing `PathService` caching as appropriate

`fret-render` owns:

- efficient, order-correct drawing of `Path/Text/SVG/Quad` (ADR 0002 / ADR 0009)

### 4) If we need richer paint semantics, upgrade contracts (do not hide it in the renderer)

If plot work requires configurable joins/caps/dashes, gradients/patterns, or AA strategy changes, prefer:

- extending ADR 0080 (Path v2), or introducing a separate paint abstraction ADR,
- and adding conformance tests to prevent renderer-only behavior drift.

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
