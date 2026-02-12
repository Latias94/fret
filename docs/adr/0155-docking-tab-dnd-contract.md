---
title: "ADR 0155: Docking Tab Drag-and-Drop Contract (Headless DnD Alignment)"
---

# ADR 0155: Docking Tab Drag-and-Drop Contract (Headless DnD Alignment)

Status: Proposed

Scope: `ecosystem/fret-docking` behavior and its interaction boundaries with internal drag sessions (runner/runtime) and headless DnD policy (`ecosystem/fret-dnd` / `ecosystem/fret-ui-kit`).

Related:

- ADR 0017: `docs/adr/0017-multi-window-display-and-dpi.md`
- ADR 0041: `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`
- ADR 0072: `docs/adr/0072-docking-interaction-arbitration-matrix.md`
- ADR 0149: `docs/adr/0149-headless-dnd-toolbox-and-ui-integration.md`
- ADR 0150: `docs/adr/0150-pointer-identity-and-multi-pointer-capture.md`
- ADR 0151: `docs/adr/0151-multi-pointer-drag-sessions-and-routing-keys.md`

## Context

Docking tab dragging is one of the most “hard to change later” editor-grade interactions:

- It must work across multiple windows (tear-off + drop into other windows).
- It must be deterministic with respect to input replay (thresholds, hover selection, tie-breakers).
- It must obey the docking interaction arbitration matrix (ADR 0072) so viewport capture, overlays, and docking do not fight for the same pointer stream.

Fret already has an internal, cross-window drag session mechanism owned by the runner (ADR 0041). Separately, we introduced a headless DnD policy toolbox and a UI integration registry (ADR 0149) to avoid re-implementing sensors/collision/modifiers per subsystem.

This ADR defines how docking tab drag/tear-off composes those layers without pushing policy into the runtime/runner.

## Goals

1. Keep docking tab drag behavior deterministic (thresholds, hover choice, stable IDs).
2. Use the headless DnD toolbox for policy-heavy decisions (activation/collision) where possible.
3. Minimize future rewrites by locking in a small but correct contract surface that supports multi-window and multi-pointer (mobile-ready).
4. Preserve ADR 0072 arbitration: once docking “claims” the pointer for a potential drag, viewport hover/wheel forwarding must be suppressed until the interaction resolves.

## Non-Goals

- Defining a general-purpose widget-level draggable/droppable API in `crates/fret-ui`.
- Solving OS drag-and-drop (ADR 0053) or clipboard formats here.
- Implementing advanced droppable shapes (paths/ports) for docking; docking targets remain rect-based.

## Decision

### 1) Delay cross-window `DragSession` creation until activation is satisfied

Docking MUST NOT create a cross-window `DragSession` on tab pointer-down.

Instead, docking maintains a local `pending` state keyed by `PointerId`:

- On tab down: record `{start_position, start_tick, panel_id, grab_offset}` and capture the pointer.
- On pointer move: if the activation constraint is satisfied (default: ~6 logical px, ImGui-style), docking starts a cross-window `DragSession` and marks it `dragging = true`.
- On pointer up/cancel (before activation): clear `pending` and release capture, without ever allocating a `DragSession`.

Rationale:

- Avoids runner/runtime overhead (and potential cross-window routing side effects) for clicks that never become drags.
- Provides a deterministic, testable activation threshold independent of OS/device event granularity.
- Aligns the policy boundary: activation belongs to ecosystem policy (ADR 0149), while cross-window routing belongs to the runner (ADR 0041).

### 2) Pending docking drags still suppress viewport hover/wheel forwarding

While a docking drag *might* start (pending or active), docking owns the interaction per ADR 0072.

Therefore, viewport hover forwarding and wheel forwarding MUST be suppressed while either:

- a docking drag session exists (even if not yet marked `dragging`), or
- a pending docking drag exists for the pointer.

### 3) Coordinate contract: window-local logical pixels

Docking drag geometry uses window-local logical pixels as canonical input coordinates:

- `start_position` and `position` are expressed in window-local logical pixels.
- `grab_offset` for tear-off is tab-local, so that when a tab becomes index 0 in a new floating window, the tab stays under the cursor (ImGui-style behavior).

This is consistent with ADR 0017 and avoids mixing pixel-space thresholds with scale factors.

### 4) Docking hover target selection uses headless collision with stable tie-breakers

Docking hover selection SHOULD reuse headless collision strategies:

- droppables are represented as rects (tab bar insertion zones, hint rects, edge zones, fallback center),
- collision strategy uses pointer-within (or equivalent), and
- ties are broken deterministically (z-index and stable `DndItemId`).

This keeps docking hover selection aligned with other ecosystem DnD surfaces (sortable, node graph, etc.).

#### Tab insertion index semantics

When the hover target is a tab bar, docking derives `insert_index` using headless “insertion side” semantics
(`InsertionSide::{Before,After}`) along the X axis:

- pointer in the left half of the *over* tab rect inserts before that tab,
- pointer in the right half inserts after that tab,
- pointer beyond the last tab inserts at the end.

### 5) Multi-pointer readiness

All docking drag state MUST be keyed by `PointerId` (ADR 0135/0151). No single-pointer global “current drag” state is allowed in the docking UI layer.

## Implementation Notes (Evidence)

- Pending → activated transition and arbitration: `ecosystem/fret-docking/src/dock/space.rs`.
- Pending-stage suppression regression test: `ecosystem/fret-docking/src/dock/tests.rs` (`pending_dock_drag_suppresses_viewport_hover_and_wheel_forwarding`).
- Collision-based hover selection: `ecosystem/fret-docking/src/dock/space.rs` (`dock_drop_target_via_dnd` helper).

## Open Questions

1. Should we extract a shared helper for “internal cross-window `DragSession` ↔ headless collision registry” so docking is not the only consumer?
2. Should the default activation threshold become configurable via `DockingInteractionSettings`, and should it be expressed in logical px, screen px, or a density-aware unit?
