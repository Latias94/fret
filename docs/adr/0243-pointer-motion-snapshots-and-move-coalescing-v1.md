# ADR 0243: Pointer Motion Snapshots and Move Coalescing (v1)

Status: Proposed

## Context

Pointer-follow visuals (lens/parallax cards, magnetic buttons, hover highlights, dock previews)
need more than “a stream of move events”:

- they often want a *current* pointer snapshot available during paint,
- they often want a velocity estimate for easing and inertial feel,
- they must remain correct under `RenderTransform` (ADR 0082) and the coordinate rules in ADR 0238.

Runners and platforms also differ in pointer sampling:

- some backends coalesce pointer moves,
- some deliver high-frequency samples, others throttle aggressively,
- web platforms may deliver move events at unpredictable cadence.

If we don’t define a mechanism-level “pointer motion snapshot” seam, ecosystems will repeatedly
recreate their own tracking, producing inconsistent behavior and hard-to-debug drift.

Related ADRs:

- RenderTransform coordinate semantics: `docs/adr/0082-render-transform-hit-testing.md`
- Pointer coordinate spaces: `docs/adr/0238-pointer-coordinate-spaces-and-element-local-mapping-v1.md`
- Scheduling / animation frames: `docs/adr/0034-timers-animation-and-redraw-scheduling.md`
- Frame clock: `docs/adr/0240-frame-clock-and-reduced-motion-gates-v1.md`

## Decision

### D1 — `fret-ui` maintains a best-effort per-pointer motion snapshot

The UI runtime maintains a per-window, per-pointer snapshot:

- last known window-logical position,
- last update frame id (or monotonic timestamp if available),
- best-effort delta and velocity estimates.

This snapshot is updated from incoming pointer events and is best-effort by design.

### D2 — Widgets can read pointer motion snapshots in any pass (non-reactive)

Widget contexts (e.g. `EventCx`, `PaintCx`) may read a pointer motion snapshot:

- `pointer_position_window(pointer_id) -> Option<Point>`
- `pointer_velocity_window(pointer_id) -> Option<Point>` (logical px / second, best-effort)

Rules:

- These reads MUST NOT participate in view-cache dependency tracking.
- If a widget wants time-driven pointer-follow animation, it must request animation frames (ADR 0034) and advance its own state using the frame clock (ADR 0240).

### D3 — Element-local snapshot mapping is derived (transform-aware)

`fret-ui` SHOULD provide derived helpers that map the window snapshot into the widget’s element-local
space, using the same inverse-transform traversal rules as event dispatch (ADR 0082 / ADR 0238):

- `pointer_position_local(pointer_id) -> Option<Point>`
- `pointer_velocity_local(pointer_id) -> Option<Point>`

### D4 — Move coalescing is allowed; semantics remain “latest position wins”

The runner may coalesce or drop intermediate move samples.

Contract:

- `PointerEvent::Move.position` is always the latest known window-logical position at the time the
  event is emitted.
- Velocity/delta are best-effort; widgets must handle `None` or large deltas robustly.

## Consequences

- Pointer-follow components can implement consistent visuals without rewriting tracking logic.
- The framework stays portable: we don’t require high-frequency raw sampling to achieve good UX.

## Non-goals

- This ADR does not define a gesture recognition layer (drag/pan/fling) as a framework contract.
- This ADR does not guarantee “perfect” velocity; it is a best-effort UX affordance.

