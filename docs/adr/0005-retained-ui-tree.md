# ADR 0005: Retained UI Tree, Layout Bounds, and Input Routing

Status: Accepted

## Context

Fret targets editor-style UIs with deep trees, frequent state updates, and multi-window docking. To avoid later rewrites, we need a stable contract for:

- how widgets lay out and paint children,
- which coordinate space is used for hit-testing,
- how events are routed (capture vs hit-test vs focus),
- what “invalidation” means in a retained tree.

## Decision

### 1) Layout writes bounds (source of truth)

- Layout is responsible for assigning a **Rect** (`bounds`) to every node.
- Containers must position children via `LayoutCx::layout_in(child, rect)`.
- `UiTree` stores each node’s latest `bounds` during layout. Hit-testing relies on these bounds.

Implication:

- Paint should not be the place where child positions are decided.
- Containers may still choose to paint children in any order, but the geometry comes from layout.

### 2) Paint uses layout bounds

- Containers should paint children using stored bounds (e.g. via `PaintCx::child_bounds(child)`).
- This is required for components like `Scroll` where the child is laid out with a translated origin.

### 3) Event target selection and routing

Target selection:

1. If pointer capture is active, the captured node receives pointer events.
2. Otherwise, pointer events target the deepest node whose `bounds` contains the pointer (hit-test).
3. Otherwise, non-pointer events target the focused node, falling back to the root.

Propagation:

- Events bubble up to parents unless `stop_propagation()` is requested.
- Pointer capture can be requested and released via context APIs.

### 4) Invalidation

- Widgets request invalidation (`Layout`, `Paint`, `HitTest`) via context.
- Invalidation is propagated up the parent chain so containers can recompute layout/paint as needed.

## Consequences

- `Scroll`, clipping, and complex editor layouts can be implemented without ad-hoc coordinate hacks.
- Hit-testing remains consistent across frames (layout is authoritative).
- Platform layers only provide input events; UI core owns routing and capture semantics.

## Future Work

- Explicit z-order policy and hit-test override hooks.
- Keyboard focus traversal rules (tab-order, focus groups).
- Partial layout/paint caching and dirty-region optimizations.

