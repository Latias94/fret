# `fret-canvas`

Policy-light canvas substrate helpers for Fret ecosystem widgets.

This crate exists to reduce duplication across "canvas-like" retained widgets (node graphs, plots,
charts, editors) without pushing interaction policy into `crates/fret-ui`.

Design reference: `docs/adr/0137-canvas-widgets-and-interactive-surfaces.md`.

## Goals

- Provide reusable building blocks for canvas widgets:
  - pan/zoom view transforms and coordinate mapping,
  - pixel/scale policies (constant-pixel strokes, effective scale factors),
  - drag phase/value helpers (begin/update/commit/cancel, thresholds),
  - small, reusable caches (e.g. prepared text blobs).
- Keep this crate portable and mechanism-oriented:
  - no platform/backend deps,
  - no domain models,
  - no gesture maps / tool rules / snapping policies.

## Unit conventions

Fret uses **window-local logical pixels** as its “screen px” unit (similar to CSS px).
In code, this is typically represented as `fret_core::Px` (or plain `f32` values documented as
screen-space pixels).

When a canvas widget uses `Widget::render_transform` for pan/zoom (ADR 0083), pointer event
positions are delivered in the widget's **local (untransformed) coordinate space**. In that case,
any UX tuning expressed in screen pixels (hit slop, click distance, drag threshold, handle radius)
should be converted to canvas units before comparison:

- `fret_canvas::scale::canvas_units_from_screen_px(screen_px, zoom)` (typically `screen_px / zoom`)

## Modules

- `fret_canvas::view`
  - `PanZoom2D`: a small pan/zoom view model that can generate a `Transform2D` compatible with
    `Widget::render_transform` and provides `screen_to_canvas`/`canvas_to_screen` mapping.
- `fret_canvas::scale`
  - `effective_scale_factor`: DPI scale factor multiplied by view zoom (for resource preparation).
  - `constant_pixel_stroke_width`: helper for constant-screen-pixel strokes under zoom.
- `fret_canvas::drag`
  - `DragPhase`: minimal drag lifecycle vocabulary.
  - `DragThreshold`: screen-px threshold converted to canvas units under zoom.
- `fret_canvas::text`
  - `TextCache`: a keyed cache for prepared `TextBlobId` + `TextMetrics` that releases resources
    via `UiServices`.
- `fret_canvas::cache`
  - `PathCache`: a keyed cache for prepared `PathId` + `PathMetrics` that releases resources via
    `UiServices`.
  - `SvgCache`: a keyed cache for registered `SvgId`s that unregisters SVGs via `UiServices`.

## Replay caches and resource liveness

`SceneOpCache` / `SceneOpTileCache` intentionally store only recorded `SceneOp`s and do **not**
manage the lifetimes of renderer-owned resources referenced by ops (`TextBlobId`, `PathId`, `SvgId`).

When you replay cached ops on a cache hit, make sure the corresponding retained caches are also
"touched" so entries are not pruned while still referenced by replay caches:

- `TextCache::touch_blobs_in_scene_ops(&ops)`
- `PathCache::touch_paths_in_scene_ops(&ops)`
- `SvgCache::touch_svgs_in_scene_ops(&ops)`

To reduce foot-guns, prefer the on-hit hook variants when replaying cached ops:

- `SceneOpCache::try_replay_with(..., |ops| { ...touch... })`
- `SceneOpTileCache::try_replay_with(..., |ops| { ...touch... })`
- `warm_scene_op_tiles_u64_with(..., |ops| { ...touch... }, ...)`

If you have multiple retained caches, you can aggregate touch behavior into one call:

- `(&mut text_cache, &mut path_cache, &mut svg_cache).touch_hosted_resources_in_scene_ops(&ops)`

## Future: declarative surface

Fret now provides a declarative canvas element in `crates/fret-ui` (ADR 0156).

- Declarative: use `fret_ui::ElementContext::canvas(...)` + `fret_ui::canvas::CanvasPainter` to
  emit `SceneOp`s with hosted resources (`TextBlobId`/`PathId`/`SvgId`) and scoped helpers for
  clip/transform/effect stacks.
- Retained: keep using `ecosystem/fret-canvas` for retained widgets (node graphs, plots, editors)
  that own their own interaction policy and internal caches.

This crate intentionally stays focused on reusable retained-canvas helpers that remain useful even
when the high-level authoring style is declarative.

## Optional: UI integration (`fret-canvas/ui`)

For convenience, `fret-canvas` can also provide canvas-specific declarative wiring and recipes
behind an opt-in feature:

- Enable: `fret-canvas/ui`
- Provides: `fret_canvas::ui::{canvas_surface_panel, pan_zoom_canvas_surface_panel, canvas_tool_router_panel, ...}`
- Includes a convenience preset: `fret_canvas::ui::editor_pan_zoom_canvas_surface_panel` (wheel pan + Ctrl/Cmd+wheel zoom).

This keeps the default crate portable and lightweight while allowing users to depend on a single
canvas crate when building editor-grade interactive surfaces.

## Optional: rstar backend

`fret-canvas` ships with a uniform-grid spatial index by default.

For experiments and large graphs with highly non-uniform distributions, an opt-in R-tree backend is
available via the `rstar` feature:

- Enable: `fret-canvas/rstar`
- Call sites should prefer `fret_canvas::spatial::DefaultIndexWithBackrefs<T>` so the backend can
  be swapped without refactors.

## Optional: kurbo backend

For robust 2D geometry operations (Bezier refinement, path hit-testing, intersection helpers),
`fret-canvas` can optionally use `kurbo` (ADR 0167).

- Enable: `fret-canvas/kurbo`
- Current scope: Bezier wire refinement (`bezier_wire_distance2`, `closest_point_on_bezier_wire`).
