# MVP Plan (Active, Short-Horizon)

This document is the **current execution plan** that complements `docs/roadmap.md`.

Completed stage definitions are archived in `docs/mvp-archive.md` to keep this file actionable.

## Current Workspace Status

- MVP 0–6: done (see `docs/mvp-archive.md`)
  - MVP 5: Text MVP landed (single-line input + IME cursor-area loop)
  - MVP 6: Commands + keymap MVP landed (bind/route/persist; `when` gating)
- MVP 7: MVP done in demo (command palette overlay; shortcut display; menu model types added)
  - Keymap v2 sequences + pending bindings are prototype implemented (ADR 0043 / ADR 0021).

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

Status:

- Prototype implemented in `fret-demo` (Tab / Shift+Tab bindings, focusable traversal across multiple TextInputs).

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

Status:

- Prototype implemented in `fret-demo` (Cmd/Ctrl+A/C/X/V bindings; effect-driven clipboard read/write; paste delivered as `Event::ClipboardText`).

References:

- `docs/adr/0001-app-effects.md`
- `docs/adr/0003-platform-boundary.md`
- `docs/adr/0012-keyboard-ime-and-text-input.md`
- `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`

## MVP 10 — Text Editing Baseline (Caret + Selection + Navigation)

Goal: make `TextInput` usable enough for editor UIs by locking the state model and core editing commands early.

**Scope**

- Replace the MVP “select_all bool” with a real selection model:
  - caret position (byte index or grapheme cluster index; pick one and lock it),
  - selection range (start/end), including mouse drag selection.
- Core navigation/edit commands routed via commands (not hard-coded keys):
  - `text.move_left/right`, `text.move_word_left/right`, `text.move_home/end`,
  - `text.delete_backward/forward`, `text.delete_word_backward/forward`,
  - `text.select_*` variants (shift-modified movement).
- IME correctness:
  - inline preedit rendering in the widget (already MVP for single-line),
  - `ImeSetCursorArea` updated from the actual caret rect after layout/paint (so candidate windows track the caret).

**Definition of Done**

- `fret-demo` `TextInput` supports:
  - caret movement with arrow keys,
  - selection expansion with shift+arrows,
  - copy/cut/paste over selection,
  - IME candidate window anchored at caret (macOS/Windows).

Status:

- Prototype implemented in `fret-demo` (caret + selection; mouse drag selection; arrow/word nav + delete commands; repeatable key-repeat for text editing commands; IME cursor-area follows caret).

References:

- `docs/adr/0012-keyboard-ime-and-text-input.md`
- `docs/adr/0029-text-pipeline-and-atlas-strategy.md`
- `docs/adr/0044-text-editing-state-and-commands.md`

## MVP 11 — Text Layout Contracts (Hit Test + Caret Metrics)

Goal: lock the long-term contract for “text geometry queries” that high-end widgets require (code editor, large
documents, precise IME caret positioning) before building more components on top.

This MVP is primarily a **contract / API boundary** milestone; performance work can iterate after the contract is stable.

**Scope**

- Extend the text boundary (ADR 0006 / ADR 0029) to support:
  - hit-testing from x/y to caret position (byte offset),
  - caret/selection geometry queries (rects) for painting selection and IME cursor-area,
  - multi-line constraints (wrap + line breaks) for future editor widgets.
- Decide the lifetime model:
  - “layout object” handle (e.g. `TextLayoutId`/`TextLayout`) vs “stateless query methods”.
- Define caching expectations:
  - avoid allocating new blobs/atlas entries for measurement-only queries,
  - allow shaped-run caches and incremental atlas uploads.

**Definition of Done**

- An ADR (or ADR updates) fully specifies the geometry query API, including:
  - index representation (byte offsets at char boundaries; ADR 0044),
  - coordinate spaces (logical px; scale_factor behavior),
  - behavior for IME preedit cursor within composed text.
- A demo plan exists for validating it with:
  - a multi-line text widget (not necessarily a code editor yet),
  - accurate selection painting and caret positioning at arbitrary x/y.

References:

- `docs/adr/0006-text-system.md`
- `docs/adr/0029-text-pipeline-and-atlas-strategy.md`
- Zed/GPUI text system patterns:
  - `repo-ref/zed/crates/gpui/src/text_system.rs`

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
