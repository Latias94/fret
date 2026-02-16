# ADR 0014: Settings and Configuration as Files + Strong Types


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Zed: https://github.com/zed-industries/zed

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
Status: Accepted

## Context

Editor-grade applications quickly accumulate settings:

- user settings, project settings,
- window/layout persistence,
- key bindings (keymap),
- plugin configuration.

Distributed runtime registration of settings across many crates can appear flexible, but in practice
it often produces:

- a fragmented settings model,
- difficulty building a coherent settings UI,
- complex macro-based layers that entangle “pre-UI” and “UI” regions.

Zed’s Settings UI experience shows that the core move is to treat **files as the organizing principle**
and keep the model strongly typed.

References:

- Zed settings UI writeup:
  - https://zed.dev/blog/settings-ui
- Zed code anchors (non-normative):
  - file-backed settings store and precedence layering:
    `repo-ref/zed/crates/settings/src/settings_store.rs` (`SettingsStore`, `SettingsFile`)
  - file watching + update helpers:
    `repo-ref/zed/crates/settings/src/settings_file.rs` (`watch_config_file`, `watch_config_dir`)
  - keymap file parsing and JSON schema generation:
    `repo-ref/zed/crates/settings/src/keymap_file.rs`

## Decision

### 1) Files are the organizing principle

Settings and layout are represented as a small set of well-known files, e.g.:

- `user/settings.json`
- `project/settings.json`
- `user/keymap.json`
- `user/menubar.json`
- `user/layout.json` (dock/window layout)

The exact paths can evolve, but the “file-scoped model” is part of the architecture.

#### Recommended default locations

To minimize later churn, pick stable defaults that follow platform conventions:

- **Per-project** (checked into VCS optionally): `./.fret/`
  - `./.fret/settings.json` (project settings)
  - `./.fret/layout.json` (project-scoped layout, optional)
- **Per-user** (machine local): OS config directories
  - macOS: `~/Library/Application Support/fret/`
  - Linux: `$XDG_CONFIG_HOME/fret/` (fallback: `~/.config/fret/`)
  - Windows: `%APPDATA%\\fret\\`

Recommended user files:

- `settings.json` (user settings)
- `keymap.json` (user key bindings)
- `menubar.json` (user menu bar, optional)
- `layout.json` (user window/dock layout; see ADR 0013 + ADR 0017)

Notes:

- Text defaults (e.g. default UI font family candidates) live in `settings.json` and are owned by the
  app/editor layer, then injected into the renderer (see ADR 0029).

### 2) Settings are strongly typed

Model settings as strongly typed Rust structures that map directly to these files.

#### Menu bar presentation (example)

Apps may want different menu bar surfaces depending on platform and product goals (native OS menu
bar vs an in-window menu bar for fully custom chrome).

Example `settings.json` snippet:

```json
{
  "settings_version": 1,
  "menu_bar": {
    "os": "auto",
    "in_window": "auto"
  }
}
```

Suggested semantics (non-normative):

- `os: auto|on|off` controls whether the app publishes `Effect::SetMenuBar` for native runners.
- `in_window: auto|on|off` controls whether the app renders a menu bar inside its window chrome.

### 3) Scope layering is explicit

When multiple scopes exist (user + project + workspace), the merge/override behavior is defined
explicitly and reflected in the settings UI.

## Consequences

- Settings UI can be built directly from the typed model without macro glue.
- Layout and keymap persistence become predictable and debuggable.
- Downstream engines/applications can adopt the same model for their editor-specific configuration.

## Future Work

- Choose a serialization strategy and schema validation approach (JSON + serde, optional JSON schema export).
- Define how plugins contribute settings while keeping the core model coherent.
