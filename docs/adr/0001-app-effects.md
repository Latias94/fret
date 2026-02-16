# ADR 0001: App Effects Queue


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Zed: https://github.com/zed-industries/zed

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
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
- Redraw requests are deduplicated by `App` (a window is either dirty or not).
- The runner drains effects in a fixed-point loop (bounded) because applying one effect may enqueue more effects (e.g. `window_created` callbacks).

### Initial effect set

The minimal P0 set is:

- `Effect::Redraw(AppWindowId)`
- `Effect::Window(WindowRequest)`
- `Effect::Command(CommandId)` (reserved; may become a separate command bus later)

Current code already includes additional effects to support editor-grade UX (docking, viewports, scheduling).
Treat `crates/fret-app/src/app.rs` as the canonical source of truth for the full enum.

Examples of “non-minimal” effects already in use:

- `Effect::Dock(DockOp)` (docking emits operations, app applies them)
- `Effect::ViewportInput(ViewportInputEvent)` (engine viewport input forwarding)
- `Effect::QuitApp` (request application exit; native runners may exit their event loop)
- `Effect::HideApp` / `Effect::HideOtherApps` / `Effect::UnhideAllApps` (macOS app visibility controls)
- `Effect::RequestAnimationFrame(AppWindowId)` / `Effect::SetTimer { .. }` / `Effect::CancelTimer { .. }` (scheduling)
- `Effect::ImeAllow { .. }` / `Effect::ImeSetCursorArea { .. }` (IME enablement and candidate window positioning)
- `Effect::CursorSetIcon { .. }` (system cursor feedback, e.g. resize handles)

## Consequences

- Multi-window features (docking tear-off) become predictable: dock emits a `WindowRequest::Create(...)`, runner creates the OS window, and then dock graph is updated.
- Cross-window UX can be made deterministic without direct platform calls: UI can emit a best-effort `WindowRequest::Raise { .. }` when activating a panel in another window.
- Borrow scopes stay small: widgets mutate models and enqueue effects without needing to hold platform objects.

## Future Work

- Add additional effect types (clipboard, cursor, drag-and-drop, IME requests).
- Add more window operations (set title, set cursor, set window placement, native menu integration).
- Add scheduling hooks (timers/animations) as effects or as a sibling subsystem.

## Notes (Zed/GPUI reference, non-normative)

- GPUI also uses an explicit “effect cycle” and provides `App::defer` / `Window::defer` to schedule
  work at the end of the current cycle (avoiding re-entrancy and borrow hazards):
  `repo-ref/zed/crates/gpui/src/app.rs` (`App::defer`, `Effect::Defer`),
  `repo-ref/zed/crates/gpui/src/window.rs` (`Window::defer`).
