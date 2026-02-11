# ADR 0021: Keymap File Format (keymap.json)

Status: Accepted

## Context

Fret is an editor-grade UI framework with heavy keyboard usage:

- global shortcuts (command palette, navigation),
- text editing shortcuts (property fields, future code editor),
- viewport-focused shortcuts (camera controls, gizmos),
- platform differences (macOS Command vs Windows/Linux Control),
- plugins contributing commands and default bindings.

We already decided:

- shortcuts are matched on **physical keys** + modifiers (ADR 0018),
- input priority and command routing are scope-aware (ADR 0020),
- settings/config are organized as files (ADR 0014).

What remains is to define a stable, extensible on-disk format for key bindings.

References:

- Zed’s “files as the organizing principle” for configuration:
  - https://zed.dev/blog/settings-ui
- Zed keymap format and validation (non-normative):
  - `repo-ref/zed/crates/settings/src/keymap_file.rs`
- Fret keyboard model and focus/command routing:
  - `docs/adr/0012-keyboard-ime-and-text-input.md`
  - `docs/adr/0018-key-codes-and-shortcuts.md`
  - `docs/adr/0020-focus-and-command-routing.md`

## Decision

### 1) File name and versioning

Key bindings are stored in a versioned file:

- `keymap.json`
- top-level `keymap_version: u32` (starting at `1`)

### 2) Physical key representation

Each binding stores the key as a **physical key code token**, not a character.

The canonical representation is a string token that maps 1:1 to the framework’s physical key enum
(`KeyCode`, aligned with `keyboard-types::Code` per ADR 0091), e.g.:

- `"KeyA"`, `"KeyP"`
- `"Digit1"`
- `"ArrowUp"`
- `"Enter"`, `"Escape"`, `"Tab"`, `"Space"`
- `"F1"`, `"F12"`
- `"MetaLeft"`, `"MetaRight"`
- `"Unidentified"`

Modifiers are stored as an explicit set:

- `"shift"`, `"ctrl"`, `"alt"`, `"altgr"`, `"meta"`

Notes:

- `altgr` is semantically distinct from `ctrl+alt` and is required for international layouts.
  See `docs/adr/0043-shortcut-arbitration-pending-bindings-and-altgr.md`.

### 3) Binding shape (single-stroke, extensible)

V1 supports **single-stroke** shortcuts (one key + modifiers).

The schema reserves a path for future multi-stroke chords (e.g. `Ctrl+K, Ctrl+C`) by allowing `keys`
to become either a single object or an array in a future version.

### 3b) Binding shape v2 (multi-stroke sequences)

V2 supports editor-style multi-stroke bindings (ADR 0043):

- `"keymap_version": 2`
- `binding.keys` may be an array of key specs (each spec is the same `{ "mods": [...], "key": "KeyX" }` shape as v1).

Example:

```json
{
  "keymap_version": 2,
  "bindings": [
    {
      "command": "editor.comment_line",
      "when": "focus.is_text_input == false",
      "keys": [
        { "mods": ["ctrl"], "key": "KeyK" },
        { "mods": ["ctrl"], "key": "KeyC" }
      ]
    }
  ]
}
```

Semantics:

- “Last-wins” resolution applies to the **full sequence** (not each chord independently).
- Pending prefixes are handled by the UI dispatcher (timeout + replay), not by platform backends.
  See `docs/adr/0043-shortcut-arbitration-pending-bindings-and-altgr.md`.

### 4) Platform targeting

Bindings may optionally target specific platforms:

- `"platform": "macos" | "windows" | "linux" | "all"` (default: `"all"`)

This allows standard editor conventions:

- macOS uses `meta` for most global shortcuts,
- Windows/Linux use `ctrl` for most global shortcuts.

### 5) Context gating (minimal but future-proof)

Bindings may optionally include a `when` expression to restrict applicability.

V1 defines `when` as an opaque string that is interpreted by the command router; the expression
language is intentionally deferred, but the format is reserved early to avoid a breaking redesign.

Recommended initial contexts to support:

- `focus.is_text_input`
- `focus.is_viewport`
- `ui.has_modal`
- `window.is_focused`
- `panel.kind == "core.scene"`

### 6) Unbinding and overrides

Keymaps are layered (ADR 0014):

1. framework defaults
2. plugin defaults
3. user `keymap.json`
4. project `keymap.json` (optional)

Later layers override earlier ones.

To disable a default binding, `command` can be set to `null` (explicit unbind).

### 7) Conflict detection and reporting

When loading keymaps, conflicts are detected by the tuple:

`(platform, when, modifiers, key)`

Resolution rule:

- last-wins (later layers override earlier layers),
- conflicts should be logged and surfaced in a future “Keymap” UI.

## Example (keymap.json, v1)

```json
{
  "keymap_version": 1,
  "bindings": [
    {
      "command": "app.command_palette",
      "platform": "macos",
      "keys": { "mods": ["meta"], "key": "KeyP" }
    },
    {
      "command": "app.command_palette",
      "platform": "windows",
      "keys": { "mods": ["ctrl"], "key": "KeyP" }
    },
    {
      "command": "app.command_palette",
      "platform": "linux",
      "keys": { "mods": ["ctrl"], "key": "KeyP" }
    },
    {
      "command": "text.delete_backward",
      "when": "focus.is_text_input",
      "keys": { "mods": [], "key": "Backspace" }
    },
    {
      "command": null,
      "keys": { "mods": ["ctrl"], "key": "KeyW" }
    }
  ]
}
```

## Consequences

- Key bindings remain stable across keyboard layouts (physical keys).
- Platform differences are handled without forking the entire settings model.
- Future context-aware bindings are possible without a format-breaking redesign.
- Plugins can ship reasonable defaults while user/project overrides stay predictable.

## Future Work

- Keep the `when` expression language and available context keys aligned with ADR 0022 (`docs/adr/0022-when-expressions.md`).
- Harden multi-stroke chord arbitration against text input/IME (see ADR 0043).
- Lock shortcut arbitration + AltGr semantics + pending bindings (see `docs/adr/0043-shortcut-arbitration-pending-bindings-and-altgr.md`).
- Provide a canonical list of `KeyCode` tokens and migration rules when upstream (`keyboard-types`) changes.

## Implementation Notes (Current Prototype)

- Keymap format + parser are implemented in `crates/fret-runtime` (`KeymapFileV1`, `Keymap`, `WhenExpr`).
- Layered keymap loading from disk (user + project) is implemented in `crates/fret-app/src/config_files.rs` (`load_layered_keymap`) and used by the bootstrap golden path (`ecosystem/fret-bootstrap/src/lib.rs`).
- Sample file: `docs/examples/keymap.json`
