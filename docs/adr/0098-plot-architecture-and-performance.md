# ADR 0098: Plot Architecture and Performance Baseline (2D Layers + 3D Viewport Surfaces)

Status: Proposed

## Context

Fret targets editor-grade, cross-platform UIs, and should also be viable for general-purpose apps.
Interactive plots are a common requirement in both domains (time series, scatter, histograms, heatmaps,
crosshair + tooltip, pan/zoom, selection/query regions).

We already have the renderer substrate required for 2D plots:

- `SceneOp::Path` + `PathService` tessellation/caching (ADR 0080).
- `SceneOp::Text` + `TextService` (ADR 0006 / ADR 0029).
- SVG ops for icons/markers and general UI assets.

And we already have a portability-preserving route for real 3D:

- Viewport surfaces via `SceneOp::ViewportSurface` (ADR 0007).
- Engine render hook pipeline (ADR 0038).
- Viewport input forwarding (ADR 0025).

So the remaining design work is to lock down a **clean, cache-friendly plot architecture** that:

- stays within Fret’s layering rules (no `wgpu`/`winit` in UI ecosystem crates),
- scales to large datasets without pathological per-frame work,
- keeps API surface small and evolvable.

This ADR synthesizes best practices observed in:

- `egui_plot` (immediate-mode plot state keyed by IDs, item identity requirements, borrowed/generator data).
- ImPlot (setup vs set-next configuration, data striding + getters to avoid copying, large-but-not-infinite dataset guidance).
- ImPlot3D (3D interaction + GPU constraints, mesh/surface complexity, index-size pitfalls).
- GPUI component plot/chart substrate (retained `Plot` trait, scale modules, builder-style shapes, stroke styles).

## Relationship to Other ADRs

- ADR 0096 defines **where plots live** (`ecosystem/fret-plot`) and the **portable scene primitive constraint**
  (plots must emit `SceneOp::{Path,Quad,Text}` and remain `wgpu`/`winit` free).
- ADR 0097 defines the **Plot3D route** (embedded viewport surfaces + input forwarding).

This ADR does not replace those decisions. It defines a cohesive retained architecture and performance baseline
for implementations that conform to ADR 0096 and ADR 0097.

## Decision

### 1) 2D plot is a retained canvas with composable layers

The primary integration surface is a retained widget:

- `PlotCanvas<L: PlotLayer>`: owns layout, axes, interaction state, caching keys, and event handling.
- `PlotLayer`: produces series geometry for a given model + viewport + style.

This aligns with Fret’s retained-first direction (ADR 0005 / ADR 0051) while still allowing
an ergonomic builder API on top if desired.

### 1.25) Input mapping is explicit and configurable

`PlotCanvas` uses a small `PlotInputMap` struct to define pointer chords for pan/fit/box-zoom/query
and selection modifiers. The default mapping follows ImPlot's `MapInputDefault` (with an optional
`Shift+LMB` box-zoom alternative for accessibility).

### 2) Stable identity is mandatory (no index-keyed state)

Series identity must be stable across reordering, filtering, and streaming updates:

- Series are keyed by `SeriesId` (see ADR 0096).
- Interaction state (hidden, pinned, hover, caches) lives in the widget/canvas and is keyed by `SeriesId`.

This mirrors the ID-based state strategy in `egui_plot` and the label-ID registration model in ImPlot,
but keeps state local to the widget rather than a global context.

### 2.5) Multi-axis (Y2/Y3/Y4) is a second view, not a second plot

To align with common ImPlot usage while keeping contracts portable, multi-axis support is modeled as:

- A per-series axis choice (`YAxis::{Left,Right,Right2,Right3}`) rather than implicit scaling.
- A shared X range (pan/zoom in X affects all axes).
- Independent Y view bounds per axis (`PlotState.view_bounds`, `PlotState.view_bounds_y2`,
  `PlotState.view_bounds_y3`, `PlotState.view_bounds_y4`), allowing different scales without
  distorting the primary Y axis.

Implementation note:

- Axes are auto-enabled when the model contains series assigned to them, so forgetting to call
  `y*_axis_*` configuration APIs does not silently plot those series against the primary Y axis.

### 2.6) Axis scales are explicit (Linear / Log10)

Axis scaling is an explicit, per-axis configuration on the canvas (not hidden inside series data):

- `AxisScale::{Linear,Log10}` is carried by the canvas for X, Y, Y2, Y3, and Y4.
- `PlotTransform` maps **data space → axis space → pixels** so pan/zoom math and clamping behave
  consistently across scales.
- Tick generation is scale-aware (log axes always produce log ticks regardless of "nice/linear" hints).
- Cache keys include axis scale so switching a scale forces path/tick rebuild.

Log10 constraints:

- Non-positive values are not representable; invalid samples break segments and are skipped for hit testing.
- View/data bounds are sanitized into a deterministic positive domain to keep transforms well-defined.

