# ADR 0002: Display List (Scene) Contract

Status: Accepted

## Context

We need a renderer-facing representation that:

- is backend-agnostic (`wgpu` now, WebGPU/wasm later),
- supports multi-window rendering (per-window surface),
- is friendly to instanced GPU rendering (Zed/GPUI style),
- keeps UI core independent from `wgpu` types.

## Decision

Define a minimal display list called `Scene` in `fret-core`, consisting of ordered `SceneOp` primitives.

### Coordinate system

- All `Rect/Point/Size` in `SceneOp` are **logical pixels** in a top-left origin coordinate space.
- The platform provides `scale_factor` when presenting; the renderer converts to physical pixels.

### Color space

- `Color` is treated as **linear** RGBA at the API boundary.
- The renderer uses **premultiplied alpha** for blending.

### Clipping

- `PushClipRect/PopClip` define a rectangular clip stack.
- Initial implementation may use scissor rectangles; future shaders can implement soft clip if needed.

### Minimal primitives (current direction)

- `Quad` (rounded corners via SDF; borders optional)
- `Image` (atlas-based)
- `Text` (shaped runs + glyph atlas)
- `ViewportSurface` (embed engine render targets)

## Consequences

- UI core can build `Scene` without knowing GPU details.
- Renderer can batch aggressively (instancing) and evolve without breaking UI code.

## Future Work

- Define text shaping ownership (what lives in `fret-core` vs `fret-render`).
- Decide whether to add vector paths (triangulation vs texture atlas vs GPU raster).

