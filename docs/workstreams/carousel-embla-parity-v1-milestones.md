# Carousel Embla parity (v1) — Milestones

## Overview

Milestones are grouped by contract risk and regression value. Each milestone should be landable independently.
Prefer “mechanism first, then shadcn recipe”, with a gate at the end of each phase.

## M0 — Baseline parity audit (done)

**Exit criteria**

- Upstream references captured (Embla defaults and drag handler semantics).
- In-tree anchors identified for carousel implementation, UI gallery page, and existing gates.

## M1 — Pointer arbitration contract closure (mechanism)

**Goal**

Support Embla-like “armed → threshold → steal capture → cancel loser” without breaking descendant interactions.

**Exit criteria**

- Pointer regions can opt in to capture-phase pointer moves.
- Capture switching dispatches `PointerCancel` to the previous capture target.
- A focused test exists proving cancel-on-capture-switch.

## M2 — Carousel “drag from interactive descendants” parity (shadcn)

**Goal**

Dragging starting on a button inside a slide behaves like Embla/shadcn:

- If the gesture becomes a drag, the button does not activate.
- If the gesture is a click (no threshold crossing), the button activates.

**Exit criteria**

- Rust test exists and is green:
  - pointer down on inner pressable → move > threshold → up ⇒ no activation
  - pointer down + up without crossing threshold ⇒ activation

## M3 — UI gallery + diag script gate (reproducible interaction)

**Goal**

Make the behavior observable and automatically gated in a demo surface.

**Exit criteria**

- UI gallery carousel page contains an inner button with stable `test_id`.
- Diag script runs and asserts:
  - drag-from-inner-button does not set “clicked” marker
  - click-on-inner-button sets “clicked” marker

## M4 — Optional: headless engine extraction plan (v2 proposal)

**Goal**

Prepare a clean refactor path to move drag/snap math and state machine out of shadcn recipes.

**Exit criteria**

- A concrete proposed module/API surface exists:
  - inputs: item sizes, viewport size, options, pointer deltas
  - outputs: offset, selected index, events/callbacks
- Layer decision documented (`fret-ui-kit` vs `fret-ui-headless`).

