# ADR 0022: `when` Expressions (Context Gating for Commands and Keymap)

Status: Accepted

## Context

Editor key bindings and commands must often be context-sensitive:

- text editing widgets should consume text shortcuts,
- viewport panels should enable camera/gizmo controls when focused,
- modals/popups should block most global shortcuts,
- plugins should be able to add context-aware commands without hardcoding UI internals.

ADR 0021 reserves a `when` string in `keymap.json`, but without a defined expression model we risk:

- ad-hoc hardcoded checks sprinkled across the UI runtime,
- incompatible interpretations between keymap matching and command discoverability,
- a later breaking redesign of the keymap format.

References:

- Focus + command routing semantics:
  - `docs/adr/0020-focus-and-command-routing.md`
- Keymap file format reserving `when`:
  - `docs/adr/0021-keymap-file-format.md`

## Decision

### 1) `when` is a small boolean expression language

`when` is defined as a pure, side-effect-free boolean expression evaluated against an `InputContext`.

V1 grammar (conceptual):

- literals: `true`, `false`, string literals `"..."`, numbers (optional for future)
- identifiers: dotted paths like `focus.is_text_input`
- operators:
  - unary: `!`
  - binary: `&&`, `||`
  - equality: `==`, `!=`
- parentheses: `( ... )`

If `when` is omitted, it is treated as `true`.

### 2) Unknown identifiers are false (safe default)

If an identifier is not available in the current context, it evaluates to `false`.

This prevents unexpected global activation when a context key is missing (especially important for plugins).

### 3) Canonical context keys (v1 minimum set)

The framework defines a minimal set of context keys that are stable across platforms:

Window/UI state:

- `ui.has_modal` (bool)
- `ui.has_popup` (bool)
- `window.is_focused` (bool)

Focus state:

- `focus.is_text_input` (bool)
- `focus.is_viewport` (bool)
- `focus.panel_kind` (string, empty if none)

Platform:

- `platform` (string: `"macos" | "windows" | "linux" | "web"`)

Key contexts (v1):

- `keyctx.<context>` (bool): true when `<context>` is active in the current key-context stack.
  - `<context>` uses dot-separated naming (e.g. `cookbook.commands`, `workspace.tabs`).
  - Hierarchical matching is supported: if `cookbook.commands` is active, `keyctx.cookbook` is also true.
  - The key-context stack is derived from the focused node chain (or the modal barrier root when
    focus is outside the barrier subtree), and does not cross the active modal barrier.

Plugins may register additional keys in a namespaced manner, e.g.:

- `plugin.my_plugin.some_flag`

### 4) `when` is shared by keymap and command discoverability

The same `when` expression is used for:

- keymap matching (whether a binding is eligible),
- command palette / menus (whether a command is enabled/visible).

This avoids divergence between “can you trigger it?” and “does it show up?”.

### 5) Parsing/compilation is cached

`when` strings are parsed/compiled once and cached:

- parsing errors disable the binding/command (treated as `false`) and should be logged.

## Consequences

- Context gating becomes consistent across key bindings, menus, and command palette.
- The system remains extensible without leaking UI internals into configuration.
- Safety defaults prevent accidental activation in unknown contexts.

## Future Work

- Decide whether to support numeric comparisons and set membership (`in`) for richer gating.
- Add a validation mode for keymap/commands to surface unknown keys and parse errors in UI.
- Define exact plugin registration APIs for context keys and their types.
