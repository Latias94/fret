# Foreground inheritance (`currentColor`) (fearless refactor v1) — Milestones

Last updated: 2026-02-24

This workstream is a “fearless refactor” because it reduces the need to thread foreground tokens through component
trees. The goal is that hosts compute a semantic foreground once, and leaf visuals (icons/spinners/text) inherit it
automatically.

## Milestones

### M0 — Tracker + decision anchors (landed)

- Workstream spec + status table:
  - `docs/workstreams/current-color-inheritance-fearless-refactor-v1.md`
- TODO list:
  - `docs/workstreams/current-color-inheritance-fearless-refactor-v1-todo.md`

### M1 — Provider + leaf adoption (landed)

- `fret-ui-kit`: `currentColor` provider helpers (`with_current_color_provider` / `inherited_current_color`).
- Icons default to inherited `currentColor` when `color=None`.
- Spinner defaults to inherited `currentColor` when `color=None`.

Evidence anchors:

- `ecosystem/fret-ui-kit/src/declarative/current_color.rs`
- `ecosystem/fret-ui-kit/src/declarative/icon.rs`
- `ecosystem/fret-ui-shadcn/src/spinner.rs`

### M2 — Host adoption: Button (landed)

- `shadcn::Button` provides `currentColor` to its subtree.
- `shadcn::Button` offers deferred icon slots (`leading_icon` / `trailing_icon` / `icon`) so common icon usage is
  built under the provider (no manual token threading).

Evidence anchor:

- `ecosystem/fret-ui-shadcn/src/button.rs`

### M3 — Gallery alignment: ButtonGroup (landed)

- Update the ButtonGroup gallery page to avoid manual fg token threading and make code snippets copy/paste friendly.
- Gate with a diag screenshot script (Preview + Code).

Evidence anchors:

- `apps/fret-ui-gallery/src/ui/previews/pages/components/basics/button_group.rs`
- `tools/diag-scripts/ui-gallery-button-group-demo-screenshots.json`

### M4 — Host adoption: Menu family (in progress)

- `DropdownMenuItem` provides `currentColor` and supports `leading_icon`.
- `ContextMenuItem` supports deferred `leading_icon`.
- `MenubarItem` supports deferred `leading_icon`.
- `CommandItem` provides `currentColor` and supports `leading_icon` (including disabled state propagation).
- Gallery: align Command page with shadcn `command-demo` (icons + disabled item + shortcuts).
- Gate with light/dark screenshot scripts for both Dropdown Menu and Command demo icon visibility.
  - Context Menu: `tools/diag-scripts/ui-gallery-context-menu-icons-screenshots-zinc-light.json` + `...-zinc-dark.json`
  - Menubar: `tools/diag-scripts/ui-gallery-menubar-with-icons-screenshots-zinc-light.json` + `...-zinc-dark.json`

Evidence anchors:

- `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`
- `ecosystem/fret-ui-shadcn/src/command.rs`
- `apps/fret-ui-gallery/src/ui/pages/command.rs`
- `tools/diag-scripts/ui-gallery-dropdown-menu-icons-screenshots.json`
- `tools/diag-scripts/ui-gallery-command-docs-demo-icons-screenshots.json`
- `tools/diag-scripts/ui-gallery-command-docs-demo-icons-screenshots-zinc-dark.json`
- `ecosystem/fret-ui-shadcn/src/context_menu.rs`
- `ecosystem/fret-ui-shadcn/src/menubar.rs`

### M4b — Host adoption: Tabs triggers (landed)

- `TabsTrigger` foreground is treated as a host-provided `currentColor` so icons/text follow selected/disabled state.
- Prefer deferred icon slots for trigger icons (`TabsItem::trigger_leading_icon` / `trigger_trailing_icon`) so icons are
  built under the provider (no manual fg token threading).
- Gate with a light/dark screenshot script for icon visibility.

Evidence anchors:

