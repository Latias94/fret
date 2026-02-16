# ADR 0019: Scene State Stack (Transforms, Opacity, Layers)


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Zed: https://github.com/zed-industries/zed

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
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

### 1.1) Coordinate space (clarification)

All geometry carried by `SceneOp` remains in **logical pixels**, but it is interpreted in the **current local
coordinate space** as affected by the state stack.

In other words:

- `PushTransform` changes the coordinate system for subsequent ops.
- `PushClip*` defines clip geometry in the coordinate system in effect at the time of the push.
- The renderer applies `scale_factor` when converting logical pixels to physical pixels (ADR 0002).

For the locked v1 semantics of transform + clip composition, see ADR 0078.

### 2) Ordering remains authoritative

All state stack operations participate in the same ordering semantics:

- `Scene.ops` order is authoritative.
- Renderers may batch only when the effective state and order are preserved.

### 3) Multi-root overlays map naturally to layers

Multi-root composition (ADR 0011) may be implemented by building the final `Scene` as:

- base roots paint first,
- overlay/popup/modal roots paint later,

optionally encoded as explicit layer ops in the display list for clarity and debugging.

For the locked v1 semantics of `PushLayer / PopLayer`, see ADR 0079.

### 4) Stack invariants (debuggability)

Scene producers must maintain balanced stacks:

- every `PushTransform` must have a matching `PopTransform`,
- every `PushOpacity` must have a matching `PopOpacity`,
- every `PushLayer` must have a matching `PopLayer`,
- every `PushClip*` must have a matching `PopClip`.

Implementations are encouraged to validate these invariants in debug builds and during tests to avoid
“silent no-op” behavior in renderers.

## Consequences

- Transforms, opacity, and layers can be added without rewriting the core display list contract.
- Clipping can evolve from scissor rectangles to shader-based clipping without changing UI semantics.
- Renderer complexity stays manageable because state changes define natural batch boundaries.

## Future Work

- Lock transform + clip composition semantics (ADR 0078).
- Decide whether “layer” is an explicit batching contract or a debug-only marker (and whether it implies
  offscreen composition).
- Define how state interacts with text shaping caches and atlas coordinates.
