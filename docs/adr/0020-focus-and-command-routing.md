# ADR 0020: Focus, Command Routing, and Input Priority


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Zed: https://github.com/zed-industries/zed

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
Status: Accepted

## Context

Fret targets editor-class workflows with:

- multi-root UI composition (base UI + overlays + popups + modals),
- docking and multiple windows,
- engine viewports embedded in panels,
- heavy keyboard usage (global shortcuts + text editing + viewport controls).

If focus and command routing semantics are not defined early, later additions (menus, modals, code editor,
viewport input forwarding) tend to force a rewrite of the event system.

References:

- Multi-root overlays and modal blocking:
  - `docs/adr/0011-overlays-and-multi-root.md`
- Keyboard/IME split (shortcuts vs text):
  - `docs/adr/0012-keyboard-ime-and-text-input.md`
  - `docs/adr/0018-key-codes-and-shortcuts.md`
- App effects queue (avoid reentrancy and scattered side effects):
  - `docs/adr/0001-app-effects.md`

## Decision

### 1) Separate “input events” from “commands”

Fret treats low-level input (`Event`) and high-level intent (`CommandId`) as separate layers:

- input events are routed to widgets (focus/capture/hit-test rules),
- commands are dispatched through a command router that is focus-aware and scope-aware.

This avoids coupling key binding logic directly to widget implementations.

### 2) Focus model is window-local with overlay-aware z-order

Each OS window has an independent focus state:

- `focused_node`: the node that receives keyboard events by default,
- `captured_node`: optional pointer capture target (authoritative for pointer events),
- `active_modal_root`: optional modal layer that can block underlying layers.

Multi-root composition (ADR 0011) defines root z-order. Hit-testing for pointer events traverses
roots from top-most to bottom-most.

### 3) Input priority rules (canonical)

#### Pointer events

1. If pointer capture is active, the captured node receives pointer events.
2. Else hit-test top-most root first; the deepest hit node becomes the target.
3. Events bubble to parents unless stopped.

#### Keyboard + text events

Keyboard input is processed in this order:

1. **Modal gate**: if a modal root is active, only that root (and its descendants) can receive keyboard input,
   unless a specific command is marked “global override” (e.g. emergency close).
2. **Shortcut resolution**: `KeyDown` can be matched against the keymap to produce a `CommandId`.
   - Key repeat behavior is explicit: repeated `KeyDown` may re-dispatch a command only if the command is marked
     `repeatable` (e.g. text navigation/deletion). Otherwise repeated keydowns fall through to widgets.
3. **Text input**: `TextInput` and `ImeEvent` are routed to the focused text-editing widget (if any).
4. **Fallback**: unhandled `KeyDown/KeyUp` events are delivered to the focused node as normal widget events.

This ensures text editing remains correct and does not depend on key press interpretation.

### 4) Command routing is scope-aware

Commands are routed through the following scopes, in order:

1. **Focused widget scope**: the focused node (and bubbling chain) can handle the command.
2. **Window scope**: window-level handlers (menus/toolbars/dock manager).
3. **App scope**: global handlers (project actions, command palette).

Commands must be allowed to declare their intended scope, so that conflicts are resolvable and discoverable.

### 5) Command dispatch is effect-driven (avoid reentrancy)

To avoid reentrancy bugs and borrow conflicts, command dispatch is serialized through the app loop:

- widgets request commands by enqueueing an effect (conceptually `Effect::Command(CommandId)`),
- the runner drains effects at a synchronization point and invokes command handlers in a bounded loop.

This mirrors the “effects queue” approach used for platform actions.

### 6) Viewport focus and engine input forwarding

Engine viewports are treated as focusable targets:

- when a viewport panel is focused, relevant input (pointer and optionally keyboard) may be forwarded to the engine,
  via a data-only event contract (viewport target + uv/px mapping + modifiers).

Forwarding rules must respect modal gating (e.g. a modal dialog prevents viewport camera controls).

## Consequences

- Multi-root overlays behave predictably: popups and modals capture input without fragile hacks.
- Global shortcuts and text editing can coexist without key interpretation bugs.
- Docking and viewport input forwarding remain compatible with the same focus model.
- Command dispatch remains non-reentrant and debuggable via the effects drain loop.

## Future Work

- Specify the concrete command handler API and how handlers are registered (app/window/widget scopes).
- Define a canonical keymap file format and conflict resolution strategy (see ADR 0014 + ADR 0018).
- Define focus traversal (tab order, focus scopes) across roots and widgets.

## Notes (Zed/GPUI reference, non-normative)

- GPUI builds a per-frame dispatch tree with explicit capture/bubble phases, and treats “commands”
  as typed actions routed via the dispatch path rather than raw key events:
  `repo-ref/zed/crates/gpui/src/key_dispatch.rs` (`DispatchTree`),
  `repo-ref/zed/crates/gpui/src/window.rs` (`dispatch_event`, `dispatch_key_down_up_event`,
  `is_action_available`).
- GPUI also tracks pending multi-stroke bindings (“pending input”) on the window and exposes it for
  UX/diagnostics (`repo-ref/zed/crates/gpui/src/window.rs`), which aligns with Fret’s goal of
  keeping shortcut resolution explicit and debuggable.
