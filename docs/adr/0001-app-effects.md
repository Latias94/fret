# ADR 0001: App Effects Queue

Status: Accepted

## Context

Fret targets an editor-grade UI with multiple windows (tear-off docking), and long-lived app state shared across many widgets. In Rust, we want to avoid:

- widget code directly performing platform actions (window create/close, clipboard, etc.),
- borrow conflicts between `&mut App` and `&mut` state,
- scattered “who triggers redraw” logic across layers.

## Decision

Introduce an `App`-owned effects queue:

- UI/widgets enqueue side effects as data (`Effect`).
- The platform/runner drains effects at defined synchronization points and performs OS operations.
- Redraw requests are also collected in `App` and surfaced via `flush_effects()` to keep a single consumption point.

### Invariants

- Widgets must not call platform APIs directly.
- Effects are best-effort and should be safe to drop when the target window/model no longer exists.
- The runner may deduplicate effects (e.g. multiple redraws for the same window).

### Initial effect set

- `Effect::Redraw(AppWindowId)`
- `Effect::Window(WindowRequest)`
- `Effect::Command(CommandId)` (reserved; may become a separate command bus later)

## Consequences

- Multi-window features (docking tear-off) become predictable: dock emits a `WindowRequest::Create(...)`, runner creates the OS window, and then dock graph is updated.
- Borrow scopes stay small: widgets mutate models and enqueue effects without needing to hold platform objects.

## Future Work

- Add `App::flush_effects()` semantics for “effects generated while draining effects” (loop or fixed-point).
- Add more window operations (set title, set cursor, clipboard, drag-and-drop).
- Add scheduling hooks (timers/animations) as effects or as a sibling subsystem.
