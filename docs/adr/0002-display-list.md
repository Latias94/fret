# ADR 0002: Display List (Scene) Contract


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Zed: https://github.com/zed-industries/zed

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
Status: Accepted

## Context

We need a renderer-facing representation that:

- is backend-agnostic (`wgpu` now, WebGPU/wasm later),
- supports multi-window rendering (per-window surface),
- is friendly to instanced GPU rendering (Zed/GPUI style),
- keeps UI core independent from `wgpu` types.

## Decision

Define a minimal display list called `Scene` in `fret-core`, consisting of ordered `SceneOp` primitives.

### Ordering semantics

`Scene.ops` order is **authoritative** for compositing. The renderer must preserve operation order
across primitive kinds (quads, viewports, text, images).

Batching and instancing are allowed only when they do not change visible ordering (typically by
batching adjacent operations). See `docs/adr/0009-renderer-ordering-and-batching.md`.

### Coordinate system

- All `Rect/Point/Size` in `SceneOp` are **logical pixels** in a top-left origin coordinate space.
- The platform provides `scale_factor` when presenting; the renderer converts to physical pixels.

### Color space

- `Color` is treated as **linear** RGBA at the API boundary.
- The renderer uses **premultiplied alpha** for blending.

### Clipping

- `PushClipRect/PopClip` define a rectangular clip stack.
- `PushClipRRect/PopClip` extend the stack with rounded clipping (ADR 0063).
- Rect clips may map to scissor rectangles (fast); rounded clips require soft/AA clip behavior.

Transform interaction:

- Clip geometry is expressed in the current local coordinate space (after transforms).
- Correct clip+transform composition semantics are locked in ADR 0078.

### Primitives (current surface area)

The scene contract has grown beyond the initial “Quad/Image/Text/Viewport” set as higher-level UI
needs became concrete (icons, vector paths).

Current `SceneOp` primitives include:

- `Quad` (rounded corners; borders optional, ADR 0030)
- `Image` / `ImageRegion`
- `MaskImage` (alpha mask + tint)
- `SvgMaskIcon` / `SvgImage` (ADR 0065)
- `Text`
- `Path` (prepared vector path; see `fret-core::vector_path`)
- `ViewportSurface` (embed engine render targets)

## Consequences

- UI core can build `Scene` without knowing GPU details.
- Renderer can batch aggressively (instancing) and evolve without breaking UI code.

## Future Work

- Define text shaping ownership (what lives in `fret-core` vs `fret-render`).
- Formalize the vector path contract (fill/stroke semantics, AA expectations, caching keys, and
  transform interaction) now that `PathService` + `SceneOp::Path` exist:
  - `docs/adr/0080-vector-path-contract.md`

## Notes (Zed/GPUI reference, non-normative)

- GPUI’s internal `Scene` records ordered paint operations and then derives an order key used for
  batching without breaking visible ordering (including replay/range reuse):
  `repo-ref/zed/crates/gpui/src/scene.rs` (`Scene`, `PaintOperation`, `Scene::finish`, `Scene::replay`).
