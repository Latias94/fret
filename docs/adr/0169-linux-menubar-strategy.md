# ADR 0169: Linux Menubar Strategy (In-window Fallback First)

Status: Proposed

## Context

Fret targets editor-grade UX where a menu bar is a primary discovery surface. On Windows and macOS,
native OS menu bars are well-defined and users expect standard behavior.

On Linux desktops, “OS menubar” behavior is fragmented:

- Some environments support global menus via DBus/AppMenu (e.g. historical Ubuntu indicators), but
  this is not universal and has ecosystem drift (Ayatana, different shells, varying defaults).
- GTK can render a native menubar, but adopting GTK as a runtime dependency is a large commitment
  (toolkit coupling, theming integration, packaging, and event loop considerations).
- Winit does not provide a cross-desktop “native menubar” abstraction for Linux.

We already have a data-only `MenuBar` model (ADR 0023) and an OS integration seam
`Effect::SetMenuBar` (ADR 0168). We need a strategy that preserves portability and avoids locking
the project into a heavyweight toolkit too early.

## Decision

For Linux (native desktop runner), Fret will treat the OS menubar as **unsupported for now**.

- The desktop runner on Linux MAY ignore `Effect::SetMenuBar` for OS integration.
- Apps SHOULD provide an **in-window menubar** (ecosystem-owned UI surface) as the default Linux
  experience, backed by the same `MenuBar` model.
- Shortcut labels and enablement gating MUST remain consistent across surfaces by using:
  - the stable shortcut display policy (`Keymap::display_shortcut_for_command_sequence`), and
  - best-effort enable/disable gating via `when` + window-scoped `InputContext` snapshots
    (`WindowInputContextService`).

This keeps contracts stable while still delivering editor-grade UX on Linux.

## Options Considered

### A) Implement a GTK menubar now

Pros:

- Native-looking menubar for common GTK desktops.

Cons:

- Large new dependency surface and toolkit coupling.
- Packaging and theming complexity.
- Requires careful integration with the existing winit event loop.

### B) Implement DBus global menu (AppMenu / Ayatana)

Pros:

- Matches the “global menu” model on desktops that support it.

Cons:

- Not universal; behavior varies across desktops.
- Adds a new IPC integration surface and testing burden.

### C) In-window menubar as the default (Chosen)

Pros:

- Works everywhere on Linux regardless of desktop environment.
- Keeps `fret-ui` runtime contract stable; policy stays in ecosystem.
- Aligns with the existing “menus are data” model and shadcn/kit surfaces.

Cons:

- Not an OS-native menubar.
- Requires apps to allocate vertical chrome space.

## Consequences

### Positive

- Linux support does not force a heavyweight toolkit commitment.
- Menu authoring remains data-driven and command-derived.
- Shortcut labels and enablement logic stay consistent across OS menubar (where available),
  in-window menus, and command palette.

### Negative / Trade-offs

- Linux will not have an OS-native menubar in the near term.
- Some Linux desktop integrations (global menus) will remain a potential gap until revisited.

## Implementation Notes

- Linux desktop runner currently performs no OS menubar mapping; this is consistent with the
  decision.
- In-window menubar is ecosystem-owned and can be rendered via shadcn/kit bridges from `MenuBar`.

## Future Work

- Revisit GTK menubar integration behind an explicit feature gate (e.g. `linux-gtk-menubar`) if a
  strong use case emerges.
- Revisit DBus global menu support if a clear, widely-supported target is identified.
- If/when Linux OS menubar is implemented, document capability discovery and fallback behavior in
  the platform capability matrix (ADR 0054).