- `ecosystem/fret-ui-shadcn/src/tabs.rs`
- `apps/fret-ui-gallery/src/ui/previews/gallery/nav/tabs.rs`
- `tools/diag-scripts/ui-gallery-tabs-icons-screenshots-zinc-light-dark.json`

### M4c — Host adoption: Toggle Group items (landed)

- `ToggleGroupItem` foreground is treated as a host-provided `currentColor` so icons/text follow selected/disabled state.
- Prefer deferred icon slots for icon-only items (`ToggleGroupItem::icon` / `leading_icon` / `trailing_icon`) so icons are
  built under the provider (no manual fg token threading).
- Gate with a light/dark screenshot script for demo icon visibility.

Evidence anchors:

- `ecosystem/fret-ui-shadcn/src/toggle_group.rs`
- `apps/fret-ui-gallery/src/ui/pages/toggle_group.rs`
- `tools/diag-scripts/ui-gallery-toggle-group-demo-icons-screenshots-zinc-light-dark.json`

### M4d — Host adoption: Toggle (landed)

- `Toggle` foreground is treated as a host-provided `currentColor` so icons/text follow hover/selected/disabled states.
- Prefer deferred icon slots for common patterns (`Toggle::icon` / `leading_icon` / `trailing_icon`) so icons are built
  under the provider (no manual fg token threading).
- Gate with a light/dark screenshot script for demo icon visibility.

Evidence anchors:

- `ecosystem/fret-ui-shadcn/src/toggle.rs`
- `apps/fret-ui-gallery/src/ui/pages/toggle.rs`
- `tools/diag-scripts/ui-gallery-toggle-demo-icons-screenshots-zinc-light-dark.json`

### M4e — Host adoption: Select scroll arrows (landed)

- Align shadcn Select scroll arrow affordance icon foreground with upstream `text-popover-foreground` semantics.
- Gate with a light/dark screenshot script that captures scroll arrow visibility/contrast.

Evidence anchors:

- `ecosystem/fret-ui-shadcn/src/select.rs`
- `tools/diag-scripts/ui-gallery-shadcn-select-scroll-arrows-icons-screenshots-zinc-light-dark.json`

### M4f — Host adoption: InputGroupButton icons (landed)

- `InputGroupButton` provides resolved `fg` via `currentColor` so icon slots inherit the host foreground.
- Add icon slots (`icon` / `leading_icon` / `trailing_icon`) to avoid prebuilt `AnyElement` children bypassing the provider.
- Gate with light/dark screenshot scripts targeting Spinner extras.

Evidence anchors:

- `ecosystem/fret-ui-shadcn/src/input_group.rs`
- `apps/fret-ui-gallery/src/ui/previews/pages/components/basics/spinner.rs`
- `tools/diag-scripts/ui-gallery-spinner-extras-input-group-button-icon-screenshots-zinc-light.json`
- `tools/diag-scripts/ui-gallery-spinner-extras-input-group-button-icon-screenshots-zinc-dark.json`

### M5 — Text adoption (landed; biggest ROI after icons)

- Decide the minimal text surface that should inherit `currentColor` (v1 scope is “foreground only”).
- Implement inherit-first, theme-fallback-second for `ui::TextBox` + `ui::RawTextBox`.
- Add a focused unit test proving text inherits `currentColor` when available.

Evidence anchor:

- `ecosystem/fret-ui-kit/src/ui.rs`

### M6 — Close the loop (planned)

- Audit remaining “foreground leaves” (glyphs, chevrons, status dots) and adopt inheritance where appropriate.
- Expand gallery migrations to remove manual fg token threading where it is now redundant.
- Add 1–2 diag scripts to lock “primary button + icon” and “menu item + leading icon” outcomes across light/dark.

## Definition of done (v1)

- Provider + leaf adoption landed without breaking callers.
- At least 2 high-ROI hosts provide `currentColor` (Button + one menu family).
- At least 2 regression gates exist (unit tests + diag scripts).
- Gallery examples for the aligned components are copy/paste friendly and do not thread fg tokens by default.
