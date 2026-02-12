---
title: "ADR 0150: Pointer Identity (`PointerId`) and Multi-Pointer Capture/Dispatch"
---

# ADR 0150: Pointer Identity (`PointerId`) and Multi-Pointer Capture/Dispatch

Status: Proposed

Scope: `fret-core` input contracts and `fret-ui` runtime dispatch/capture semantics.

Related:

- ADR 0017: `docs/adr/0017-multi-window-display-and-dpi.md`
- ADR 0041: `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`
- ADR 0072: `docs/adr/0072-docking-interaction-arbitration-matrix.md`
- ADR 0149: `docs/adr/0149-headless-dnd-toolbox-and-ui-integration.md`
- ADR 0151: `docs/adr/0151-multi-pointer-drag-sessions-and-routing-keys.md`

## Context

Fret already targets editor-grade interaction and multi-window workflows:

- multi-window / tear-off docking and cross-window internal drag (ADR 0041),
- deterministic interaction arbitration between docking, overlays, and viewport capture (ADR 0072),
- a strict coordinate contract (logical pixels as canonical; ADR 0017).

To avoid future large-scale rewrites when adding touch/pen/mobile and richer ecosystem drag-and-drop,
we need a stable, portable way to identify pointer contacts across the entire input stack.

Today, Fret pointer events do not carry an explicit pointer identity, and the runtime’s pointer capture
is effectively “single pointer” (one captured node for the whole window).

This blocks or complicates:

- true multi-touch (simultaneous drags/gestures),
- concurrent interactions (e.g. two-finger pan/zoom while a stylus drags),
- per-pointer capture and cancellation semantics,
- multi-pointer-aware DnD/toolbox integration (ADR 0149).

Pointer identity is a hard-to-change contract; deciding it early avoids future breaking migrations.

## Goals

1. Add an explicit `PointerId` to core input events so that multiple pointers can be tracked and routed.
2. Define stable invariants for pointer identity across windows and across the runner/runtime boundary.
3. Define the runtime contract for pointer capture and cancellation as *per-pointer* concepts.
4. Provide a migration path that does not immediately force all widgets to become multi-pointer-aware.

## Non-Goals

- Defining multi-touch gesture semantics (pinch/rotate) beyond identifying pointers.
- Changing docking arbitration rules (ADR 0072 remains authoritative).
- Introducing a full “gesture recognizer” framework in the runtime.

## Decision

### 1) Add `PointerId` to `fret-core` pointer-related events

Introduce a portable, comparable identifier:

- `pub struct PointerId(pub u64);`

and include it in:

- `PointerEvent::{Move, Down, Up, Wheel, PinchGesture}`
- `PointerCancelEvent`
- `InternalDragEvent`

Rationale:

- A `PointerId` is required to represent multi-touch and stylus/mouse concurrency.
- `InternalDragEvent` must carry `PointerId` so cross-window internal drag can remain correctly associated
  with its originating pointer across synthesized `Enter/Over/Leave/Drop/Cancel` events (ADR 0041).

### 2) Runner mapping rules (portable invariants)

The runner/backend is responsible for populating `PointerId` using platform-native identifiers.

Invariants:

- `PointerId` is stable for the duration of a pointer contact (mouse button session, touch contact, pen contact).
- `PointerId` may be reused only after the corresponding contact ends (Up/Cancel).
- For the mouse pointer, `PointerId(0)` is reserved and used consistently.
- `PointerId` is app-scoped: the same contact retains its id even if events are delivered to different windows
  during cross-window routing (runner-synth internal drag; ADR 0041).

### 3) Runtime capture is defined per-pointer

Pointer capture is a per-pointer mapping:

- `capture: PointerId -> NodeId`

Semantics:

- Pointer events for a given `PointerId` are routed to its captured node if one exists, regardless of hit-testing.
- Capturing a pointer does not affect routing for other pointers.
- Releasing capture for a pointer only affects that pointer.
- `PointerCancelEvent` cancels capture for its `pointer_id` and triggers any subsystem-owned cancel behavior.

### 4) Migration and compatibility

We adopt a staged migration to keep churn manageable:

1. Add `PointerId` fields to the `fret-core` event structs/enums (breaking change gated behind a single migration).
2. Update runners to populate ids (mouse=0; touch/pen=platform contact id).
3. Update `fret-ui` to:
   - store capture per pointer internally,
   - continue supporting existing widgets by treating “no explicit pointer id usage” as operating on `PointerId(0)`.
4. Gradually update policy-heavy subsystems and ecosystem crates (docking, overlays, viewport tools, DnD integrations)
   to use the correct `PointerId` for capture and drag sessions.

## Consequences

- `fret-core` input event contracts change shape; downstream code must migrate once.
- `fret-ui` must evolve capture storage and event dispatch plumbing (even if most widgets remain single-pointer).
- Multi-pointer support becomes feasible without redesigning event types, drag sessions, or DnD toolboxes later.

## Notes and Alignment

- This ADR intentionally focuses on identity and dispatch/capture. Gesture policy (e.g. touch pan vs scroll,
  two-finger transforms) remains in higher layers (e.g. `fret-ui-kit`) or future ADRs.
- This ADR is a prerequisite for “multi-pointer-ready” ecosystem drag-and-drop (ADR 0149).
