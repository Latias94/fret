---
title: Docking Interaction Arbitration Checklist (ADR 0072)
---

# Docking Interaction Arbitration Checklist (ADR 0072)

This is a **manual + automated conformance checklist** for:

- docking interactions (tab drag, split drag, floating window drag),
- non-modal overlays (popover/menu-style click-through surfaces),
- viewport tool capture (camera drags, gizmos, marquee),
- and their arbitration rules.

Source of truth:

- `docs/adr/0072-docking-interaction-arbitration-matrix.md`

This checklist is intentionally short and is meant to be used while iterating on docking and
viewport tooling, to avoid late rewrites and accidental behavior drift.

---

## How to run the current demos

- Docking demo: `cargo run -p fret-demo --bin docking_demo`
- Docking arbitration demo (combined harness): `cargo run -p fret-demo --bin docking_arbitration_demo`
  - Single-window / no OS multi-window mode (validates ADR 0084 degradation): `FRET_SINGLE_WINDOW=1 cargo run -p fret-demo --bin docking_arbitration_demo`
- Components overlays demo: `cargo run -p fret-demo --bin components_gallery`

### Docking arbitration demo: synth pointer mode

The docking arbitration demo includes a **synthetic pointer stream** so multi-pointer arbitration
and input edges can be validated even on hardware without touch/pen.

Controls (when synth mode is enabled, these keys are consumed):

- Toggle synth mode: `F1`
- Move synth pointer: `I/J/K/L`
- Synth touch press/release: `Space` (emits `PointerId != 0` with `PointerType::Touch`)
- Mouse right press/release at synth position: `B` (emits `PointerId(0)` with `MouseButton::Right`)
- Mouse wheel up/down at synth position: `U` / `O` (emits `PointerId(0)` wheel)

---

## Driver integration checklist (P0)

These are the most common integration failure modes that cause late rewrites in editor-grade docking.

### Keep the dock host alive

- Always submit a `DockSpace` host for every window that participates in docking.
- Do not conditionally omit the dock host when a window is "collapsed" or when panels are not currently visible.

### Submit docking early in the per-frame build

- Build the docking host early so it can act as a stable drop target and internal drag route anchor (ADR 0072).
- Prefer a driver pattern where docking is built before other UI that might participate in hit-testing during drags.

### Bind panel UI roots before layout/paint

- Render/bind dock panel roots every frame before `UiTree::layout_all` / `UiTree::paint_all`.
- Prefer calling `fret_components_docking::render_and_bind_dock_panels(...)` instead of ad-hoc `DockPanelContentService::set(...)`.

---

## Conformance checklist (P0)

These items are copied from ADR 0072 and expanded into concrete steps.

### 1) Dock drag does not trigger "outside press" dismissal while dragging

**Goal**

- While a dock drag session is active, click-through outside press observers must not fight the drag.

**Manual steps (needs combined demo)**

1. Open a non-modal overlay (popover/menu) in the same window.
2. Start a dock tab drag (do not release yet).
3. Move the pointer and click on underlay content.

**Expected**

- No "outside press" dismissal side effects are triggered mid-drag.
- The drag remains the owner of the pointer session.

References:

- ADR 0069 (`outside press observer`): `docs/adr/0069-outside-press-and-dismissable-non-modal-overlays.md`

### 2) Starting a dock drag closes/suspends non-modal overlays in the same window

**Goal**

- Docking interactions should not leave stale popovers/menus open in the same window.

**Manual steps (needs combined demo)**

1. Open a popover/menu (non-modal).
2. Start a dock drag on a tab bar.

**Expected**

- The overlay closes or becomes non-interactive/pointer-transparent until the drag ends.
- Focus is not forced to change in a way that breaks the drag.

### 3) Modal dialogs block docking and viewport input consistently

**Goal**

- Modal barriers must make underlay docking/viewport content inert (pointer + keyboard + semantics).

**Manual steps**

1. In `components_gallery`, open a modal (`Dialog` / `AlertDialog` / `Sheet`).
2. Try to interact with underlay content.

**Expected**

- Underlay does not receive input while modal barrier is active.

References:

- ADR 0011 (`multi-root overlays`): `docs/adr/0011-overlays-and-multi-root.md`
- ADR 0067 (`overlay policy split`): `docs/adr/0067-overlay-policy-architecture-dismissal-focus-portal.md`

### 4) Viewport tool capture and dock drag capture do not conflict

**Goal**

- A single pointer session cannot be owned by both viewport capture and dock drag.

**Automated coverage (unit tests)**

- Dock drag suppresses viewport hover + wheel forwarding:
  - `ecosystem/fret-docking/src/dock/tests.rs`
- Viewport capture continues to emit clamped pointer moves even when leaving the viewport:
  - `ecosystem/fret-docking/src/dock/tests.rs`

**Manual steps (needs viewport demo)**

1. Start a viewport drag (camera orbit / marquee).
2. While dragging, attempt to start a dock tab drag or split drag.

**Expected**

- The active session remains the owner; no competing capture session starts.
- Secondary right-click presses do not trigger context menus while viewport capture is active.

References:

- ADR 0072: `docs/adr/0072-docking-interaction-arbitration-matrix.md`
- Viewport input boundary: `docs/adr/0025-viewport-input-forwarding.md`

### 5) Escape cancels dock drag and restores focus safely

**Goal**

- Escape cancels the cross-window dock drag session.
- Focus/overlays do not end up in a broken state.

**Manual steps**

1. Start a dock tab drag.
2. Press Escape.

**Expected**

- Drag session ends (no dock op committed).
- Focus is restored safely if it was lost during the drag.
- Non-modal overlays that were closed for drag hygiene remain closed unless explicitly re-opened (e.g. user re-triggers the popover/menu).

---

## Harness status (P0)

Status: implemented in `fret-demo` as `docking_arbitration_demo`.

The harness combines:

- one dock space with at least one real viewport panel (generates `Effect::ViewportInput`),
- a non-modal overlay trigger (popover/menu),
- and a modal dialog trigger,

so that ADR 0072 can be validated end-to-end in one window.
