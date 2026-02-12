# ADR 0144: Canvas Pan/Zoom Input Mapping v1

- Status: Proposed
- Date: 2026-01-13
- Related:
  - ADR 0141 (Declarative Canvas element + hosted painter)
  - ADR 0074 (Action hook host pattern)
  - ADR 0051 / ADR 0066 (Model observation + invalidation)

## Context

Fret is a general-purpose, cross-platform UI framework. We want to enable interactive "infinite
canvas" experiences (node graphs, charts, vector editors, ports/3D overlays) without baking in
domain semantics into the runtime.

The repository already has:

- A declarative `Canvas` leaf element that can emit scene ops (ADR 0141).
- A hosted `CanvasPainter` that caches and manages renderer-owned resources (`TextBlobId`,
  `PathId`, `SvgId`) and provides scoped stack helpers.
- A policy-light `PanZoom2D` view model in `ecosystem/fret-canvas` that matches a common retained
  canvas coordinate convention.

What is missing is a *standard input mapping contract* for pan/zoom on desktop and a clear
boundary for where that policy should live.

## Problem

If the framework hardcodes a single input mapping (e.g. wheel always zooms), it will conflict with
embedded scroll views and general UI expectations. If every crate invents its own mapping, the
ecosystem will drift and users will see inconsistent behavior across widgets.

We need:

1. A clear boundary: runtime mechanisms vs component-layer policy.
2. A default desktop mapping that is safe for embedding into scrollable UIs.
3. A way to offer alternative mappings for editor/CAD-style canvases without compromising (2).

## Decision

### 1) Policy lives in `ecosystem/fret-canvas` (feature-gated), not in `fret-ui`

- `fret-ui` provides the `Canvas` leaf element and low-level pointer hooks (`PointerRegion`).
- `fret-canvas` provides reusable *canvas-specific policy* recipes behind an opt-in feature (e.g.
  `fret-canvas/ui`) so users can depend on a single canvas crate without pulling the full UI kit.
- A minimal `CanvasSurface` helper exists as wiring substrate (`PointerRegion + Canvas`), but it
  remains policy-free.

### 2) Default desktop mapping is "safe by default" (document-style)

We define a baseline mapping named `DefaultSafe`:

- `Wheel` without zoom modifier:
  - **Do not consume** the event.
  - Rationale: allow parent scroll containers and platform expectations to work.
- `Ctrl+Wheel` (Windows/Linux) or `Cmd+Wheel` (macOS):
  - Zoom about the pointer position and **consume** the event.
  - In code, treat `ctrl || meta` as the zoom modifier because the runtime does not currently
    expose platform in pointer hooks.
- `Middle-drag`:
  - Pan (translate) the canvas, capturing the pointer while dragging.

This mapping intentionally avoids requiring keyboard state tracking during pointer hooks (e.g.
Space-to-pan) until we introduce a first-class "gesture / chord" substrate.

### 3) Provide an explicit alternative mapping for editor/CAD canvases

We define a second preset named `DesktopCanvasCad`:

- `Wheel` zooms about the pointer (consumed).
- Panning remains `Middle-drag` by default (or `Right-drag` in app-specific variants).

Apps choose this preset explicitly for node editors, CAD, and other "canvas-first" surfaces.

## Mechanics and invariants

### View model

The recipe uses a `Model<PanZoom2D>`:

- `pan` is canvas-space translation applied after scaling.
- `zoom` is a uniform scale factor and must remain finite and > 0.

### Coordinate mapping

`PanZoom2D` defines mapping conventions:

- `T(bounds.origin) * S(zoom) * T(pan)` maps canvas space to window space.
- `screen_to_canvas` and `canvas_to_screen` are defined with respect to `bounds`.

### Raster scale factor

Hosted resources that depend on DPI/zoom (text shaping, path tessellation) should use:

`raster_scale_factor = painter.scale_factor() * zoom`

This keeps output stable across DPI changes while allowing policy-controlled zoom.

### Wheel zoom curve

For compatibility with existing ecosystem widgets, wheel zoom uses a deterministic exponential
curve:

- Base: `1.18`
- Step: `120.0` (typical wheel delta unit on desktop)
- Factor: `base.powf(-delta_y / step * speed)`

## Future work (non-goals of v1)

- Touchpad pinch/magnify nuances:
  - `PointerEvent::PinchGesture` is supported, but platform backends differ in delta units and
    update frequency; we may need additional policy knobs once real apps adopt the recipe.
- Touch-first gesture semantics:
  - single-finger drag is tool-specific (selection vs pan) and should be recipe-configurable.
- Space-to-pan and other chords that require key state during pointer hooks:
  - likely needs a dedicated gesture/chord substrate (or a focus-scoped key state provider).

## Consequences

- Canvas pan/zoom becomes consistent across crates that opt into the recipe surface.
- The default mapping does not break embedding inside scroll containers.
- Editor/CAD crates can choose the more aggressive mapping explicitly without compromising the
  framework default.
