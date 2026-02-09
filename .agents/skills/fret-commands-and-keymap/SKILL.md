---
name: fret-commands-and-keymap
description: Commands, `keymap.json`, menus, and command palette contracts in Fret. Use when adding new commands, keybindings, menubar entries, `when` gating, or debugging focus-aware shortcut routing.
---

# Fret commands and keymaps

## When to use

- You’re adding a new editor action (“toggle panel”, “focus search”, “frame selection”, “close tab”).
- You’re wiring up a command palette or menu item and want one source of truth.
- A shortcut fires in the wrong place (e.g. triggers while typing in a text field).
- You need platform-specific bindings (macOS `meta` vs Windows/Linux `ctrl`) or multi-stroke sequences.

## Mental model (how Fret wants you to think)

- **Commands are stable IDs + metadata** (`CommandId` + `CommandMeta`).
- **Keymaps resolve physical keys** (layout-independent) into `CommandId`s.
- **Routing is focus-aware** (ADR 0020): widget/window/app scopes, bubbling, and context gating via `when`.
- **UI surfaces derive from the same model**: menu labels, palette search keywords, default bindings, enable/disable.

## Quick start

### 1) Register a command (metadata)

```rust
use fret_runtime::{CommandId, CommandMeta, CommandRegistry, CommandScope};

pub fn register_commands(commands: &mut CommandRegistry) {
    commands.register(
        CommandId::new("app.command_palette"),
        CommandMeta::new("Command Palette")
            .with_scope(CommandScope::Window)
            .with_keywords(["palette", "cmdk"]),
    );
}
```

### 2) Bind keys in `keymap.json` (recommended; layered overrides)

```json
{
  "keymap_version": 2,
  "bindings": [
    {
      "command": "app.command_palette",
      "when": "window.is_focused == true",
      "keys": [{ "mods": ["ctrl"], "key": "KeyP" }]
    }
  ]
}
```

Notes:

- Prefer `when` gates like `focus.is_text_input == false` for global shortcuts that must not interfere with text editing.
- Use `platform` filters (`macos` / `windows` / `linux` / `web`) for conventional bindings.

## Workflow

1. Define a stable `CommandId` and register `CommandMeta` (scope + keywords).
2. Add default bindings in `keymap.json` with explicit `when` gating.
3. Verify focus-aware routing with a resolution/unit test (and a diag script for end-to-end behavior).
4. Treat command IDs and `when` expressions as contracts (avoid breaking renames).

## Common pitfalls

- **Character-based shortcuts** (should be physical key codes; ADR 0018).
- **Missing `when` gates**, causing global shortcuts to fire inside text inputs / IME composition.
- **Duplicated enable/disable logic** inside widgets instead of one `when` expression + command metadata.
- **Unstable command IDs**: renaming breaks keymaps, menus, and scripts; treat IDs as contracts.

## Regression gates (recommended defaults)

- Add a `Keymap::from_bytes(...)` parse test for any non-trivial `when` expression or multi-stroke binding.
- Add a `resolve(...)` test with an `InputContext` that matches the intended focus state.
- Add a `tools/diag-scripts/*.json` scenario that presses the shortcut and asserts a stable `test_id` outcome
  (e.g. “palette overlay opened”, “menu item toggled”).

## References (start here)

- ADRs:
  - `docs/adr/0018-key-codes-and-shortcuts.md`
  - `docs/adr/0020-focus-and-command-routing.md`
  - `docs/adr/0021-keymap-file-format.md`
  - `docs/adr/0022-when-expressions.md`
  - `docs/adr/0023-command-metadata-menus-and-palette.md`
  - `docs/adr/0043-shortcut-arbitration-pending-bindings-and-altgr.md`
- Code entry points:
  - `crates/fret-runtime/src/commands.rs` (`CommandRegistry`, `CommandMeta`)
  - `crates/fret-runtime/src/keymap.rs` (`Keymap`, parsing, resolution, conflict detection)
  - `crates/fret-runtime/src/menu.rs` (data-only menu model)
  - `crates/fret-app/src/keymap.rs` (layered keymap loading + reverse lookup)
  - `ecosystem/fret-bootstrap/src/ui_app_driver.rs` (config watcher + hot reload)

## Evidence anchors

- ADRs: `docs/adr/0020-focus-and-command-routing.md`, `docs/adr/0021-keymap-file-format.md`, `docs/adr/0022-when-expressions.md`
- Parsing/resolution: `crates/fret-runtime/src/keymap.rs`
- Command metadata + routing: `crates/fret-runtime/src/commands.rs`

## Related skills

- `fret-text-input-and-ime` (focus gating so shortcuts don’t break text/IME)
- `fret-diag-workflow` (scripted repros for “shortcut does the right thing”)
