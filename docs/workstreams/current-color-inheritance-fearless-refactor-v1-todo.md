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
- [ ] Menu family provides resolved `fg`:
  - [ ] `DropdownMenuItem` (and label/shortcut rows)
  - [ ] `SelectItem`
  - [ ] `CommandItem`
- [ ] Badge provides resolved `fg`.
- [ ] Tabs triggers / pill-like controls provide resolved `fg`.

## Text adoption (the big win after icons)

- [ ] Decide the minimal text surface that should inherit `currentColor`:
  - [ ] `fret-ui-kit::ui::text(...)` default color?
  - [ ] `declarative::text` wrappers?
  - [ ] a separate `currentTextStyle` provider (v2)?
- [ ] Implement the chosen default (inherit-first, theme fallback second).
- [ ] Add a focused unit test proving “button sets fg → text inherits fg”.

## Gallery + docs cleanups

- [ ] Remove manual `*_fg` threading in gallery pages where inheritance is now sufficient.
- [ ] Add one “copy/paste ready” snippet per component page where code previously relied on local variables.

## Diagnostics / regression gates

- [x] Add a Button Group diag script capturing Demo preview + Code.
- [ ] Add a “primary button + icon” diag script that asserts the icon is visible:
  - [ ] capture screenshot in zinc/light and zinc/dark presets
  - [ ] (optional) pixel-change assertion for the icon bounds
- [ ] Add a “menu item with leading icon + disabled/active” diag script.

## Cross-cutting checks

- [ ] Ensure `currentColor` does not leak across unrelated subtrees (nested scopes restore correctly).
- [ ] Confirm explicit color overrides still win over inherited `currentColor`.
- [ ] Document the rule of thumb: “hosts provide, leaves consume”.

