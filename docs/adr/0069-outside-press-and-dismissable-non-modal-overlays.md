---
title: "ADR 0069: Outside Press for Non-Modal Overlays (Dismissable, Click-Through)"
---

# ADR 0069: Outside Press for Non-Modal Overlays (Dismissable, Click-Through)

## Status

Accepted.

## Context

Shadcn/Radix-style overlays (popover, context menu, hover-card-like surfaces) commonly need:

- deterministic anchored placement (ADR 0064),
- dismissal on Escape,
- dismissal on "outside press" (pointer down outside the overlay),
- predictable focus restoration (only when appropriate).

In a DOM environment, outside press is typically implemented with Radix `DismissableLayer` and
event propagation control. In Fret, input dispatch is hit-test based, so an overlay does **not**
receive pointer events when the cursor is outside its subtree unless the runtime provides an
explicit observer mechanism.

We also want to avoid a combinatorial explosion of boolean toggles in the runtime contract surface.

## Decision

We classify overlays into two runtime categories:

1) **Modal overlays** (barrier-backed):
   - installed with `blocks_underlay_input = true`,
   - block underlay input and semantics,
   - enforce focus trapping (see ADR 0068),
   - may dismiss on outside press by handling hit-tested events in the barrier layer.

2) **Non-modal overlays** (dismissable, click-through):
   - installed with `blocks_underlay_input = false`,
   - do not block underlay input/semantics,
   - can dismiss on outside press via a runtime observer pass.

### Outside press observer pass

The runtime provides an opt-in per-layer flag:

- `wants_pointer_down_outside_events`

When a `PointerEvent::Down` occurs and there is no pointer capture, the runtime performs an
**outside press observer pass** before the normal hit-test dispatch:

- It finds the topmost visible non-modal layer (in paint order) that:
  - is in the active input stack (respecting modal barrier scoping),
  - has `wants_pointer_down_outside_events = true`,
  - and does **not** contain the hit-tested target.
- The runtime dispatches the same pointer-down event to that layer's root chain.
- The observer pass does **not** prevent the normal hit-tested dispatch: non-modal outside press is
  always **click-through**.
- The observer pass must be **side-effect free** for input routing:
  - it must not change pointer capture,
  - it must not override focus,
  - it must not block/bubble-stop the subsequent normal dispatch.

This is the minimal contract needed to express Radix-like dismissal behavior without adding a
matrix of per-component runtime toggles.

### Dismissable branches (Radix `DismissableLayerBranch`)

Some disjoint subtrees should be treated as “inside” an overlay for dismissal purposes even though
they are not descendants of the overlay root (e.g. a trigger button rendered in the underlay).

To support this Radix-aligned outcome, non-modal layers may provide a set of “branch” roots that
the runtime treats as inside for the outside-press observer pass:

- if the hit-tested target is inside any registered branch subtree, the observer pass does not
  dispatch an outside-press event for that overlay layer (and does not dismiss anything under it).

This preserves the click-through guarantee: the normal hit-tested dispatch still runs.

### Presence and close transitions (click-through correctness)

Non-modal overlays commonly animate out (opacity / scale / slide) while remaining mounted.

During this close transition window, the overlay may be:

- `present = true` (still painted), and
- `open = false` (no longer interactive; close has been requested).

To preserve the “outside press is click-through” guarantee:

- A non-modal overlay that is `present=true` but `open=false` must be **pointer-transparent**.
  - It must not become the hit-tested target.
  - It must not receive pointer-down-outside observer events (because it is no longer dismissable).

This ensures underlay widgets can be clicked immediately while a fading surface finishes its out
transition.

### Focus restoration

When a non-modal overlay is closed due to click-through outside press, focus may already move to a
different widget as part of the same interaction. Therefore:

- Overlay focus restoration must be conditional: restore previous focus **only if** focus is still
  inside the closing overlay (or is missing), otherwise do not override the new focus target.

This rule is implemented by `OverlayPortal::hide` and applies to all overlays.

## Consequences

- `Popover`, `PopoverSurface`, and `ContextMenu` are installed as **non-modal** overlays.
- Modal widgets (command palette, dialog, sheet) continue to use barrier-backed layers.
- Components can implement outside press dismissal purely in the component layer using the
  existing event + bounds logic, without requiring modal barriers.

## References

- Radix UI Primitives: <https://github.com/radix-ui/primitives> (pinned locally; see `docs/repo-ref.md`)
- Shadcn recipes: `repo-ref/ui`
- Overlay placement contract: `docs/adr/0064-overlay-placement-contract.md`
- Overlay policy architecture: `docs/adr/0067-overlay-policy-architecture-dismissal-focus-portal.md`
