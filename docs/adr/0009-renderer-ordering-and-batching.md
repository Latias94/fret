# ADR 0009: Scene Ordering and Renderer Batching

Status: Accepted

## Context

Fret targets editor-grade UI composition:

- docking layouts with nested panels,
- engine viewports with UI overlays (gizmos, selection, drag previews),
- popups/menus/drag hints that must layer correctly,
- multi-window tear-off, where each window is composited independently.

In this environment, **draw order correctness** is a hard requirement. Any renderer optimization
that reorders draw operations can break expected layering (e.g. viewport overlays).

At the same time, we still want to batch for performance (instancing, atlas usage).

References:

- Fret display list contract (ordered ops):
  - `docs/adr/0002-display-list.md`
- Zed/GPUI quad SDF + batching style (illustrative renderer design):
  - `repo-ref/zed/crates/gpui/src/platform/blade/shaders.wgsl`

## Decision

### 1) Scene operation order is authoritative

`fret-core::Scene` is an **ordered** list of `SceneOp`. The renderer must interpret the list as
the authoritative compositing order.

The renderer **must not reorder** operations across different primitive kinds (e.g. draw all
`ViewportSurface` first and all `Quad` later).

### 2) Batching is allowed only when order is preserved

The renderer may batch **adjacent** operations when doing so does not change visible results.

Examples of allowed batching:

- consecutive `Quad` ops combined into one instanced draw, as long as they remain in-order,
- consecutive `Text` ops combined, as long as state changes are applied in-order,
- scissor/clip changes cause batch boundaries.

Examples of disallowed batching:

- collecting all quads into a single list and drawing them after viewports,
- sorting by a secondary key (e.g. `DrawOrder`) across non-adjacent ops,
- any cross-op reordering that changes the relative order between primitives.

### 3) Clip stack participates in ordering

`PushClipRect/PopClip` define a clip stack that affects subsequent operations.
Batch boundaries must be inserted when the effective clip changes.

Initial implementations may map clip rects to scissor rectangles; future implementations may
add soft clip, but the ordering semantics remain the same.

## Consequences

- Correct overlay composition (viewport + UI overlay, popups, drag hints) is guaranteed by the core contract.
- Renderer performance remains viable via adjacency-preserving batching and instancing.
- `DrawOrder` is not a general-purpose sorting key; it may be used only within a single op kind
  or as an internal tie-breaker that does not violate operation order.

## Future Work

- Formalize how `DrawOrder` should evolve (keep as internal/debug, or remove from the public contract).
- Define additional “stateful” ops (transform, opacity groups, layers) while preserving order semantics.
- Add a renderer test harness that verifies ordering for mixed primitives (quad + viewport + text).
