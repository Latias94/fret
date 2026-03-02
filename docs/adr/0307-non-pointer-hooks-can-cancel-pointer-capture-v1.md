# ADR 0307: Non-pointer Hooks Can Cancel Pointer Capture (v1)

Status: Proposed

## Context

Pointer capture is a mechanism-level tool used by editor-grade interactions (canvas panning, marquee
selection, resizing, dragging, docking drags, etc.). Today, declarative pointer hooks can request
and release pointer capture via `UiPointerActionHost`:

- `capture_pointer()`
- `release_pointer_capture()`

However, **non-pointer** action hooks (notably keyboard hooks via `OnKeyDown`) are only given a
`UiFocusActionHost`, which does not expose any capture-related operation.

This creates a contract gap:

- A surface can correctly clear its local interaction state on `Escape`,
- but cannot release an active pointer capture when the pointer stream is currently captured,
  causing “stuck drag” behavior until the next pointer-driven cancellation path runs.

This gap is currently observable in the `fret-node` declarative paint-only node graph workstream
(`docs/workstreams/fret-node-declarative-fearless-refactor-v1/todo.md`), where Escape cancel
semantics are desired for editor-grade parity.

Related contracts and references:

- ADR 0020 (focus + command routing; pointer capture routing)
- ADR 0049 (viewport tools; Cancel semantics)
- ADR 0069 (outside press; capture must not be changed implicitly by unrelated policy)

## Decision

Introduce an explicit mechanism for **non-pointer** hooks to request cancellation of pointer
capture in a window.

### D1 — Add `cancel_pointer_capture_in_window` to `UiFocusActionHost`

Extend the object-safe host surface for non-pointer hooks:

- Add a new method (exact naming TBD, but concept is fixed):
  - `fn cancel_pointer_capture_in_window(&mut self, window: AppWindowId);`

Semantics:

- The UI runtime must clear any active pointer captures for the specified `window`.
- This is a **mechanism** operation: it does not decide *when* to cancel; it enables ecosystem/app
  policy code to implement editor-grade “Escape cancels drag” behavior deterministically.

### D2 — Cancellation is window-scoped (not element-scoped)

Non-pointer hooks do not have a `PointerId`, and “current pointer” is not a stable concept for
keyboard dispatch.

Therefore, the contracted operation is window-scoped:

- cancel all active pointer captures in `window`.

This matches common editor UX expectations for Escape cancel and avoids exposing hidden state such
as “the last pointer id that captured”.

### D3 — Optional: deliver `PointerCancel` to affected captures

To support cleanup paths that are currently wired to `PointerCancel` (notably retained widget
state machines), the runtime may deliver a synthetic cancel to the previously captured targets as a
best-effort cleanup signal.

If implemented:

- The cancel delivery must be deterministic.
- It must not violate the capture arbitration invariants from ADR 0020 / ADR 0069.

If not implemented in v1:

- Ecosystem/app policy must clear its own interaction state when requesting cancellation.

## Consequences

Pros:

- Enables editor-grade Escape cancel behavior in declarative-first surfaces without retained APIs.
- Keeps the mechanism/policy boundary clean: the runtime exposes a capability; ecosystems decide
  when to use it.
- Reduces pressure to keep retained-only escape hatches solely for pointer capture release.

Cons / risks:

- Window-scoped cancellation is a sharp tool; misuse can cancel unrelated captures (multi-touch).
- If synthetic `PointerCancel` is delivered, it increases dispatch complexity and must be validated
  carefully to avoid reentrancy or ordering bugs.

## Alternatives considered

1. **Expose `UiPointerActionHost` to key hooks**
   - Rejected: key hooks do not have a stable `PointerId`; reusing `release_pointer_capture()` would
     be ambiguous and could encode hidden “current pointer” state.

2. **Make Escape automatically cancel capture in the runtime**
   - Rejected: this is policy, not mechanism. Overlays/docking/other systems may want different
     precedence rules (ADR 0072).

3. **Keep retained-only cancellation paths**
   - Rejected: this blocks declarative-first adoption and makes `fret-node`’s ecosystem surfaces
     leak retained constraints.

## Rollout plan (suggested)

1. Implement the host method in the declarative runtime key hook host.
2. Add a focused diagnostics gate proving:
   - start a captured drag,
   - press Escape,
   - assert `pointer_capture_active=false` and the drag state is cleared.
3. Update the `fret-node` workstream contract gap log and remove the “cannot release capture from
   key hooks” limitation once aligned.

