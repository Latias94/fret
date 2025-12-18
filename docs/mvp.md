# MVP Plan (Active, Short-Horizon)

This document is the **current execution plan** that complements `docs/roadmap.md`.

Completed stage definitions are archived in `docs/mvp-archive.md` to keep this file actionable.

## Current Workspace Status

- MVP 0–6: done (see `docs/mvp-archive.md`)
  - MVP 5: Text MVP landed (single-line input + IME cursor-area loop)
  - MVP 6: Commands + keymap MVP landed (bind/route/persist; `when` gating)
- MVP 7: MVP done in demo (command palette overlay; shortcut display; menu model types added)

## MVP 7 — Command UI Surfaces (Palette + Menu Skeleton)

Goal: validate ADR 0023 end-to-end by exposing commands as first-class UI surfaces, not only shortcuts.

**Scope**

- Command palette overlay:
  - open/close via a command (e.g. `command_palette.toggle`)
  - search/filter and execute commands
  - show the resolved shortcut (if bound) next to each command
- Minimal menu model (data-only) that can later drive a menubar/context menus:
  - a menu is just a list of command ids + separators + optional `when`
  - rendering can be deferred; the goal is to lock the contract and wiring
- Overlay correctness:
  - palette is a multi-root overlay (ADR 0011)
  - focus enters palette while open and returns to the previous focus on close (ADR 0020)

**Non-goals**

- Perfect fuzzy matching and scoring.
- Full native menubar integration (macOS global menu, etc.).

**Definition of Done**

- `fret-demo` can open a command palette overlay and run a command by keyboard.
- Palette obeys modal gates: if a modal is open, palette behavior is deterministic (either blocked or becomes the modal itself; pick one and lock it).
- Palette lists commands with titles and (if bound) the active shortcut.

References:

- `docs/adr/0011-overlays-and-multi-root.md`
- `docs/adr/0020-focus-and-command-routing.md`
- `docs/adr/0021-keymap-file-format.md`
- `docs/adr/0022-when-expressions.md`
- `docs/adr/0023-command-metadata-menus-and-palette.md`

## MVP 8 — Focus Traversal + Focus Scopes

Goal: make large editor UIs usable with keyboard navigation, and lock focus-scope semantics before widget count explodes.

**Scope**

- Implement focus traversal (`Tab` / `Shift+Tab`) across a focus scope.
- Define focus scopes for:
  - modal overlays (trap focus)
  - dock panels (local traversal)
  - command palette (single scope root)
- Expose traversal as commands (`focus.next`, `focus.previous`) so keymaps can override.

**Definition of Done**

- `fret-demo` can traverse focus between at least:
  - a dock tab bar (or a few focusable buttons),
  - a `TextInput`,
  - a modal overlay control.
- Focus traversal never escapes a modal overlay.

References:

- `docs/adr/0011-overlays-and-multi-root.md`
- `docs/adr/0020-focus-and-command-routing.md`

## MVP 9 — Clipboard P0 (Text)

Goal: land the platform boundary for clipboard and make `TextInput` production-usable.

**Scope**

- Platform clipboard service (text only) behind the effect boundary (ADR 0003 / ADR 0001).
- Core text commands:
  - `text.copy`, `text.cut`, `text.paste`, `text.select_all`
- `TextInput` integration and keymap defaults (platform conventions).

**Definition of Done**

- `fret-demo` `TextInput` supports copy/cut/paste/select-all across platforms.
- Clipboard plumbing is effect-driven (UI has no direct platform handle).

References:

- `docs/adr/0001-app-effects.md`
- `docs/adr/0003-platform-boundary.md`
- `docs/adr/0012-keyboard-ime-and-text-input.md`
- `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`

## Parking Lot (Explicitly Deferred)

- External OS drag & drop hover semantics on macOS/winit (see `docs/known-issues.md`).
- Code-editor-grade text widgets (virtualized layout + incremental shaping caches).

## Code Anchors

- Command registry + effects: `crates/fret-app/src/app.rs`
- Keymap + `when`: `crates/fret-app/src/keymap.rs`, `crates/fret-app/src/when_expr.rs`
- Command palette demo widget: `crates/fret-demo/src/command_palette.rs`
- Focus + routing: `crates/fret-ui/src/tree.rs`
- Overlay/multi-root: `crates/fret-ui/src/tree.rs`, `docs/adr/0011-overlays-and-multi-root.md`
- Desktop runner: `crates/fret-runner-winit-wgpu/src/runner.rs`
