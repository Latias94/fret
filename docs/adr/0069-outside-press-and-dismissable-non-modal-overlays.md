---
title: "ADR 0069: Outside Press for Non-Modal Overlays (Dismissable, Click-Through by Default)"
---

# ADR 0069: Outside Press for Non-Modal Overlays (Dismissable, Click-Through by Default)

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

2) **Non-modal overlays** (dismissable, click-through by default):
   - installed with `blocks_underlay_input = false`,
   - do not block underlay input/semantics,
   - can dismiss on outside press via a runtime observer pass.

Additionally, some **non-modal** overlays (notably Radix-style menus) need an extra outcome:

- **Disable outside pointer events while open** (Radix `disableOutsidePointerEvents`):
  - underlay widgets must not receive *any* pointer interaction while the overlay is open, not just
    the initial pointer-down.
  - this is **not** a full modal barrier (no focus trap requirement; no a11y "hide others" in this
    ADR), but it does require a runtime input-scoping mechanism beyond the pointer-down observer
    pass.
  - importantly, menus still need *global pointer movement* outside the overlay subtree for
    policies like Radix Menu “safe hover corridor” between a submenu trigger and its submenu. In
    Fret this is modeled as an opt-in **pointer-move observer** on the dismissible root (see
    `DismissibleActionHooks::on_pointer_move`) and a per-layer `wants_pointer_move_events` flag,
    and it must continue to run even when hit-tested pointer dispatch is suppressed by occlusion.

### Outside press observer pass

The runtime provides opt-in per-layer flags:

- `wants_pointer_down_outside_events`
- `consume_pointer_down_outside_events` (optional; defaults to false)

When a `PointerEvent::Down` occurs and there is no pointer capture, the runtime performs an
**outside press observer pass** before the normal hit-test dispatch:

- It finds the topmost visible non-modal layer (in paint order) that:
  - is in the active input stack (respecting modal barrier scoping),
  - has `wants_pointer_down_outside_events = true`,
  - and does **not** contain the hit-tested target.
- The runtime dispatches the same pointer-down event to that layer's root chain.
- By default, the observer pass does **not** prevent the normal hit-tested dispatch: non-modal
  outside press is **click-through**.
- When `consume_pointer_down_outside_events = true` and the observer pass dispatches an outside
  press event for that layer, the runtime suppresses the normal hit-tested **pointer-down**
  dispatch for the same input event.
  - This matches the common “menus are not click-through on pointer down” behavior in native UI
    stacks (Unity, Godot, desktop toolkits).
  - This suppression is scoped to the observer-triggering layer only (topmost in paint order) and
    respects branches (see below).
- The observer pass must be **side-effect free** for input routing:
  - it must not change pointer capture,
  - it must not override focus,
  - it must not block/bubble-stop the subsequent normal dispatch *except* when the layer is
    explicitly configured to consume outside pointer-down events as described above.
  - Implementation note: observer dispatch runs through the normal widget event entrypoint with
    `InputDispatchPhase::Observer`; the runtime ignores focus/capture/propagation requests during
    this pass (except for the explicit consume-on-outside-pointer-down suppression described
    above).
- Touch pointers are treated differently to preserve scroll/drag ergonomics:
  - the runtime records a touch pointer-down-outside candidate on `PointerDown`,
  - cancels it once the pointer moves beyond a small slop threshold,
  - and dispatches the outside-press observer event on `PointerUp` only when the candidate remains
    valid.
  - When `consume_pointer_down_outside_events = true`, the runtime suppresses the normal hit-tested
    **pointer-up** dispatch for the same touch interaction.

This is the minimal contract needed to express Radix-like dismissal behavior without adding a
matrix of per-component runtime toggles.

### Dismiss handlers and "prevent default" (Radix outcome)

Radix allows dismissal callbacks (e.g. `onPointerDownOutside`, `onInteractOutside`,
`onEscapeKeyDown`) to "prevent default" and keep the overlay open.

In Fret, the overlay substrate expresses this outcome via an optional dismiss handler:

- `OnDismissRequest` receives a mutable `DismissRequestCx { reason, ... }`.
- Handlers may call `req.prevent_default()` to keep the overlay open.
- When default is not prevented, overlay orchestration closes the `open` model automatically.

### Disable outside pointer events (Radix `disableOutsidePointerEvents`)

Radix `DismissableLayer` has a stronger option than "consume outside pointer down": it can prevent
pointer events outside the surface from reaching the underlay while the surface is open.

This differs materially from `consume_pointer_down_outside_events`:

- `consume_pointer_down_outside_events` only suppresses the normal hit-tested **pointer-down**
  dispatch for the same input event that triggered the outside-press observer.
- `disableOutsidePointerEvents` must also prevent underlay hover/move/click interactions while the
  overlay remains open.
  - Wheel events are allowed to route to underlay scrollables by default (editor ergonomics),
    matching GPUI's `BlockMouseExceptScroll` behavior.

To express that outcome in Fret without turning menus into full modal barriers, the overlay
substrate supports an additional per-overlay flag:

- `disable_outside_pointer_events` (component/policy level; see `OverlayRequest` in
  `ecosystem/fret-ui-kit`)

Runtime mechanism:

- While `disable_outside_pointer_events=true` and the overlay is open, the overlay controller sets
  `pointer_occlusion = BlockMouseExceptScroll` on the overlay layer, causing mouse hit-test
  dispatch to ignore targets in underlay layers (except wheel, respecting ADR 0011 scoping).
- Outside-press dismissal still uses the observer pass described above (the overlay can close on
  outside press even though the underlay is inert for non-scroll pointer interactions).

