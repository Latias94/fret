# ADR 0168: OS Menubar Integration via `Effect::SetMenuBar`

Status: Proposed

## Context

Fret targets editor-grade desktop UX (Unity/Unreal/Godot-style workflows). For desktop apps, an OS
menu bar is a first-class discovery surface that:

- Matches user expectations for “editor” software on Windows/macOS/Linux.
- Provides predictable keyboard affordances (accelerators) and standard navigation.
- Enables user customization (reorder/hide/rename) without rewriting UI widgets.

Fret already has a data-only menu model (`fret-runtime::MenuBar`, ADR 0023) that can power:

- in-window menu rendering (cross-platform parity),
- context menus,
- command palette breadcrumbs,
- and an OS menubar mapping on native runners.

We want an integration seam that does not expand the `fret-ui` contract surface and does not force
apps to choose a particular component library.

## Decision

Introduce a platform integration effect:

`Effect::SetMenuBar { window: Option<AppWindowId>, menu_bar: MenuBar }`

Native runners MAY map this to an OS menu bar when supported. Web runners MAY ignore it.

### Window-scoped `InputContext` snapshots

Runners need a reliable `InputContext` snapshot for:

- keymap reverse lookup (shortcut labels),
- best-effort enable/disable gating via `when` (ADR 0022).

The runner must not depend on `fret-ui` internals to compute focus/modal state. Instead, the UI
runtime should publish a window-scoped snapshot that runners can read.

Decision: introduce a global, window-scoped snapshot service (data-only; `fret-runtime`) that stores
the latest `InputContext` per `AppWindowId`, updated by `fret-ui` during dispatch/paint.

Publishing requirements:

- The UI runtime MUST publish a post-dispatch snapshot after processing an input event so the
  runner can gate OS menu items without waiting for the next paint.
- The UI runtime MUST publish a post-dispatch snapshot after command routing (`dispatch_command`),
  since commands can change focus/modal state (e.g. opening a modal overlay).
- The UI runtime SHOULD also publish during paint to cover programmatic focus changes that did not
  go through an input event (best-effort).

Some enable/disable predicates are not derivable from focus/modal state (e.g. whether Undo/Redo is
available for the active document). Apps may publish additional window-scoped availability via a
separate data-only service (e.g. `WindowCommandAvailabilityService`), which the UI runtime can
merge into the published `InputContext` snapshot (v1: `edit.can_undo`, `edit.can_redo`).

### Semantics

- `window: None` sets the “default” menu bar for the application, applied to all current windows
  and reused for future windows.
- `window: Some(id)` sets the menu bar for a specific window.
  - On platforms with a single global menubar (macOS), runners MAY treat this as equivalent to
    `window: None`.

### Command dispatch

- Selecting a menu item MUST dispatch the associated `CommandId` using the normal command routing
  (ADR 0020), typically by enqueueing `Effect::Command { window: Some(id), command }`.
- The OS integration MUST NOT directly call app handlers in a re-entrant way (keep serialized
  dispatch through the effects queue).

### Shortcut display

Apps MAY allow end-user customization by loading a file-backed menu model (e.g. layered
`.fret/menubar.json` + user `menubar.json`) and emitting `Effect::SetMenuBar` with the parsed
`MenuBar`. This keeps the runner contract data-only while allowing per-project overrides.

Recommended default: support both of the following `menubar.json` styles (versioned):

- Full replace (`menus: [...]`) — write the entire menu bar explicitly.
- Patch ops (`ops: [...]`) — apply small, user-friendly edits on top of the app-provided baseline
  menu bar (rename/reorder/hide/insert).
  - Patch targets can use a menu path array: `menu: ["File", "Recent"]` applies to a submenu.
  - Item anchors support `before` / `after` as either a command id string or a numeric index.
  - For non-command items (like submenus), prefer `remove_at` / `move_at_*` with an explicit selector
    (e.g. `{"type":"submenu","title":"Recent"}`) or an index anchor.

- Menu items SHOULD display a shortcut label derived from keymap reverse lookup (ADR 0021).
- Runners and in-window menu surfaces SHOULD prefer a **stable display heuristic** over a
  fully “live focus-sensitive” label, to avoid flicker as focus moves within a window.
  - Evidence: `Keymap::display_shortcut_for_command_sequence` (stable ranking across a small set of
    default `InputContext` variants).

### Enable/disable gating

- Runners SHOULD disable menu items when:
  - `CommandMeta.when` does not match the current `InputContext` (ADR 0022), or
  - `MenuItem.when` is present and does not match.
- This is a best-effort UX hint. The actual command handler remains authoritative.
- If the runner cannot track focus/modal state reliably, it SHOULD avoid applying focus-sensitive
  gating (e.g. `focus.is_text_input`) until a window-scoped `InputContext` snapshot is available.

## Rationale

- Keeps menu authoring data-driven and command-derived (ADR 0023).
- Adds a stable seam for OS integration without pulling policy-heavy UI surfaces into `fret-ui`.
- Enables Unity-style “customizable menus” by layering menu contributions and user config without
  binding to a widget recipe library.

## Consequences

### Positive

- Native desktop apps can provide a familiar OS menubar without rewriting menu authoring.
- Shortcut labels stay consistent with user/project keymap overrides.
- The in-window menu UI becomes optional rather than mandatory.

### Negative / Trade-offs

- OS menu capabilities vary by platform (global vs per-window, system menus, roles).
- Some menu semantics are hard to standardize early (checked items, radio groups, native roles).
- Runner implementations must bridge native events safely into the effect queue.

## Implementation Notes

- Windows-first implementation uses Win32 `HMENU` + `WM_COMMAND` mapping.
- macOS implementation uses `NSMenu` and validates items via `validateMenuItem:` before menu open.
- Runners should retain the current `MenuBar` when `window: None` so new windows inherit it.

## Evidence (current implementation)

- Effect contract: `crates/fret-runtime/src/effect.rs`
- Menu model: `crates/fret-runtime/src/menu.rs`
- Window-scoped input context snapshots: `crates/fret-runtime/src/window_input_context.rs`
- Windows native menu mapping: `crates/fret-launch/src/runner/desktop/windows_menu.rs`
- macOS native menu mapping: `crates/fret-launch/src/runner/desktop/macos_menu.rs`
- Desktop runner effect handling + window inheritance: `crates/fret-launch/src/runner/desktop/mod.rs`
- Demo wiring (desktop): `apps/fret-ui-gallery/src/driver.rs`

## Future Work

- Expand the window-scoped `InputContext` snapshot to cover more UX-sensitive predicates
  (e.g. selection state, hover intent, per-modal stacks) without increasing runner/UI coupling.
- macOS roles and standard menus (Services, App menu, standard edit actions, Window menu expectations).
- Linux OS menubar integration (GTK/DBus) if/when a strong target emerges (see ADR 0169 for current
  fallback-first strategy).
- Standardize richer menu item semantics:
  - checked/radio state,
  - dynamic submenu population (e.g. “Open Recent”),
  - platform roles and system menus.
- Define a user customization layer:
  - menu contribution API (`path`-based),
  - user override schema (hide/reorder/rename),
  - hot reload when config changes.
