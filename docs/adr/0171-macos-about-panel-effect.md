# ADR 0171: macOS About Panel Effect

Status: Proposed

## Context

ADR 0170 defines how we model App menu conventions as command-driven menu items (e.g. `app.about`)
while keeping the menu model data-only and keymap-driven.

On macOS, users expect the **standard native About panel** (provided by `NSApplication`) when
choosing the About menu item from the global menubar. Rendering an in-window toast/dialog is
functional but feels non-native, and it also complicates multi-window behavior and focus-driven
gating.

We want to keep:

- menu items command-driven (so keymap customization and command routing remain consistent), and
- OS-native behavior where it improves user expectations, without inflating `fret-ui` contracts.

## Decision

Introduce a portable effect in `fret-runtime`:

- `Effect::ShowAboutPanel`

Semantics:

- It is a best-effort request to show an About UI surface.
- Native runners MAY map it to a platform-native implementation.
- Runners on platforms without a native equivalent MAY ignore it.

### macOS mapping

The macOS desktop runner handles `Effect::ShowAboutPanel` by invoking:

- `NSApplication orderFrontStandardAboutPanel:`

### Command mapping (golden path)

On macOS, the bootstrap driver maps the standard App command:

- `app.about` → `Effect::ShowAboutPanel`

On non-macOS platforms, `app.about` remains app-owned (no forced default mapping), so demos and
products can provide an in-window About dialog if desired.

## Consequences

### Positive

- macOS About behaves natively without making menus non-command-driven.
- Keeps the menu/data model stable and avoids adding new policy surface in `fret-ui`.
- Improves consistency with editor-grade UX expectations (Zed/Unity-style workflows).

### Trade-offs

- Adds a new runtime contract variant (`Effect`), which is hard-to-change once shipped.
- Non-macOS platforms still need an app-level About experience if desired.

## Options Considered

### A) Keep `app.about` fully app-owned everywhere (Rejected)

Pros:
- No new runtime contract.

Cons:
- macOS default demos feel non-native by default.

### B) Make the macOS runner invoke native About directly on menu selection (Rejected)

Pros:
- No new `Effect` variant.

Cons:
- Breaks the "menu items dispatch through commands/effects" principle and reduces composability.

### C) Add `Effect::ShowAboutPanel` (Chosen)

Pros:
- Preserves command-driven menu dispatch and keymap customization.
- Allows a native mapping where available.

## References

- ADR 0170: `docs/adr/0170-menu-roles-system-menus-and-os-actions.md`
- Apple docs: `orderFrontStandardAboutPanel:`
