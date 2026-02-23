# Foreground inheritance (`currentColor`) (fearless refactor v1) — Milestones

Last updated: 2026-02-23

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
- `CommandItem` provides `currentColor` and supports `leading_icon` (missing: gates + demo migrations).

Evidence anchors:

- `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`
- `ecosystem/fret-ui-shadcn/src/command.rs`

### M5 — Text adoption (planned; biggest ROI after icons)

- Decide the minimal text surface that should inherit `currentColor` (v1 scope is “foreground only”).
- Implement inherit-first, theme-fallback-second for the chosen surface.
- Add a focused unit test proving “button sets fg → text inherits fg”.

### M6 — Close the loop (planned)

- Audit remaining “foreground leaves” (glyphs, chevrons, status dots) and adopt inheritance where appropriate.
- Expand gallery migrations to remove manual fg token threading where it is now redundant.
- Add 1–2 diag scripts to lock “primary button + icon” and “menu item + leading icon” outcomes across light/dark.

## Definition of done (v1)

- Provider + leaf adoption landed without breaking callers.
- At least 2 high-ROI hosts provide `currentColor` (Button + one menu family).
- At least 2 regression gates exist (unit tests + diag scripts).
- Gallery examples for the aligned components are copy/paste friendly and do not thread fg tokens by default.