### 3) Data access is adapter-based (zero-copy first)

To avoid forcing allocations/copies for large datasets, series data should be representable as:

- borrowed slices (`&[Point]` / `&[SamplePoint]`),
- owned shared buffers (`Arc<[Point]>`),
- generated/on-demand accessors (a “getter + count” pattern).

This is directly inspired by ImPlot’s “stride + getter” approach and `egui_plot`’s
`PlotPoints::{Borrowed,Owned,Generator}`.

Additionally, plots should support **view-range sampling** for generator-like sources and chunked
stores:

- `SeriesData::sample_range(x_range, budget) -> Option<Vec<DataPoint>>` allows a series to provide a
  bounded set of points for the currently visible X range without requiring the plot to iterate the
  entire dataset.
- `Series::from_explicit_callback(y=f(x), x_range, points)` is the ergonomic entry point for
  function plots (mirrors `egui_plot::PlotPoints::from_explicit_callback`).

### 4) Performance baseline: CPU-driven LOD + cache reuse

The default rendering path is **CPU-driven and cache-friendly**:

1. Transform data points into plot-local pixel space (`PlotTransform`).
2. Apply LOD/decimation bounded by the viewport width in device pixels.
3. Emit scene primitives (`Path/Quad/Text`) and rely on renderer caching (ADR 0080).

If later we need richer semantics (dashes, gradients, GPU line strips), we upgrade contracts via
dedicated ADRs rather than hiding behavior in the renderer.

### 5) Plot3D stays route B1: embedded viewport surfaces

Plot3D is not a “3D scene op” at P0/P1. It is an embedded viewport surface with portable input events.

This keeps 3D correctness (depth test) and performance (instancing/meshes) in the engine pipeline,
while keeping UI contracts portable and wgpu-free.

## Design (Recommended for Fret)

### A) Modules (keep math/policy headless where possible)

In `fret-plot`, prefer a split between:

- `plot/*` (headless-ish):
  - transforms and coordinate math,
  - tick generation (linear/log/time),
  - decimation / resampling,
  - hit testing helpers (nearest-point, nearest-segment, interval search for monotonic X),
  - interaction math (pan/zoom constraints, cursor linking).
- `retained/*` (UI bridge):
  - layout and rendering into `SceneOp::*`,
  - caching keys and invalidation,
  - pointer/scroll/key input -> plot actions,
  - legend/tooltip/crosshair policy.
  - suggested internal split (current codebase):
    - `retained/canvas.rs`: widget + event routing + scene emission (generic `PlotCanvas<L>`)
    - `retained/layout.rs`: plot/axis region geometry + hit testing
    - `retained/layers.rs`: `PlotLayer` + concrete plot layers + paint/hit-test helpers
    - `retained/state.rs`: `PlotState` / `PlotOutput` + snapshots
    - `retained/models.rs`: `*Series` / `*PlotModel` data types
    - `retained/style.rs`: `LinePlotStyle` and related styling enums

This matches the “headless core + thin renderer bridge” approach in ADR 0096.

### B) Layer contract (what layers are allowed to do)

`PlotLayer` should be “pure-ish”:

- Input: model reference, plot viewport, transform, style/theme, interaction snapshot.
- Output: scene payload (paths/quads/text) and per-series metadata needed for legend/hit-testing.

Layers should not:

- mutate the model,
- depend on renderer implementation details,
- allocate unbounded memory per frame (bounded by viewport pixels).

### C) Downsampling / LOD strategy (P0/P1)

Start with a single robust baseline that behaves well for large datasets:

- 1D timeseries / monotonic-X: **min/max per pixel column** (two samples per column),
  preserving spikes and avoiding aliasing.
- General polylines: **max-error decimation** or LTTB-style selection to target
  `O(viewport_px_width)` points.

Caching strategy:

- cache decimated points per `(SeriesId, model_revision, viewport_px, scale, bounds, style_key)`.
- cache axis tick layout per `(viewport_px, scale, bounds, font_key)`.

### D) Interaction and state ownership

Split state into:

- **Ephemeral widget state** (should not be persisted automatically):
  - hover target, transient drag state, last pointer position, in-progress selection box.
- **User preference state** (optionally persisted by the app):
  - hidden series set, pinned series, axis locking, cursor-linking, manual bounds.

This avoids storing long-lived preferences in a global UI context (ImPlot/egui-style) while still
supporting persistence in editor apps.

#### Input/Output contract (recommended)

To enable linking plots, building inspectors, and persisting view preferences without leaking plot
internals, plots should expose:

- **Input state** (`PlotState`, caller-owned, optional):
  - A `Model<PlotState>` that the caller can pass into the plot widget.
  - Stores long-lived user preferences: view bounds (auto vs manual), hidden/pinned series, query
    regions, etc.
  - This mirrors ImPlot’s "set-next" and egui_plot’s ID-keyed state, but keeps the state explicitly
    owned and managed by the application.
