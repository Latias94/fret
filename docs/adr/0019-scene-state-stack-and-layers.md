# ADR 0019: Scene State Stack (Transforms, Opacity, Layers)

Status: Accepted

## Context

Editor UIs require composition features beyond “draw quads in a list”:

- transforms (scrolling, zoomable canvases, gizmos),
- opacity groups (disabled UI, fade animations),
- layers (overlays, popups, drag previews),
- clipping that composes with transforms (and later soft clip / rounded clip),
- future effects like drop shadows and blur.

If these features are added ad-hoc, the display list (`Scene`) tends to be rewritten repeatedly.

References:

- Ordered display list semantics:
  - `docs/adr/0002-display-list.md`
  - `docs/adr/0009-renderer-ordering-and-batching.md`
- Multi-root overlay requirement:
  - `docs/adr/0011-overlays-and-multi-root.md`
- GPUI shader approach (illustrative: clip + transforms + quad SDF):
  - `repo-ref/zed/crates/gpui/src/platform/blade/shaders.wgsl`

## Decision

### 1) Scene is extended via a state stack model

Fret’s `Scene` remains an ordered list of operations, but it evolves to support an explicit **state stack**.

Conceptually, the following stateful operations are reserved:

- `PushTransform / PopTransform`
- `PushOpacity / PopOpacity` (or `PushOpacityGroup`)
- `PushLayer / PopLayer` (optional; can be modeled as state + ordering)
- `PushClipRect / PopClip` (already present)

State operations affect all subsequent draw ops until popped.

### 2) Ordering remains authoritative

All state stack operations participate in the same ordering semantics:

- `Scene.ops` order is authoritative.
- Renderers may batch only when the effective state and order are preserved.

### 3) Multi-root overlays map naturally to layers

Multi-root composition (ADR 0011) may be implemented by building the final `Scene` as:

- base roots paint first,
- overlay/popup/modal roots paint later,

optionally encoded as explicit layer ops in the display list for clarity and debugging.

## Consequences

- Transforms, opacity, and layers can be added without rewriting the core display list contract.
- Clipping can evolve from scissor rectangles to shader-based clipping without changing UI semantics.
- Renderer complexity stays manageable because state changes define natural batch boundaries.

## Future Work

- Define the exact state representation (mat3/mat4 for transforms, premultiplied opacity rules).
- Decide whether “layer” is an explicit op or derived from root ordering only.
- Define how state interacts with text shaping caches and atlas coordinates.

