---
title: "ADR 0072: Docking Interaction Arbitration Matrix (Docking vs Overlays vs Viewport Capture)"
---

# ADR 0072: Docking Interaction Arbitration Matrix (Docking vs Overlays vs Viewport Capture)

Status: Accepted

## Update: Multi-pointer and pointer identity (2026-01)

This ADR was originally authored with a “single pointer” mental model. Fret’s input contracts are evolving to
support multi-pointer interaction via explicit `PointerId` (ADR 0150) and pointer-keyed drag sessions (ADR 0151).

This section locks the arbitration rules in the presence of multiple concurrent pointers.

## Implementation Status (as of 2025-12-29)

This ADR is implemented for the non-modal overlay stack and docking drags:

- Dock drag sessions are represented as `DragKind::DockPanel` (runtime-level mechanism).
- While a dock drag session is active in a window, `fret-ui-kit/window_overlays` closes
  non-modal popovers (sets their `open` model to `false`) and suppresses hover overlays to avoid
  fighting docking interaction.
- While a dock drag session is active, docking suppresses starting competing capture sessions from
  secondary pointer-down events (e.g. viewport capture) in the same window.
- While a dock drag session is active, docking suppresses forwarding pointer-move and wheel events
  to embedded viewports in the same window.
- Dock drags honor modifier inversion: docking preview enable/disable is driven by
  `fret_runtime::DockingInteractionSettings` (default: dock by default; hold Shift to float).

## Context

Fret targets editor-grade interaction: docking, tear-off windows, multiple embedded viewports, and
multi-root overlays (menus/popovers/tooltips/modals). These systems compete for pointer capture,
hit-testing, focus, and dismissal behavior.

Without a locked arbitration contract, the UI will drift into inconsistent behavior such as:

- docking drags accidentally closing or being blocked by non-modal overlays,
- viewport tool capture fighting docking drag capture,
- modal barriers inconsistently blocking docking and viewport input,
- "click outside" dismissal triggering during a docking drag,
- cross-root focus/capture leaks after closing overlays during drags.

Existing contracts define the *mechanisms* but not the arbitration policy:

- Multi-root overlays + modal blocking: `docs/adr/0011-overlays-and-multi-root.md`
- Overlay policy boundary (dismissal/focus/portal): `docs/adr/0067-overlay-policy-architecture-dismissal-focus-portal.md`
- Docking ops + persistence: `docs/adr/0013-docking-ops-and-persistence.md`
- Viewport input forwarding: `docs/adr/0025-viewport-input-forwarding.md`
- Viewport tools capture example: `docs/adr/0049-viewport-tools-input-capture-and-overlays.md`

This ADR defines a deterministic arbitration matrix for pointer/keyboard interactions between
docking, overlays, and viewport tools.

## Decision

### 1) Terms and states

- **Dock drag session**: a pointer interaction that began on a dock tab (or a dock chrome affordance)
  and is currently dragging a `PanelKey` payload.
- **Viewport tool capture**: an app-owned capture mode where viewport tools continue receiving move/up
  regardless of pointer leaving viewport bounds.
- **Overlay categories**:
  - **Modal**: installed with `blocks_underlay_input = true` (barrier-backed).
  - **Non-modal dismissable**: popovers/menus/hover cards that commonly close on outside press.
  - **Pointer-transparent**: tooltip-like overlays that do not participate in hit-testing.

### 2) Arbitration order (pointer events)

For each window, pointer events are routed using the following precedence:

1) **Pointer capture** is authoritative (runtime contract).
2) If a **modal barrier** is visible (`blocks_underlay_input = true`), only that barrier root and
   roots above it are eligible targets; docking and viewport interactions are blocked by design.
3) Otherwise, if a **dock drag session** is active, docking receives pointer move/up for that session.
4) Otherwise, if a **viewport tool capture** is active, viewport tools receive pointer move/up for that session.
5) Otherwise, normal hit-testing applies across visible roots in z-order (ADR 0011).

Notes:

- A dock drag session and a viewport tool capture must not both become active from the same pointer
  down. The initial hit target decides which interaction claims the session.
- Pointer-transparent overlays must not block docking or viewport interaction.

### 2.1) Multi-pointer arbitration (pointer-keyed routing + window-global locks)

This section applies when multiple pointers (mouse/touch/pen contacts) are active concurrently.

Definitions:

- **Pointer identity**: Each pointer stream has a stable `PointerId` for the duration of the contact (ADR 0150).
- **Per-pointer capture**: capture is stored per `PointerId` (ADR 0150).
- **Per-pointer session ownership**: for a given `PointerId`, at most one subsystem owns the “active session”
  (docking drag, viewport capture, or normal hit-tested interaction).

