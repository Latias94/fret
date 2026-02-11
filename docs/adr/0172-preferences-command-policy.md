# ADR 0172: Preferences Command Policy (App-Owned)

Status: Proposed

## Context

`app.preferences` is a standard command in `fret-app` (with a macOS default keybinding, `Cmd+,`),
and is typically exposed from an OS menubar via `MenuBar` (ADR 0168 / ADR 0170).

Unlike the About menu item (which macOS has a standard native panel for), **Preferences has no
single OS-native UI** that a runner can show without application policy.

Fret also targets both:

- editor-grade UX (where Preferences often opens an in-app settings editor), and
- general-purpose apps (which may want a dedicated Preferences window, a modal sheet, or a route).

We want to keep menus command-driven and avoid expanding `fret-runtime` with effects that imply UI
policy.

## Decision

- `app.preferences` remains **app-owned** and **command-driven**.
- There is **no default runner-native behavior** for Preferences.
- Golden-path stacks MAY provide convenience hooks to route `app.preferences` to an app-defined UI.

### Golden-path convenience

The `fret-bootstrap` `UiAppDriver` provides an optional `on_preferences` hook for apps that want a
central place to implement Preferences without mixing it into their generic command handler.

Semantics:

- The hook is invoked for window-scoped command dispatch of `app.preferences`.
- Apps decide whether Preferences is a window-local surface (sheet/panel) or a global window.

Note:

- `fret-kit` enables a small default Preferences overlay by default (built with `fret-ui-kit`
  primitives) that surfaces layered config file locations and clipboard copy actions. Apps can
  override this by installing their own `on_preferences` hook.

## Consequences

### Positive

- Keeps `fret-runtime` free of UI policy for Preferences.
- Preserves portability across OS menubar and in-window menubar.
- Works naturally with keymap customization (`Cmd+,` is a default, not a hard-coded behavior).

### Trade-offs

- There is no out-of-the-box Preferences UI; demos and products must implement it.
- “Standard” menu wiring exists, but the action is intentionally not standardized as a platform
  effect.

## References

- ADR 0168: `docs/adr/0168-os-menubar-effect-setmenubar.md`
- ADR 0170: `docs/adr/0170-menu-roles-system-menus-and-os-actions.md`