- **Output state** (`PlotOutput`, caller-owned, optional):
  - A `Model<PlotOutput>` that the plot widget writes to whenever interaction-derived values change.
  - Contains a monotonic `revision` plus a compact snapshot (view bounds, cursor/hover in data space,
    active query selection, etc.).
  - Intended as an observation point only. Application code should not mutate `PlotOutput` directly;
    use `PlotState` to control the plot instead.

### E) 3D UI bridge shape (P0/P1)

`fret-plot3d` provides:

- `Plot3dCanvas` that emits `SceneOp::ViewportSurface { target, ... }`,
- viewport input forwarding helpers built on `ViewportMapping` -> `ViewportInputEvent` (ADR 0132):
  - retained: `crates/fret-ui/src/retained_bridge.rs` (`viewport_surface::handle_viewport_surface_input`)
  - declarative: `ecosystem/fret-ui-kit/src/declarative/viewport_surface.rs` (`viewport_surface_panel`)
- optional overlay primitives (axes gizmo, HUD text) implemented as regular `SceneOp::Text/Path/Quad`
  above the surface.

The engine/driver owns:

- the `RenderTargetId` allocation,
- rendering into that target (depth, shading, picking buffers),
- camera controller state (orbit/pan/zoom), driven by forwarded input.

This matches the practical constraints highlighted by ImPlot3D (depth correctness and GPU data volume).

## Consequences

- We get a portable 2D plot stack that reuses the existing renderer and its caching.
- We get a 3D path that is correct and fast without polluting UI crates with wgpu types.
- Large dataset behavior becomes an explicit contract (LOD bounded by viewport pixels).
- Future upgrades (dashes, gradients, GPU line strips, glyph atlases) are staged behind targeted ADRs.

## Numeric Precision

Plot data coordinates use **`f64`** (especially important for time axes such as Unix seconds). Screen
space remains `f32` (`Px`) at the rendering boundary. This matches the practical precision needs
observed in `egui_plot` and avoids a class of “time axis jitter” and “large coordinate collapse”
issues on narrow views.

## Alternatives Considered

### A) Immediate-mode API as the core (egui_plot / ImPlot style)

Pros:

- extremely ergonomic for “draw plots in this frame”
- easy to add plot items ad-hoc

Cons for Fret:

- fights retained caching and explicit invalidation (ADR 0051),
- encourages per-frame rebuilding of geometry/text,
- typically stores state in global contexts keyed by IDs,
- hard to keep performance predictable on large datasets without deeper caching.

We can add an optional “builder” facade later, but the core should remain retained and cache-driven.

### B) Add plot-specific scene ops (polyline/markers/mesh)

Pros:

- potentially faster by encoding high-level primitives

Cons:

- locks high-entropy contracts too early,
- requires conformance tests across renderers,
- increases long-term maintenance burden.

Defer until the CPU+cache baseline proves insufficient, then introduce via a dedicated ADR.

## Follow-ups (Suggested Work Items)

### P0 (stabilize the substrate)

- Add a “series data adapter” trait that supports slices + getters without allocation.
- Lock an explicit decimation policy and add unit tests for it (correctness on spikes, monotonicity, NaNs).
- Add a plot stress/perf harness integrated with ADR 0095 (large-series + interaction scenarios).
- Standardize plot state IO: caller-owned `PlotState` + widget-written `PlotOutput` snapshots.

### P1 (feature growth without entropy explosion)

- Add additional layers: `Area`, `Bars`, `Heatmap` (each as its own `PlotLayer`).
- Add stroke styles inspired by GPUI’s `StrokeStyle` (Linear / Step / Smooth), implemented via portable path commands.
- Add optional cursor linking (multi-plot sync) with explicit state sharing.

### Plot3D P0/P1

- Standardize `Plot3dCanvas` input mapping and document the coordinate conventions (logical px vs physical px).
- Add a minimal camera controller in the demo driver (orbit/pan/zoom) to validate the input forwarding pipeline.

## References

- `egui_plot`: https://github.com/emilk/egui_plot
- ImPlot: https://github.com/epezent/implot
- ImPlot3D: https://github.com/brenocq/implot3d
- GPUI component plot/chart substrate (local checkout): `repo-ref/gpui-component/crates/ui/src/plot/mod.rs`
- Plot crate placement: `docs/adr/0096-plot-widgets-and-crate-placement.md`
- Plot3D strategy: `docs/adr/0097-plot3d-rendering-strategy.md`
- Vector path contract: `docs/adr/0080-vector-path-contract.md`
- Viewport surfaces: `docs/adr/0007-viewport-surfaces.md`
- Viewport input forwarding: `docs/adr/0025-viewport-input-forwarding.md`
- Engine render hook: `docs/adr/0038-engine-render-hook-and-submission-coordinator.md`
