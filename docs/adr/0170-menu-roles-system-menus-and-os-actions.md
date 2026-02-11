# ADR 0170: Menu Roles, System Menus, and OS Actions

Status: Accepted

## Context

ADR 0168 defines the platform integration seam for OS menubars via `Effect::SetMenuBar`, and ADR 0023 defines a minimal, data-only `MenuBar` model derived from commands.

As the OS menubar mapping matured (Windows `HMENU`, macOS `NSMenu`), we hit a platform reality:

- macOS expects certain “standard” menus and behaviors (Application menu conventions, Services,
  Window menu hook, standard edit actions).
- Windows has long-standing accelerator expectations (e.g. `Ctrl+C`/`Ctrl+V`) and system behaviors
  that benefit from being represented as native actions where possible.
- In-window menus (client-side menubars) must remain themeable and fully customizable, but should
  still be able to express the same high-level semantics without forking the data model.

Without an explicit model for “menu roles” and “system menus”, we risk encoding these semantics in
ad-hoc title checks (“a menu titled `Window` means the system Window menu”) or per-runner special
cases. That becomes a hard-to-change implicit contract.

We want a minimal, explicit vocabulary that:

- keeps `fret-runtime` data-only,
- enables best-effort OS-native integration where available,
- preserves portability (in-window fallback) without forcing OS-specific UI policy into `fret-ui`,
- remains compatible with `menubar.json` layering and future schema evolution.

## Decision

Extend the menu model vocabulary with **optional semantic hints** that runners and in-window
surfaces can interpret consistently:

### 1) Menu roles (top-level and submenus)

Introduce `MenuRole` as an optional semantic identifier for `Menu` (and optionally for submenus),
covering at least:

- `App` (macOS Application menu; global)
- `File`
- `Edit`
- `View`
- `Window` (macOS Window menu hook; global)
- `Help`

Semantics:

- Roles are **hints**. The data model remains authoritative for structure and titles.
- Runners MAY apply platform-specific integration based on role (e.g. macOS `Window` hook).
- In-window menubars SHOULD treat roles as metadata (for ordering, grouping, or display) but MUST
  not require them to render correctly.

### 2) System-managed submenus (e.g. macOS Services)

Introduce a `SystemMenuType` vocabulary for submenus whose content is managed by the OS.

Minimum v1:

- `Services` (macOS Services menu)

Semantics:

- OS runners MAY map a `SystemMenu(Services)` entry into the platform’s system-managed menu slot.
- In-window menus SHOULD ignore or render a disabled placeholder for system-managed menus.

### 3) OS actions (standard command semantics)

Introduce optional `OsAction` metadata for command-backed menu items (typically attached to
command metadata), representing standard edit actions where OS integration can provide specialized
behavior and expected keyboard/selector wiring.

Minimum v1:

- `Cut`, `Copy`, `Paste`, `SelectAll`, `Undo`, `Redo`

Semantics:

- `OsAction` does not replace `CommandId` routing; it is an **additional hint**.
- When available, OS runners MAY bind native selectors/accelerators for these actions, while still
  dispatching through the normal command/effect path.
- In-window menus may use `OsAction` only for labeling or grouping; it is not required.

### 4) Cross-surface consistency

Regardless of whether menus are rendered as OS menubars or in-window UI:

- Shortcut labels MUST use the stable display policy (ADR 0168 / MVP 2).
- Enable/disable gating MUST be best-effort and driven by `when` + the window-scoped
  `InputContext` snapshots (ADR 0168).

### 5) macOS App menu conventions (MVP baseline)

When authoring a macOS Application menu (typically the first menu in the menubar), prefer:

- top-level menu: `MenuRole::App`
- include a system-managed Services submenu via `SystemMenuType::Services` when desired
- keep items command-driven so keymap customization and command routing stay consistent

Recommended item order (with separators), matching common native conventions:

1. About
2. (sep)
3. Preferences…
4. (sep)
5. Services (system menu; optional)
6. (sep)
7. Hide
8. Hide Others
9. Show All
10. (sep)
11. Quit

Notes:

- The titles remain app-defined (via command metadata); runners may apply additional native behavior,
  but menu item selection must still dispatch through the standard command/effects path.
- `fret-workspace` provides a helper to inject this baseline (see `WorkspaceMenuCommands`).

## Options Considered

### A) Implicit semantics via menu titles (Rejected)

Pros:
- No model changes.

Cons:
- Becomes an implicit contract and is hard to evolve.
- Breaks localization and customization (“Window” might be renamed).
- Encourages per-runner string hacks.

### B) Full OS-native menu API surface in `fret-ui` (Rejected)

Pros:
- Maximum OS integration control.

Cons:
- Inflates `fret-ui` contract surface (violates ADR 0066 intent).
- Makes portability and demo-driven iteration harder.

### C) Minimal semantic vocabulary in `fret-runtime` (Chosen)

Pros:
- Explicit, portable, and data-only.
- Enables OS best-effort integration without forcing a widget kit.
- Supports both OS menubars and in-window menubars from one model.

Cons:
- Requires careful schema evolution for `menubar.json` (likely a future version bump).
- Some platform semantics still require runner-specific behavior.

## Consequences

### Positive

- macOS standard menu expectations can be represented without title string hacks.
- Windows/macOS can map standard edit actions more naturally when desired.
- In-window menus remain themeable and can still express meaningful semantics for ordering and UX.

### Negative / Trade-offs

- The menu model becomes slightly richer.
- `menubar.json` may need a version bump to express roles/system menus cleanly.

## Implementation Notes

- This ADR intentionally does not mandate a specific Rust type layout; it locks the semantic
  vocabulary and cross-surface expectations.
- A follow-up change should:
  - update `fret-runtime::menu` data model to carry role/system/action metadata,
  - update OS runners (macOS first) to honor these semantics,
  - update the in-window menubar bridge to ignore or gracefully represent system menus,
  - express roles/system menus in `menubar.json` via `menu_bar_version: 2` (menu `role`, and `system_menu` items).

## References (non-normative)

- Zed/GPUI system menus and OS actions:
  - `repo-ref/zed/crates/gpui/src/platform/app_menu.rs` (`SystemMenuType`, `OsAction`)
- Follow-up: ADR 0171 (`Effect::ShowAboutPanel`) to provide a native About baseline on macOS while keeping About command-driven.
