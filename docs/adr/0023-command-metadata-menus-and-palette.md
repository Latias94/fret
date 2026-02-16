# ADR 0023: Command Metadata, Menus, and Command Palette


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Zed: https://github.com/zed-industries/zed

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
Status: Accepted

## Context

A game engine editor requires a large command surface:

- file/project actions,
- editing tools (gizmos, snapping, selection modes),
- view/layout actions (docking, tabs, windows),
- debug/diagnostics (profilers, render doc hooks),
- plugin-contributed commands.

Without a unified command model, editors tend to accumulate:

- ad-hoc keyboard handlers,
- inconsistent menu structures,
- non-discoverable features,
- duplicated “enable/disable” logic scattered across widgets.

References:

- Focus + command routing semantics:
  - `docs/adr/0020-focus-and-command-routing.md`
- Keymap file format:
  - `docs/adr/0021-keymap-file-format.md`
- `when` expressions:
  - `docs/adr/0022-when-expressions.md`
- Zed ownership + app-owned entities and effects-driven update loop (design inspiration):
  - https://zed.dev/blog/gpui-ownership
- Zed/GPUI command/action system (non-normative code anchors):
  - typed actions and action registry: `repo-ref/zed/crates/gpui/src/action.rs`
  - dispatch + keymap resolution pipeline: `repo-ref/zed/crates/gpui/src/key_dispatch.rs`
  - command palette surface: `repo-ref/zed/crates/command_palette`

## Decision

### 1) Commands are stable IDs + structured metadata

Each command has a stable ID:

- `CommandId` is a string identifier, namespaced (e.g. `app.command_palette`, `dock.close_tab`, `viewport.frame_selection`).

Commands also have structured metadata used by UI surfaces:

- `title` (display name)
- `description` (optional)
- `category` (e.g. `File`, `Edit`, `View`, `Dock`, `Viewport`, `Debug`)
- `keywords` (search terms for palette)
- `default_keybindings` (optional; expressed using the same shape as ADR 0021)
- `when` (optional gating expression; ADR 0022)
- `scope` (widget/window/app; ADR 0020)
- `repeatable` (optional; allows key-repeat to re-dispatch the command, e.g. text editing/navigation)

Implementation anchors (current workspace):

- Command registry + `CommandMeta` shape: `crates/fret-app/src/app.rs`
- Keymap reverse lookup for UI display (best-effort): `crates/fret-app/src/keymap.rs`
- Minimal menu model types (data-only): `crates/fret-app/src/menu.rs`
- Command palette UI surface:
  - The previous retained-widget implementation lived in `fret-ui-kit`, but the public component surface is
    now declarative-only (see ADR 0066 / migration notes in `docs/shadcn-declarative-progress.md`).
  - The planned declarative surface is `fret-ui-shadcn::command` backed by reusable infra in
    `fret-ui-kit` (overlay policy + headless filtering/navigation).

### 2) Menus and toolbars are derived from command metadata

Menu items reference commands; they do not duplicate logic.

Menu entries define:

- `path` (e.g. `["View", "Dock", "Close Tab"]`)
- `command`
- optional `when` override (rare; prefer command-level gating)

### 3) Command palette is the canonical discovery surface

The command palette lists all commands that are:

- registered,
- discoverable (not hidden),
- permitted by `when` in the current context.

Palette ranking uses:

- title + keywords,
- recency/frequency (future),
- category boosts (optional).

### 4) Registration is app-owned, plugin-safe

Commands are registered in an app-owned registry, allowing:

- core commands,
- plugin commands,
- project-specific commands.

Plugins can register commands but must not directly access platform APIs or renderer internals (ADR 0016).

### 5) Dispatch is effect-driven and scope-aware

Key presses resolve to `CommandId` via keymap (ADR 0021) and then dispatch follows ADR 0020.

Command dispatch is serialized via the app loop (effects queue) to avoid reentrancy and to keep a single
place for debugging and logging.

## Consequences

- Editor functionality becomes discoverable and consistent (menus/palette/shortcuts share one model).
- Enable/disable logic is centralized through `when` and scope routing.
- Plugins integrate cleanly: they contribute commands without entangling UI or renderer.

## Future Work

- Decide how to represent parameterized commands (e.g. “Open Recent: <path>”).
- Define localization strategy for titles/descriptions (if needed).
- Add UI for key binding conflicts and command enablement debugging.
