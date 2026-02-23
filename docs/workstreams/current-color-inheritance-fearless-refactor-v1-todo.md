# Foreground inheritance (`currentColor`) (fearless refactor v1) — TODO

Last updated: 2026-02-23

## Foundation (authoring glue)

- [x] Add `currentColor` provider surface (`with_current_color_provider` / `inherited_current_color`).
- [x] Export provider helpers in `fret-ui-kit::declarative::prelude`.

## Leaf adoption (consume `currentColor`)

- [x] `declarative::icon_with(...)` inherits `currentColor` when `color=None`.
- [x] `shadcn::Spinner` inherits `currentColor` when `color=None`.
- [ ] Audit other “foreground leaves” and adopt inheritance where appropriate:
  - [ ] checkmark/radio dot glyphs (if rendered as icons)
  - [ ] chevrons/arrows used in triggers
  - [ ] status dots / inline SVG badges

## Host adoption (provide `currentColor`)

- [x] `shadcn::Button` provides resolved `fg` to subtree.
- [x] `shadcn::Button` supports deferred icon slots (`leading_icon` / `trailing_icon` / `icon`).
- [x] Audit icon-only controls that override `children` to render a single SVG icon and migrate to deferred icon slots where possible.
- [ ] Menu family provides resolved `fg` (and prefers deferred icon slots):
  - [x] `DropdownMenuItem` provides `currentColor` and supports `leading_icon`.
  - [ ] `SelectItem` (if/when it grows icon slots)
  - [x] `CommandItem` provides `currentColor` and supports `leading_icon` (add gates + migrate demos)
- [x] Badge provides resolved `fg`.
- [x] Tabs triggers / pill-like controls provide resolved `fg` (prefer deferred icon slots for trigger icons).
- [x] ToggleGroup items / pill-like controls provide resolved `fg` (prefer deferred icon slots for icon-only groups).
- [x] Toggle provides resolved `fg` (prefer deferred icon slots for icons).

## Text adoption (the big win after icons)

- [x] Decide the minimal text surface that should inherit `currentColor`:
  - [x] `fret-ui-kit::ui::text(...)` default color (via `ui::TextBox` + `ui::RawTextBox`).
  - [ ] `declarative::text` wrappers? (defer)
  - [ ] a separate `currentTextStyle` provider (v2)? (defer)
- [x] Implement the chosen default (inherit-first, theme fallback second).
- [x] Add a focused unit test proving text inherits `currentColor`.

## Gallery + docs cleanups

- [x] ButtonGroup gallery page: migrate icon usage to deferred `Button` icon slots (no manual fg threading).
- [ ] Remove manual `*_fg` threading in gallery pages where inheritance is now sufficient.
- [ ] Add one “copy/paste ready” snippet per component page where code previously relied on local variables.

## Diagnostics / regression gates

- [x] Add a Button Group diag script capturing Demo preview + Code.
- [x] Add a Dropdown Menu icons diag script (zinc/light + zinc/dark) to gate leading-icon foreground inheritance.
- [x] Add Command docs demo icons screenshot scripts (zinc/light + zinc/dark) to gate `CommandItem::leading_icon` inheritance.
- [x] Add a “primary button + icon” diag script that asserts the icon is visible:
  - [x] capture screenshot in zinc/light and zinc/dark presets (`tools/diag-scripts/ui-gallery-button-group-demo-icons-screenshots-zinc-light-dark.json`)
  - [ ] (optional) pixel-change assertion for the icon bounds
- [x] Add a Tabs icons screenshot script (zinc/light + zinc/dark) to gate trigger icon visibility (`tools/diag-scripts/ui-gallery-tabs-icons-screenshots-zinc-light-dark.json`).
- [x] Add a Toggle Group demo icons screenshot script (zinc/light + zinc/dark) to gate icon-only item visibility (`tools/diag-scripts/ui-gallery-toggle-group-demo-icons-screenshots-zinc-light-dark.json`).
- [x] Add a Toggle demo icons screenshot script (zinc/light + zinc/dark) to gate icon + label foreground inheritance (`tools/diag-scripts/ui-gallery-toggle-demo-icons-screenshots-zinc-light-dark.json`).
- [ ] Add a “menu item with leading icon + disabled/active” diag script.

## Cross-cutting checks

- [ ] Ensure `currentColor` does not leak across unrelated subtrees (nested scopes restore correctly).
- [ ] Confirm explicit color overrides still win over inherited `currentColor`.
- [ ] Document the rule of thumb: “hosts provide, leaves consume”.
