# ADR 0063: Rounded Clipping and Soft Clip Masks

Status: Accepted

## Context

Fret currently exposes a rectangular clip stack via `SceneOp::PushClipRect/PopClip` (ADR 0002).
The renderer maps this to scissor rectangles, which is fast and portable, but insufficient for
DOM/shadcn-style composition where “rounded corners + overflow-hidden” is a baseline expectation.

Concrete pain points:

- `overflow-hidden` on rounded surfaces (Popover/HoverCard/ScrollArea) requires **rounded** clipping
  to avoid content bleeding through corners.
- Menu/list row highlights and scroll contents frequently rely on “clip to radius” to avoid visual
  seams.
- Component-layer workarounds (extra masks, overdraw tricks) leak policy into components and create
  drift across the ecosystem.

We want a renderer-friendly, ordering-preserving clip contract that:

- works for nested clips,
- provides smooth edges (AA),
- keeps the scene contract minimal and backend-agnostic.

## Decision

Extend the `SceneOp` contract with a rounded-rect clip operation:

- Add `SceneOp::PushClipRRect { rect, corner_radii }`
- Keep `SceneOp::PushClipRect { rect }` as the “fast path” (rectangular scissor clip)
- `SceneOp::PopClip` pops one level (shared for both clip kinds)

This enables `overflow-hidden` semantics for rounded containers without encoding component policy
into runtime widgets.

## Semantics

### Ordering

Clip operations participate in ordering exactly like `PushClipRect` does today:

- The effective clip stack applies to all subsequent operations until popped.
- Batch boundaries must be inserted when the effective clip changes (ADR 0009).

### Geometry

- `rect` is in logical pixels (top-left origin), same coordinate space as other `SceneOp` rects.
- `corner_radii` is per-corner in logical pixels (TL/TR/BR/BL), clamped by the renderer to the
  rect’s dimensions (same rule as quads, ADR 0030).

### Edge quality (AA)

Unlike scissor clipping, rounded clip boundaries must be smooth:

- The renderer should treat `PushClipRRect` as a **soft clip** (coverage-based) rather than a hard
  binary cut.
- Implementations may use shader-based SDF clipping, MSAA/stencil, or a mask pass, as long as the
  visible results are stable and ordering semantics are preserved.

### Hit-testing consistency

When the UI runtime uses rounded clipping for `Overflow::Clip`, hit-testing must match paint
clipping:

- If a parent clips hit-testing, descendants outside the rounded clip are not hit-testable.
- This is the “overflow-hidden” mental model required by shadcn-style interactive surfaces.

## Consequences

- `fret-ui` can express “rounded overflow-hidden” composition without bespoke widgets.
- `fret-render` must implement a clip stack that includes rounded clips (scissor alone is no longer
  sufficient).
- This becomes a P0 substrate for consistent component ecosystem behavior (Popover/ScrollArea etc.).

## Future Work

- General clip paths (`PushClipPath`) and/or texture masks can be added later without invalidating
  this contract.
- A fully general clip/mask system may be required for complex vector shapes; rounded-rect clipping
  is the minimal shadcn-aligned substrate.

