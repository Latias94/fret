# ADR 0016: Plugin and Panel Boundaries


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Zed: https://github.com/zed-industries/zed

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
Status: Accepted

## Context

An editor-grade framework needs extensibility:

- plugins can register dockable panels,
- plugins can register commands and key bindings,
- plugins may contribute settings/configuration.

If plugin boundaries are not defined early, plugins tend to reach “through” layers (UI into
renderer, platform into app state), creating tight coupling and forcing rewrites.

References:

- Zed Settings model: file-scoped, strongly typed configuration:
  - https://zed.dev/blog/settings-ui
- Zed panel + extension surfaces (non-normative code anchors):
  - panels as workspace-integrated surfaces:
    `repo-ref/zed/crates/panel`
  - extension API/host split:
    `repo-ref/zed/crates/extension_api`, `repo-ref/zed/crates/extension_host`
- Fret docking persistence and stable identity:
  - ADR 0013
- Fret resource handle ownership:
  - ADR 0004

## Decision

### 1) Plugins integrate via the app layer

Plugins register into an app-owned registry (conceptually `AppRegistry`) and may contribute:

- `PanelKind` + panel factory for dockable views,
- commands (IDs + metadata + handlers),
- keymap entries (shortcut → command),
- settings schema contributions (scoped into files).

### 2) Plugins do not depend on the renderer crate

Plugins must not:

- import `fret-render`,
- create `wgpu` resources directly “through” UI APIs,
- depend on platform-specific window/event types.

Plugins can only interact with GPU data via stable handles:

- `RenderTargetId`, `ImageId`, `TextBlobId`, etc.

### 3) Panel identity is stable across restarts

Plugins must provide stable identifiers for persistence:

- `PanelKind` is the persistent identity used in layout files,
- `PanelKey` (kind + optional instance) identifies panel instances during runtime and in persistence.

Note:

- runtime-only IDs still exist for other structures (e.g. `DockNodeId`), but panels themselves should not require
  a separate ephemeral ID in order to support persistence and plugin registration.

## Consequences

- The core stays maintainable: renderer/platform remain implementation details.
- Layout persistence and plugin panels are compatible by construction.
- wasm/mobile ports remain feasible because plugins do not embed platform-specific assumptions.

## Future Work

- Define sandboxing/security stance for third-party plugins.
- Add plugin lifecycle hooks (startup/shutdown, project open/close).
- Add a compatibility story for versioned settings + layout migrations.
