---
title: "ADR 0067: Overlay Policy Architecture (Dismissal + Focus + Portal)"
---

# ADR 0067: Overlay Policy Architecture (Dismissal + Focus + Portal)

Status: Accepted

## Context

shadcn/ui components inherit a large amount of behavior from Radix UI Primitives:

- Popover / Tooltip / HoverCard / ContextMenu / DropdownMenu / NavigationMenu
- Dialog / AlertDialog / Sheet
- Command palette–style transient surfaces

In DOM ecosystems these are composed from a small set of headless overlay primitives:

- **Portal** (layering / z-order)
- **Dismissal** (Escape, pointer outside, focus outside)
- **Focus scope** (trap and restore)
- **Presence** (mount/unmount and transitions)

Fret is not a DOM/CSS runtime, so we cannot reuse React/Radix implementations. However we can and
should port the **outcomes** and lock them as tests and contracts.

The key architectural risk is letting overlay *policies* leak into `crates/fret-ui`. The runtime
must stay a small, stable substrate (mechanisms only) so that component behavior can evolve
without forcing runtime churn (ADR 0066).

## Decision

### 1) Runtime vs component boundary

`crates/fret-ui` owns overlay **mechanisms**:

- per-window overlay root stack (`push_overlay_root_ex`, `remove_layer`),
- modal barrier *mechanism* (`blocks_underlay_input` + `barrier_root` semantics),
- deterministic hit testing and pointer routing across roots (including pointer transparency),
- placement solver (`overlay_placement`) and geometry queries (`elements::bounds_for_element`).

`ecosystem/fret-ui-kit` owns overlay **policies**:

- dismissal rules (Escape, click outside, focus outside, nested overlay ordering),
- focus trap/restore (modal vs non-modal),
- open/close intent (hover intent delays, long-press, etc.),
- mount/unmount/transitions (presence).

`ecosystem/fret-ui-shadcn` owns shadcn v4 APIs and recipes on top of those policies.

### 2) Policy primitives (component-owned)

`fret-ui-kit` should provide small, headless, composable overlay primitives modeled after
Radix outcomes:

#### 2.1 Portal (layer install/uninstall)

- Installs an overlay root using `UiTree::push_overlay_root_ex(...)`.
- Uninstalls using `UiTree::remove_layer(...)`.
- Determines whether the root blocks underlay input (`blocks_underlay_input`) and whether it is
  pointer-transparent (tooltip-like).
- Records and restores previous focus on close (focus restore is policy-owned, but relies on runtime focus primitives).
- May set an explicit initial focus target on open (e.g. first focusable descendant for command palette).

Invariants:

- Installation/uninstallation is symmetric and does not leak nodes, focus, or capture.
- Root ordering is deterministic within a window (ADR 0011).

#### 2.2 Dismissal (DismissableLayer outcomes)

Behavioral outcomes to match Radix:

- Escape closes the topmost dismissable surface.
- Pointer down outside closes when configured (non-modal popovers/menus typically close).
- Pointer interactions outside modal content must not reach the underlay while a barrier is active.
- Nested overlays close in LIFO order; inner overlays can prevent parent dismissal.

Runtime dependencies:

- deterministic hit testing and active root stack,
- modal barrier scope (Gate C in ADR 0066).

Note:

- “Click outside closes” is a policy knob, not a runtime decision.
- “Pointer down outside” should be modeled as an explicit event delivered to the overlay policy
  (not as ad-hoc widget checks).

#### Docking / drag / viewport interaction notes

Editor-grade docking and embedded viewports introduce additional interaction sources (dock drags,
splitter drags, viewport tool capture). Overlay policies must remain compatible with those:

- Overlay roots should not opportunistically steal pointer capture from an active drag session that
  is owned by docking or viewport tools; instead, policies should react to explicit outside-press
  events and the active layer stack.
- Modal barriers (`blocks_underlay_input = true`) are allowed to block docking and viewport input
  by design, but this must be a deliberate component decision (e.g. dialogs/sheets), not an
  accidental outcome of implementing popovers via in-tree positioning.
- Non-modal overlays (menus/popovers) should define clear precedence rules with docking drags
  (e.g. close on drag start, or suspend dismissal during drags) in the component layer.

#### 2.3 Focus scope (FocusScope outcomes)

Modal surfaces:

- Trap focus within the modal content.
- Restore focus to the trigger (or previous focus) when closed.
- Initial focus is explicitly set by policy (runtime clears focus on barrier install).
- Auto-focus behavior is customizable by policy via Radix-like hooks (open/close auto focus may
  `preventDefault` to take control of focus movement).

Non-modal surfaces:

- Do not trap focus by default, but may implement focus restore when dismissed.

Runtime dependencies:

- focus/capture primitives and focus-visible detection (ADR 0020, ADR 0061),
- focus traversal baseline (`focus.next` / `focus.previous`) within the active modal scope (ADR 0068),
- modal barrier focus constraints (Gate C in ADR 0066).

#### 2.4 Presence (mount/unmount and transitions)

Presence is component-owned:

- Runtime should not encode transition policy.
- Components may keep an overlay mounted while animating out, but must keep dismissal/focus
  semantics correct during the transition window.

Recommended contract surface for policy-owned presence:

- Treat `open` (user intent) and `present` (mounted-in-tree) as separate signals.
  - `open` drives interactive behavior (focus, dismissal, pointer affordances).
  - `present` drives whether the overlay root exists and is painted at all.
- Initial focus is applied on the **opening edge** (`open: false → true`), not on every frame.
- Focus restoration should occur when the overlay is **unmounted** (`present: true → false`), so a
  close animation can complete without prematurely moving focus.
- Pointer routing during the out-transition depends on modality:
  - Non-modal overlays should become pointer-transparent while `open=false` but `present=true`
    (click-through should remain click-through).
  - Modal overlays must continue to block underlay input while `present=true`, even if `open=false`
    (the barrier remains authoritative for the duration of the out-transition).

### 3) Modal barrier pattern (recommended structure)

To align with Flutter/WPF-style modal barriers and Radix outcomes:

- A modal overlay installs an overlay root with `blocks_underlay_input = true`.
- The overlay root contains:
  - a full-window barrier element (captures pointer outside content),
  - the modal content element subtree.

Invariants:

- Background roots are inert while the barrier is active (pointer + keyboard + focus + semantics).
- Barrier always receives pointer events that are outside modal content, enabling “click outside”
  policies without leaking events to the underlay.

## References

- Runtime boundary and gates: `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- Multi-root overlays: `docs/adr/0011-overlays-and-multi-root.md`
- Overlay placement solver: `docs/adr/0064-overlay-placement-contract.md`
- Non-modal outside press (click-through): `docs/adr/0069-outside-press-and-dismissable-non-modal-overlays.md`
- Authoritative behavior target: Radix UI Primitives (upstream: <https://github.com/radix-ui/primitives>; pinned locally, see `docs/repo-ref.md`) via shadcn/ui (`repo-ref/ui`)
- Modal barrier conceptual model: Flutter `ModalBarrier` / WPF-style overlay barrier
- Zed/GPUI (non-normative):
  - managed overlay views that dismiss via an explicit `DismissEvent`:
    `repo-ref/zed/crates/gpui/src/window.rs` (`ManagedView`, `DismissEvent`)
  - overlay-style input blocking via hitbox occlusion:
    `repo-ref/zed/crates/gpui/src/window.rs` (`HitboxBehavior::{BlockMouse, BlockMouseExceptScroll}`)