This ADR intentionally does **not** define the accessibility / "hide others" outcome for menus.
That is handled separately by the semantics/a11y architecture ADRs.

### Dismissable branches (Radix `DismissableLayerBranch`)

Some disjoint subtrees should be treated as “inside” an overlay for dismissal purposes even though
they are not descendants of the overlay root (e.g. a trigger button rendered in the underlay).

To support this Radix-aligned outcome, non-modal layers may provide a set of “branch” roots that
the runtime treats as inside for the outside-press observer pass:

- if the hit-tested target is inside any registered branch subtree, the observer pass does not
  dispatch an outside-press event for that overlay layer (and does not dismiss anything under it).

This preserves the click-through default: the normal hit-tested dispatch still runs unless the
layer both dispatches an outside-press observer event and is configured to consume outside pointer
down events.

#### Implicit trigger branch (recommended policy default)

For common toggle-triggered overlays (Popover, DropdownMenu, ContextMenu), the trigger element is
often rendered outside the overlay subtree. If the trigger is *not* treated as a branch, a trigger
click while the overlay is open can:

- first close the overlay via the outside-press observer pass, then
- immediately re-open it when the trigger toggles the open model on activate.

Therefore, component-layer overlay policies should treat the trigger as an implicit
`DismissableLayerBranch` by default (in addition to any explicit branches).

Note: for non-click-through overlays that both consume outside pointer-down events and disable
outside pointer interactions (menu-like `modal=true` outcomes), treating the trigger as an implicit
branch can prevent trigger presses from dismissing the overlay, because the trigger itself may be
pointer-occluded. In that case, policy may choose to *not* treat the trigger as a branch.

### Presence and close transitions (click-through correctness)

Non-modal overlays commonly animate out (opacity / scale / slide) while remaining mounted.

During this close transition window, the overlay may be:

- `present = true` (still painted), and
- `open = false` (no longer interactive; close has been requested).

To preserve correctness regardless of click-through vs consume policy:

- A non-modal overlay that is `present=true` but `open=false` must be **pointer-transparent**.
  - It must not become the hit-tested target.
  - It must not receive pointer-down-outside observer events (and therefore must not consume
    outside pointer-down events) because it is no longer dismissable.

This ensures underlay widgets can be clicked immediately while a fading surface finishes its out
transition.

### Focus restoration

When a non-modal overlay is closed due to click-through outside press, focus may already move to a
different widget as part of the same interaction. Therefore:

- Overlay focus restoration must be conditional: restore previous focus **only if** focus is still
  inside the closing overlay (or is missing), otherwise do not override the new focus target.

This rule is implemented by `OverlayPortal::hide` and applies to all overlays.

## Consequences

- `Popover` and `PopoverSurface` are installed as **non-modal** overlays.
- Menu-like overlays (DropdownMenu / ContextMenu / Menubar / Select) are non-modal overlays that
  typically opt into consuming outside pointer-down events to avoid underlay activation on dismiss.
- Modal widgets (command palette, dialog, sheet) continue to use barrier-backed layers.
- Components can implement outside press dismissal purely in the component layer using the
  existing event + bounds logic, without requiring modal barriers.

## Policy defaults (shadcn mapping)

This ADR defines the **mechanism**. Default behaviors are chosen in the component layer.

The current shadcn-aligned defaults in this repo:

| Component | Overlay kind | `blocks_underlay_input` | Outside-press observer | Consume outside pointer-down | Disable outside pointer events | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| `Popover` | Non-modal | `false` | Yes | No (click-through) | No | Outside press closes; underlay click can focus/activate |
| `Combobox` | Non-modal | `false` | Yes | No (click-through) | No | Popover + Command recipe; outside press closes; underlay click can focus/activate |
| `HoverCard` | Hover overlay | `false` | No | No | No | Driven by hover intent; click-through (no outside-press dismissal) |
| `Tooltip` | Tooltip overlay | `false` | No | No | No | Pointer-move observed; click-through |
| `DropdownMenu` | Non-modal menu | `false` | Yes | Yes (non-click-through) | Yes (Radix `modal=true`) | Outside press closes without activating underlay; underlay pointer is inert while open |
| `ContextMenu` | Non-modal menu | `false` | Yes | Yes (non-click-through) | Yes (Radix `modal=true`) | Same as dropdown menu; open model is component-owned |
| `Menubar` | Non-modal menu | `false` | Yes | No (click-through) | No (Radix `modal=false`) | Outside press closes and allows underlay activation |
| `Select` | Non-modal listbox-like menu | `false` | Yes | Yes (non-click-through) | Yes (Radix `modal=true`) | Outside press closes without activating underlay; underlay pointer is inert while open |
| `Dialog` / `Sheet` | Modal | `true` | N/A | N/A | N/A | Barrier-backed; underlay is inert while present |

## References

- Radix UI Primitives: <https://github.com/radix-ui/primitives> (pinned locally; see `docs/repo-ref.md`)
- Shadcn recipes: `repo-ref/ui`
- Overlay placement contract: `docs/adr/0064-overlay-placement-contract.md`
- Overlay policy architecture: `docs/adr/0067-overlay-policy-architecture-dismissal-focus-portal.md`
- Zed/GPUI (non-normative):
  - overlay-style “disable underlay pointer interactions” implemented via hitbox occlusion
    primitives, with a dedicated mode that still allows scroll to route to underlay scrollables:
    `repo-ref/zed/crates/gpui/src/window.rs` (`HitboxBehavior::{BlockMouse, BlockMouseExceptScroll}`)