Rules:

1) **Arbitration is evaluated per `PointerId`**:
   - Pointer events for a given `PointerId` route according to the order in §2, but interpreting “capture”, “dock
     drag session”, and “viewport tool capture” as *per-pointer* states.
   - A pointer cannot be simultaneously captured by two subsystems; capture ownership follows the winning session.

2) **Modal barriers are window-global**:
   - If a modal barrier is visible (`blocks_underlay_input = true`), it constrains routing for **all pointers**
     in the window (not just the pointer that opened the modal).

3) **Docking drag is window-exclusive (P0)**:
   - While any docking drag session is active in a window, a second docking drag session must not start in that
     same window, even from a different `PointerId`.
   - While docking drag is active, starting competing viewport capture sessions in the same window is disallowed
     (to prevent split ownership and inconsistent overlay/focus outcomes).

4) **Viewport capture may be per-pointer**:
   - Viewport capture is allowed to be active for one pointer while another pointer performs normal hit-tested
     interaction, as long as no window-exclusive lock (modal barrier, docking drag) blocks it.

5) **Overlay suppression scope**:
   - Starting a docking drag closes/suspends non-modal dismissable overlays for the **entire window** (all pointers),
     consistent with §3 (“Dock drag start policy”).
   - Viewport capture may suppress incidental hover-driven overlays for the owning pointer only (editor policy),
     but must not forcibly close unrelated overlays unless explicitly desired by the app policy.

### 3) Dock drag start policy (interaction hygiene)

When a dock drag session starts:

- Close or suspend **non-modal dismissable overlays** in the same window by default (menus, popovers,
  context menus). This prevents outside-press dismissal logic and hover routing from fighting the drag.
- Tooltips may remain visible if they are pointer-transparent, but must not capture or become hit-testable.
- The docking system owns pointer capture for the duration of the drag.

This policy is component-owned (overlay policy layer) but the behavior outcome is locked here.

When the drag ends:

- Do not automatically re-open overlays that were closed/suspended for drag hygiene. Re-opening is an explicit
  application action (e.g. user re-triggers the popover/menu).
- Focus may be restored safely if it was lost during the drag (runner/platform behavior), but should not be
  forced to change while the drag is still active.

### 4) Viewport tool capture start policy

When a viewport tool capture starts:

- Non-modal overlays should not open due to incidental pointer moves during capture.
- Context menus triggered by right-click should not open if the tool system is currently using that
  button for navigation/capture (editor policy).

### 5) Modifier-based docking enable/disable during drags

To match established editor UX (ImGui-class behavior), a modifier (recommended: Shift) may invert
docking behavior during a drag:

- Default: dragging a tab previews docking drop zones.
- While the modifier is held: docking previews are disabled and the drag becomes a "float/undock"
  intent (or the inverse if the user prefers "dock only when Shift is held").

The exact modifier and default are user-configurable via settings; the existence of an inversion
mechanism is locked.

### 6) Keyboard interaction while dragging

While a dock drag session is active:

- Escape cancels the drag session (no `DockOp` committed).
- Focus should not be forcibly changed by overlay dismissal during the drag.

Implementation note (current):

- Desktop runner consumes `Escape` during an active cross-window dock drag and cancels the drag
  session immediately (also clearing cross-window hover state and stopping any tear-off follow).

## Conformance checklist (P0)

- Dock drag does not trigger "outside press" dismissal while dragging.
- Starting a dock drag closes/suspends non-modal overlays in the same window.
- Modal dialogs block docking and viewport input consistently.
- Viewport tool capture and dock drag capture do not conflict: a single pointer session cannot be owned by both.
- Escape cancels dock drag and restores focus safely (overlays remain closed unless explicitly re-opened).

## Non-Goals

- Defining the full editor tool API (see ADR 0049).
- Defining tab semantics, roving focus, or keyboard navigation patterns (APG-aligned work).
- Cross-window drag session transport details (see ADR 0041 / ADR 0053).

## References

- `docs/adr/0011-overlays-and-multi-root.md`
- `docs/adr/0067-overlay-policy-architecture-dismissal-focus-portal.md`
- `docs/adr/0013-docking-ops-and-persistence.md`
- `docs/adr/0025-viewport-input-forwarding.md`
- `docs/adr/0049-viewport-tools-input-capture-and-overlays.md`
- `docs/adr/0150-pointer-identity-and-multi-pointer-capture.md`
- `docs/adr/0151-multi-pointer-drag-sessions-and-routing-keys.md`
