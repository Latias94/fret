# Material 3 Menu Submenu Wiring v1

Date: 2026-06-01

This note scopes and records the bounded fearless-refactor batch that wires Material 3 menu
submenus through the existing Radix-aligned `fret-ui-kit` menu submenu primitives.

## Truth

- Material `MenuItem` can act as a submenu trigger without dispatching a leaf selection.
- Submenu behavior is owned by `fret-ui-kit::primitives::menu::{sub, sub_trigger, sub_content}`;
  Material owns only the authoring helpers, row chrome, chevron, tokens, and overlay wiring.
- LTR opens inline-end submenus with `ArrowRight`; RTL opens inline-end submenus with `ArrowLeft`.
- `ArrowLeft` from submenu content restores focus to the submenu trigger in LTR.
- Long submenu content clamps to the dropdown max-height path and scrolls inside the submenu panel.

## Artifacts

- `MenuSubTrigger`, `MenuSubContent`, and `MenuSub` bridge the explicit Material entry-tree authoring
  model to `MenuItem { submenu: ... }`.
- `MenuItem::submenu(...)` and `MenuItem::value(...)` support compact recipe authoring and stable
  submenu identity for duplicate labels.
- `DropdownMenu::submenu_min_width(...)` gives callers the same bounded sizing escape hatch as the
  mature shadcn dropdown/context menu wrappers.
- `MaterialMenuSubmenuContext` keeps submenu policy state out of core and threads the existing kit
  submenu models through root and nested Material menu panels.

## Wiring

- `DropdownMenu` creates submenu models under its named overlay root using
  `menu::root::with_root_name_sync_root_open_and_ensure_submenu`.
- Root menu items call `menu::sub_trigger::wire` for hover, activation, and logical inline-end
  keyboard open/close.
- Submenu panels render through `menu::sub_content::submenu_panel_scroll_y_for_value_at`, so the
  trigger `controls` relation points at the mounted submenu content semantics node.
- Material row rendering adds an auto-mirrored submenu chevron and keeps open submenu triggers in
  the highlighted state layer path.
- `DropdownMenu` installs the shared submenu pointer-move handler on the overlay request instead
  of duplicating pointer grace logic in Material code.

## Proof

- `cargo test -p fret-ui-material3 --features diagnostics --test menu_state dropdown_menu_submenu_opens_on_arrow_right_and_arrow_left_restores_trigger_focus -- --exact --nocapture`
- `cargo test -p fret-ui-material3 --features diagnostics --test menu_state dropdown_menu_submenu_uses_rtl_inline_end_key_and_clamps_long_content -- --exact --nocapture`

## Residual Risk

- Pointer hover grace is wired through the shared kit handler, but this batch uses focused Material
  tests for keyboard, focus restore, RTL, chevron, and clamped submenu content. A future diagnostics
  script can add pointer corridor replay if a product surface depends on submenu hover at speed.
- Material submenu close/open motion is intentionally minimal in this batch; the root dropdown still
  owns the fade/scale motion gate, and richer submenu presence motion should wait for a concrete
  app-facing requirement.
