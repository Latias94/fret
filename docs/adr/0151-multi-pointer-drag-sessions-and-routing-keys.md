---
title: "ADR 0151: Multi-Pointer Drag Sessions, Extensible Drag Kinds, and Internal-Drag Routing Keys"
---

# ADR 0151: Multi-Pointer Drag Sessions, Extensible Drag Kinds, and Internal-Drag Routing Keys

Status: Proposed

Scope: `fret-runtime` drag session contracts and `fret-ui` internal-drag routing keys (mechanism-only).

Related:

- ADR 0041: `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`
- ADR 0072: `docs/adr/0072-docking-interaction-arbitration-matrix.md`
- ADR 0149: `docs/adr/0149-headless-dnd-toolbox-and-ui-integration.md`
- ADR 0150: `docs/adr/0150-pointer-identity-and-multi-pointer-capture.md`

## Context

Fret already has an internal drag session concept (ADR 0041) and a runtime mechanism to route internal drag events,
including a per-window internal-drag anchor override (`fret_ui::internal_drag`) used for docking tear-off.

Today, the host-facing drag API is effectively “single active session”:

- `DragHost::drag() -> Option<&DragSession>`
- `DragHost::begin_*_drag_with_kind(...)`

and the routing key for internal-drag anchors is based on a closed `DragKind` enum (e.g. docking vs “custom”).

This becomes a future breaking point when:

- we add true multi-pointer interaction (ADR 0150),
- multiple concurrent drag sessions exist (e.g. touch + pen, or two touch contacts),
- multiple ecosystem DnD flows need stable cross-window routing anchors (not just docking),
- third-party components want to introduce new drag kinds without modifying the core enum.

This ADR locks the contract shape needed to avoid a later large-scale rewrite.

## Goals

1. Make drag sessions multi-pointer-ready at the host boundary without forcing all widgets to adopt multi-pointer
   logic immediately.
2. Make “drag kind” extensible so ecosystem/third-party crates can use routing anchors without editing core enums.
3. Align internal-drag routing and arbitration with existing contracts (ADR 0041 / ADR 0072 / ADR 0150).
4. Preserve determinism: event routing and drag session selection must be stable and testable.

## Non-Goals

- Defining higher-level DnD policy (sensors/collision/modifiers) which belongs to the ecosystem (ADR 0149).
- Replacing the runner’s responsibility for cross-window hover/drop (ADR 0041).

## Decision

### 1) Introduce an extensible drag kind identifier

Replace the closed `DragKind` enum with an extensible identifier newtype:

- `pub struct DragKindId(pub u64);`

with reserved constants for built-ins, e.g.:

- `pub const DRAG_KIND_DOCK_PANEL: DragKindId = DragKindId(…);`

Rationale:

- internal-drag routing anchors need a stable key across crates and across windows;
- third-party crates must be able to define new kinds without central coordination;
- avoiding enum growth prevents churn and repeated “match exhaustiveness” migrations.

Debugging/readability:

- hosts or ecosystems may install an optional “kind name registry” (mechanism-only) to map `DragKindId -> &'static str`
  for logs and inspector tooling.

### 2) Make drag sessions keyed by `PointerId` at the host boundary

Extend the `DragHost` contract to support per-pointer drag session storage:

- `fn drag(&self, pointer: PointerId) -> Option<&DragSession>;`
- `fn drag_mut(&mut self, pointer: PointerId) -> Option<&mut DragSession>;`
- `fn cancel_drag(&mut self, pointer: PointerId);`
- `fn begin_drag_with_kind<T: Any>(..., pointer: PointerId, kind: DragKindId, ...);`
- `fn begin_cross_window_drag_with_kind<T: Any>(..., pointer: PointerId, kind: DragKindId, ...);`

Compatibility note:

- A temporary adapter may treat the existing single-session storage as `PointerId(0)` until callers migrate.

### 3) Standardize `DragSession` fields to match ADR 0041 and multi-pointer needs

`DragSession` must include:

- `session_id`: monotonic `DragSessionId` (app-scoped),
- `pointer_id`: `PointerId`,
- `kind`: `DragKindId`,
- window tracking: `source_window`, `current_window`, `cross_window_hover`,
- positions: `start_position`, `position` (logical px, per ADR 0017),
- `phase`: `Starting | Dragging | Dropped | Canceled`,
- opaque, app-owned payload (type-erased).

The runtime treats the payload as opaque; acceptance/policy remains outside the mechanism layer.

### 4) Internal-drag routing anchors are keyed by `(window, DragKindId, PointerId?)`

We lock the routing key as:

- minimum: `(window, kind)` for “kind-scoped anchor” (docking-class),
- optional extension: `(window, kind, pointer)` when a subsystem needs independent anchors per pointer.

The mechanism should prefer the smallest key that satisfies known use-cases. The initial implementation should keep
the `(window, kind)` shape, but the contract must allow adding the pointer dimension without renaming the feature.

### 5) Session selection for `Event::InternalDrag`

When the runtime receives `Event::InternalDrag { pointer_id, ... }`:

1. The runtime selects the active drag session for that `pointer_id` (if any).
2. If cross-window hover is active for that session, an anchor override may apply for its `kind` (docking-class).
3. Otherwise, internal drag follows normal hit-testing rules (overlays first; ADR 0011) with modal barrier semantics.

This preserves ADR 0041’s “drag-and-drop style events must follow the cursor, not pointer capture” invariant.

### 6) Determinism requirements

- If multiple droppable candidates exist, tie-breaking must be stable (z-order group, explicit ordering, stable ids).
- If multiple drag sessions exist (multiple pointers), internal drag events must never “jump sessions”; selection is
  purely by `pointer_id`.

## Migration Plan

1. Land ADR 0150 changes: `PointerId` in input events and per-pointer capture semantics.
2. Introduce `DragKindId` alongside the existing `DragKind` for a deprecation window (if needed), then migrate.
3. Update `DragHost` to be pointer-keyed; provide a compatibility adapter that maps legacy single-session hosts to
   `PointerId(0)`.
4. Update docking to use `DRAG_KIND_DOCK_PANEL`.
5. Ensure the internal-drag route table uses `DragKindId` keys.
6. Update ecosystem DnD integrations (ADR 0149) to use pointer-keyed sessions.

## Open Questions

- Do we want a built-in registry for `DragKindId -> name` in `fret-runtime`, or should it remain host/ecosystem-only?
- Should `DragSessionId` be global monotonic across all pointers, or monotonic per pointer? (Global is recommended for
  easier tracing and debugging.)
