# Carousel × DnD Gesture Arbitration (CAR-430)

Status: Draft (decision + implementation notes)

Goal: define how a scroll-snap container (Carousel) should coexist with `fret-dnd` pointer sensors
without breaking either "swipe to scroll" or "drag to reorder/tear-out".

Scope:

- Policy only (ecosystem). Do not change `crates/fret-ui` mechanisms for this work item.
- Applies to any component that uses a "threshold then steal capture" drag model (Carousel) and a
  headless DnD sensor activation model (`fret-dnd`).

## Problem statement

Carousel expects:

- Pointer down reaches interactive descendants (pressables/buttons).
- If movement exceeds `drag_threshold_px`, Carousel steals capture and converts the gesture into a
  scroll drag.

DnD expects:

- Pointer down starts tracking a sensor (pending).
- After the activation constraint is satisfied (distance and/or delay), DnD becomes active and
  begins a drag session.

If both are armed on the same pointer stream, naive implementations can lead to:

- accidental DnD when the user meant to swipe Carousel,
- Carousel stealing capture too early, starving DnD activation,
- inconsistent behavior across mouse vs touch.

## Constraints we can rely on

- `fret-dnd` provides activation constraints:
  - `ActivationConstraint::Distance { px }`
  - `ActivationConstraint::DelayTicks { ticks }`
  - `ActivationConstraint::DelayAndDistance { ticks, px }`
  (`ecosystem/fret-dnd/src/activation.rs`)
- Fret supports deterministic capture switching, including canceling the previous capture target
  when a parent steals capture after a threshold (see `docs/audits/carousel-shadcn-embla-parity.md`).

## Proposed policy (recommended default)

1) Mouse / trackpad:

- Prefer **handle-only DnD** for items inside Carousel.
- Carousel swipe can start from anywhere else in the item content area.

Rationale: the user intent signal is explicit and avoids ambiguous gesture competition.

2) Touch:

- Prefer **long-press DnD** (delay + small distance) for items inside Carousel.
- Keep Carousel swipe responsive on short drags.

Rationale: on touch, "drag to scroll" is the dominant gesture; long-press is the common DnD intent
signal.

3) Keyboard:

- Out of scope for v1 (DnD keyboard sensors are explicitly non-goals in ADR 0157).

## Concrete activation constraints (suggested)

These are not hard contracts; they are defaults for recipes that need both behaviors.

- Carousel drag: keep existing `drag_threshold_px` default (`10px` in Embla).
- DnD inside Carousel:
  - Mouse handle: `ActivationConstraint::Distance { px: 2.0 }` (or `None` if the handle is explicit)
  - Touch long-press: `ActivationConstraint::DelayAndDistance { ticks: 12, px: 6.0 }`
    - `ticks=12` assumes ~60fps (≈ 200ms). Tune based on user feedback.

## Implementation plan (ecosystem-only)

- Recipes that combine Carousel + DnD should:
  - route pointer events from the **handle** (mouse) or **long-press** region (touch) to
    `fret-ui-kit::dnd` sensor handlers with the chosen `ActivationConstraint`.
  - keep Carousel pointer region unchanged.
- When DnD becomes active, it should steal capture and cancel any pending descendant pressable /
  Carousel drag state (mechanism already supports cancel on capture switch).

## Regression gates (recommended)

- Add a small demo surface (UI gallery or workspace tab strip) with a draggable handle inside a
  Carousel item:
  - swipe on item body scrolls the Carousel
  - drag on handle starts DnD (no Carousel movement)
- Add a diag script that performs:
  - drag-from-body (assert Carousel offset changed)
  - drag-from-handle (assert DnD session active, Carousel offset unchanged)

