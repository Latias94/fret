---
title: "ADR 0129: Tooltip Scroll Dismissal (Radix-aligned)"
---

# ADR 0129: Tooltip Scroll Dismissal (Radix-aligned)

## Status

Accepted.

## Context

Radix Tooltip closes when its trigger is scrolled.

Upstream implementation (`@radix-ui/react-tooltip`) installs a window scroll listener and closes
the tooltip when the scroll event target contains the trigger element:

- `repo-ref/primitives/packages/react/tooltip/src/tooltip.tsx`

This matters in native-style UIs where a tooltip trigger can live inside scrollable containers:

- Without scroll dismissal, the tooltip can "stick" on screen while its trigger has moved away.
- The desired behavior is not "close on any scroll" globally, but "close when the trigger was
  inside the scroll target that actually scrolled" (Radix semantics).

Zed/GPUI has a similar UX outcome (tooltips are cleared on scroll wheel), but its tooltip system is
mouse-position based rather than anchored popper-style overlays. See:

- `repo-ref/zed/crates/gpui/src/elements/div.rs` (`register_tooltip_mouse_handlers`)

Fret's tooltip implementation is overlay-based (anchored placement), so the framework needs an
overlay-layer-level contract to express the Radix outcome precisely.

## Decision

We model "close tooltip on scroll" as an opt-in **overlay-layer dismissal contract**:

1) An overlay layer may register one or more **scroll-dismiss elements** (`GlobalElementId`s).
2) When a wheel/scroll gesture is consumed by a scrollable element, the runtime identifies the
   **scroll target node** (the scrollable node that stopped propagation for the wheel event).
3) Any visible overlay layer that has at least one registered scroll-dismiss element whose current
   node is a descendant of the scroll target node is dismissed.

This matches the Radix semantic check:

- Radix: `event.target.contains(trigger)`
- Fret: `scroll_target_node.is_ancestor_of(trigger_node)`

### Runtime surface

The UI runtime exposes a per-layer property:

- `UiTree::set_layer_scroll_dismiss_elements(layer, Vec<GlobalElementId>)`

Overlay policy code sets this for tooltip layers by registering the tooltip trigger's `GlobalElementId`.

### Dismiss hooks

Dismissal uses the existing dismissible-root contract (ADR 0069 / ADR 0067):

- overlay roots are rendered via `render_dismissible_root_with_hooks(...)`
- component/policy code installs an `OnDismissRequest` handler on that dismissible root
- scroll dismissal invokes the handler with `DismissReason::Scroll`

## Consequences

- Tooltip scroll dismissal is precise: unrelated scroll containers do not close tooltips.
- The contract is overlay-generic and can be reused by other overlays if needed, but is opt-in.
- The initial implementation targets `PointerEvent::Wheel` consumption. Future work may extend this
  to other scroll sources (e.g. scrollbar drags, programmatic scroll) if required for strict parity.
