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

## Future: declarative surface

The `declarative` feature is reserved for a future declarative canvas element surface aligned with
ADR 0137. The current crate intentionally focuses on substrate helpers that are useful both for
retained widgets and for eventual declarative authoring.
