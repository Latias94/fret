# ADR 1177: Pointer Coordinate Spaces and Element-Local Mapping (v1)

Status: Proposed

## Context

MagicUI-style components and editor-grade affordances frequently depend on *pointer-follow* effects:

- lens / parallax cards,
- hover highlights that track the cursor,
- drag handles and dock previews,
- "magnetic" buttons and ripple origins.

Today, portable pointer events are expressed as a single window-local logical position
(`PointerEvent::{Move,Down,Up,Wheel}.position: Point`). This is a good lowest-common-denominator
surface, but it pushes every component to repeat the same coordinate math:

- `event.position - bounds.origin` to compute element-local coordinates,
- ad-hoc handling for `RenderTransform` (ADR 0083),
- inconsistent treatment of pointer capture and overlays.

This becomes a “death by a thousand cuts” problem: every new interactive component re-derives the
same semantics, and small inconsistencies accumulate into UX bugs.

We want a mechanism-level, portable contract that:

- keeps `fret-core` events window-local and runner-owned,
- defines the canonical coordinate spaces used by `fret-ui` widgets,
- provides a stable, transform-aware element-local coordinate mapping,
- stays compatible with hit-testing, overlays, and pointer capture.

Related ADRs:

- Render transforms + event coordinates: `docs/adr/0083-render-transform-hit-testing.md`
- Viewport input forwarding: `docs/adr/0025-viewport-input-forwarding.md`
- Window metrics (scale factor): `docs/adr/0017-logical-and-physical-pixels.md`
- Overlays and multi-root: `docs/adr/0011-overlays-and-multi-root.md`

## Decision

### 1) Define coordinate spaces (normative)

For pointer-driven UI policies, the runtime recognizes the following spaces:

- **Window logical** (`window_logical`): window-local coordinates in logical pixels.
  - This is the coordinate space used by `fret-core::PointerEvent.position`.
- **Window physical** (`window_physical`): window-local coordinates in physical pixels.
  - Derived as `window_logical * window_scale_factor`.
- **Element local logical** (`local_logical`): element-local coordinates in logical pixels, with the
  element's layout bounds origin as `(0, 0)`.
  - Derived as `local_logical = position_in_element_layout_space - element_bounds.origin`.
- **Element local physical** (`local_physical`): element-local coordinates in physical pixels.
  - Derived as `local_logical * window_scale_factor`.

The authoritative scale factor is the window scale factor provided by the runner and recorded into
window metrics (ADR 0017).

### 2) Widget-facing semantics for `PointerEvent.position`

When `fret-ui` dispatches an event to a widget:

- For pointer-position-bearing events, the `position` field MUST be expressed in that widget's
  **untransformed layout space** as defined by ADR 0083.
  - This ensures that subtracting `EventCx.bounds.origin` yields a stable `local_logical`
    coordinate, even under `RenderTransform`.
- For `PointerEvent::Wheel.delta`, the delta MUST be mapped as a vector through the same inverse
  transform traversal used for `position` (ADR 0083).

### 3) Element-local mapping helper (mechanism, not policy)

`fret-ui` SHOULD provide a small, stable helper surface so that component code does not repeat
boilerplate:

- `EventCx::pointer_position_local(event) -> Option<Point>`
- `EventCx::pointer_delta_local(event) -> Option<Point>` (wheel)
- `EventCx::pointer_position_window(event) -> Option<Point>` (for the rare cases that need it)

These helpers are purely derived; they do not introduce new state.

### 4) Pointer capture semantics

When a pointer is captured by a widget:

- Subsequent pointer events for that pointer MUST still be mapped into the **capturing widget's**
  untransformed layout space (ADR 0083), even when the pointer is outside the widget's bounds.
- The runtime MUST NOT clamp the mapped coordinate to the widget bounds. Components (sliders, drag
  handles) frequently rely on out-of-bounds coordinates for elastic behavior.

This rule applies equally for overlay roots (ADR 0011): capture determines the coordinate space,
not hit-testing.

### 5) Viewport forwarding is explicitly separate

Forwarding pointer input into embedded engine viewports remains governed by ADR 0025:

- Viewport input events MAY carry additional geometry/mapping fields (viewport-local coordinates,
  clip rects, transforms) that are not required for UI widgets.
- `fret-ui` MUST NOT overload widget pointer coordinate semantics to satisfy embedded viewport
  needs.

## Consequences

- Pointer-follow components become consistent and less error-prone: element-local coordinates are
  well-defined and transform-aware by contract.
- Capture behavior is easier to reason about: the coordinate space follows the capture target.
- The framework avoids prematurely exposing a "full event-space algebra" in `fret-core`; derived
  mapping lives in `fret-ui` where it belongs.

## Non-goals

- This ADR does not add higher-level gesture recognition (drag/pan/fling) as a framework contract.
  Gesture policy remains ecosystem-owned.
- This ADR does not standardize "coalesced pointer move" sampling or velocity; those can be added
  later if needed for perf or smoothness.

