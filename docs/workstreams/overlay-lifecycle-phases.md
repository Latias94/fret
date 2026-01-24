# Overlay lifecycle phases (policy + runtime seam)

Status: living

This note defines the **overlay lifecycle phases** used by Fret overlay primitives (menus, popovers,
dialogs, etc.) and maps each phase to the current implementation seams in:

- `ecosystem/fret-ui-kit/src/window_overlays/*` (policy + orchestration)
- `crates/fret-ui/src/tree/*` (core input / outside-press arbitration)
- `crates/fret-ui/src/overlay_placement/*` (placement vocabulary/solver)

The goal is to make overlay behavior **deterministic, testable, and safe under view-cache reuse**
before deeper refactors.

## Terms

- **Request**: a per-frame declaration emitted by primitives/recipes into `WindowOverlays`.
- **`open`**: the authoritative model/state that represents user intent ("should be open").
- **`present`**: whether the overlay is currently mounted/painted (often `true` while closing for
  exit transitions).
- **`interactive`**: whether the overlay participates in hit-testing and dismissal observation.
- **Barrier**: modal underlay gating (background input/semantics suppression).

Ownership:

- `open` is **component/policy-owned** (ecosystem primitives). Core/runtime must not mutate `open`
  as an implicit side effect of input arbitration (except when an explicit dismissal policy asks
  for it).
- `present` is **presence-owned** (ecosystem presence primitive / policy). Core/runtime consumes it
  to decide whether a layer exists and whether modal barriers remain active during close
  transitions.

## Lifecycle state machine

Overlays conceptually move through a small state machine:

1. **Requested**
   - A request exists for an overlay id (this frame or synthesized from cached declarations).
   - Required data: id, trigger/anchor, placement options, policy hooks.
2. **Mounted**
   - The overlay root exists as a UI layer (has a `UiLayerId`) and is part of the layer stack.
   - For request types with explicit presence, this corresponds to `present=true`.
3. **Interactive**
   - The overlay is open and participates in hit-testing and dismissal observation.
   - For request types with explicit open state, this corresponds to `open=true`.
4. **Dismissing (closing transition)**
   - The overlay remains mounted/painted for an exit transition, but interactivity is reduced to
     avoid click-through or double-dispatch.
   - The key contract difference by overlay kind:
     - **Modal**: the barrier remains active while `present=true` (prevents underlay click-through
       during close animations).
     - **Non-modal dismissible**: the overlay becomes click-through during close transitions
       (`present=true` but `open=false`) so it does not steal input or observe outside presses.
5. **Unmounted**
   - The overlay no longer contributes a layer (`present=false` or the request disappears).

## Current implementation mapping

### Modal + dismissible popover (authoritative `open` + `present`)

Modal and popover requests explicitly carry:

- `open: Model<bool>`
- `present: bool`

Implementation anchors:

- Request types: `ecosystem/fret-ui-kit/src/window_overlays/requests.rs`
- Policy/render orchestration: `ecosystem/fret-ui-kit/src/window_overlays/render.rs`
- Layer knobs (present vs interactive semantics): `ecosystem/fret-ui-kit/src/window_overlays/state.rs`

View-cache seam:

- Cached request declarations are used as an optimization so `open=true` overlays can remain present
  even when view caching skips rerendering the producer subtree.
- Close transitions are intentionally treated as "instant" under producer suppression (no request
  producer running): if `open` flips false, the overlay disappears as soon as we stop synthesizing
  a request.

### Hover overlays + tooltips (per-frame only, no presence contract yet)

`HoverOverlayRequest` and `TooltipRequest` currently do not include authoritative `open/present`.
They are treated as **strict per-frame requests** to avoid “ghost overlays” that stay alive after
their producer subtree stops requesting them.

This is a known gap for view-cache reuse: if the producer subtree is skipped for a frame, the
request disappears for that frame and the overlay may unmount.

Follow-up item: track a future "authoritative presence" for hover/tooltip separately (see workstream
tracker `OVERLAY-life-002`).

## Contract checkpoints (tests)

The lifecycle contract is enforced by tests at two layers:

- `ecosystem/fret-ui-kit/src/window_overlays/tests.rs`
  - Cached request synthesis for view-cache: `cached_modal_request_is_synthesized_when_open_without_rerender`,
    `cached_popover_request_is_synthesized_when_open_without_rerender`
  - Modal close transition keeps barrier active: `modal_is_hit_testable_while_closing_but_still_present`
  - Non-modal close transition becomes click-through: `non_modal_overlay_does_not_request_outside_press_observer_while_closing`
- `crates/fret-ui/src/tree/tests/outside_press.rs`
  - Outside-press observer dispatch semantics (topmost dismissible, branch exemptions).
- `crates/fret-ui/src/tree/tests/window_input_arbitration_snapshot.rs`
  - Modal barrier arbitration scoping (capture/occlusion ordering).

## Diagnostics / scripted regressions

The scripted harness (UI Gallery) is intended to validate lifecycle invariants under:

- cached vs uncached runs (`fretboard diag matrix ui-gallery`)
- portal placement + window clamping (`bounds_within_window` predicate)

Entry points:

- `docs/ui-diagnostics-and-scripted-tests.md`
- `apps/fretboard/src/diag.rs` (suite + matrix runner)

