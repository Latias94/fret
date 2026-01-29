# shadcn/ui new-york-v4 Depth Checklist (Fret)

This checklist tracks **depth** (interaction + stress variants), not breadth coverage.

For breadth coverage snapshots, see `docs/audits/shadcn-new-york-v4-coverage.md`.
For high-impact gap notes, see `docs/audits/shadcn-new-york-v4-alignment.md`.

Status legend:

- **Gated**: backed by at least one deterministic test gate.
- **Partially gated**: some gates exist, but not across the full state matrix.
- **Not gated**: no deterministic gate yet (or not audited).

## Overlays: menus & listboxes

Goal: treat “menu height” and scroll affordances as **styling outcomes** (padding/border/row height +
scroll buttons + max-height clamping), not as incidental layout.

### Menubar (`menubar-demo*`)

- Open state: **Gated**
- Constrained height (viewport clamp): **Gated** (`*.vp1440x320.open`, `*.vp1440x240.open`)
- Row height (menu items): **Gated**
- Menu content insets + overall menu height: **Gated**
- Scroll state (first visible item under clamp): **Gated**
- Submenu placement + constrained submenu: **Gated** (`*.submenu*`)
- Submenu panel size (portal `w/h`): **Gated** (`*.submenu-kbd.open.json`, `*.submenu-kbd-vp1440x240.open.json`)
- SubTrigger open-state chrome (`data-state=open` bg/fg): **Gated** (`*.submenu-kbd.open.json`)
- Hovered/highlighted item chrome (bg/fg): **Gated** (`menubar-demo.highlight-first.open.json`)
- Keyboard roving focus visuals (focused item bg/fg): **Gated** (`menubar-demo.focus-first.open.json`)

Evidence anchors:

- Goldens: `goldens/shadcn-web/v4/new-york-v4/menubar-demo*.open.json`
- Goldens (state): `goldens/shadcn-web/v4/new-york-v4/menubar-demo.highlight-first.open.json`
- Goldens (state): `goldens/shadcn-web/v4/new-york-v4/menubar-demo.focus-first.open.json`
- Gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs`
  (`assert_menubar_demo_constrained_menu_content_insets_match`,
  `assert_menubar_demo_constrained_scroll_state_matches`,
  `assert_menubar_demo_submenu_*`)
  and `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome.rs`
  (`web_vs_fret_menubar_demo_panel_size_matches_web`,
  `web_vs_fret_menubar_demo_panel_size_matches_web_dark`,
  `web_vs_fret_menubar_demo_tiny_viewport_panel_size_matches_web`,
  `web_vs_fret_menubar_demo_tiny_viewport_panel_size_matches_web_dark`,
  `web_vs_fret_menubar_demo_submenu_kbd_tiny_viewport_panel_size_matches_web`,
  `web_vs_fret_menubar_demo_submenu_kbd_tiny_viewport_panel_size_matches_web_dark`,
  `web_vs_fret_menubar_demo_highlighted_item_chrome_matches_web`,
  `web_vs_fret_menubar_demo_focused_item_chrome_matches_web`)

### DropdownMenu (`dropdown-menu-demo*`, `context-menu-demo*`)

- Open state: **Gated**
- Constrained height (viewport clamp): **Gated** (`*.vp1440x320.open`, `*.vp1440x240.open`)
- Row height (menu items): **Gated**
- Menu content insets + overall menu height: **Gated**
- Scroll state (first visible item under clamp): **Gated**
- Submenu placement + constrained submenu: **Gated** (`*.submenu*`)
- Submenu panel size (portal `w/h`): **Gated** (`*.submenu-kbd.open.json`, `*.submenu-kbd-vp1440x240.open.json`)
- SubTrigger open-state chrome (`data-state=open` bg/fg): **Gated** (`*.submenu-kbd.open.json`)
- Hovered/highlighted item chrome (bg/fg): **Gated** (`*.highlight-first.open.json`)
- Keyboard roving focus visuals (focused item bg/fg): **Gated** (`*.focus-first.open.json`)
- Destructive variant focus tint (bg/fg): **Gated** (`context-menu-demo.submenu-kbd-delete-focus.open.json`)
- Destructive variant idle fg (no highlight): **Gated** (`context-menu-demo.submenu-kbd.open.json`)

Evidence anchors:

- Goldens:
  - `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-demo*.open.json`
  - `goldens/shadcn-web/v4/new-york-v4/context-menu-demo*.open.json`
- Goldens (state):
  - `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-demo.highlight-first.open.json`
  - `goldens/shadcn-web/v4/new-york-v4/context-menu-demo.highlight-first.open.json`
  - `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-demo.focus-first.open.json`
  - `goldens/shadcn-web/v4/new-york-v4/context-menu-demo.focus-first.open.json`
  - `goldens/shadcn-web/v4/new-york-v4/context-menu-demo.submenu-kbd-delete-focus.open.json`
  - Note: `context-menu-demo.focus-first` is captured by opening via pointer context menu and then
    forcing focus onto the first `[role='menuitem']` (scripted `steps=keys=[role='menuitem']@Home`),
    because the upstream `context-menu-demo` does not reliably open from the keyboard in our
    extractor harness.
- Gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs`
  (`assert_dropdown_menu_demo_constrained_scroll_state_matches`,
  `assert_context_menu_demo_constrained_scroll_state_matches`)
  and `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome.rs` (item state chrome + panel size)
  (`web_vs_fret_dropdown_menu_demo_panel_size_matches_web`,
  `web_vs_fret_dropdown_menu_demo_panel_size_matches_web_dark`,
  `web_vs_fret_dropdown_menu_demo_tiny_viewport_panel_size_matches_web`,
  `web_vs_fret_dropdown_menu_demo_tiny_viewport_panel_size_matches_web_dark`,
  `web_vs_fret_dropdown_menu_demo_submenu_kbd_tiny_viewport_panel_size_matches_web`,
  `web_vs_fret_dropdown_menu_demo_submenu_kbd_tiny_viewport_panel_size_matches_web_dark`,
  `web_vs_fret_context_menu_demo_panel_size_matches_web`,
  `web_vs_fret_context_menu_demo_panel_size_matches_web_dark`,
  `web_vs_fret_context_menu_demo_tiny_viewport_panel_size_matches_web`,
  `web_vs_fret_context_menu_demo_tiny_viewport_panel_size_matches_web_dark`,
  `web_vs_fret_context_menu_demo_submenu_kbd_tiny_viewport_panel_size_matches_web`,
  `web_vs_fret_context_menu_demo_submenu_kbd_tiny_viewport_panel_size_matches_web_dark`,
  `web_vs_fret_dropdown_menu_demo_highlighted_item_chrome_matches_web`,
  `web_vs_fret_dropdown_menu_demo_focused_item_chrome_matches_web`,
  `web_vs_fret_context_menu_demo_highlighted_item_chrome_matches_web`,
  `web_vs_fret_context_menu_demo_focused_item_chrome_matches_web`,
  `web_vs_fret_context_menu_demo_submenu_destructive_focused_item_chrome_matches_web`, plus `*_dark` variants).

### Select / Combobox listboxes

- Option row height: **Gated**
- Scroll button height: **Gated**
- Constrained viewport variants: **Gated** (e.g. `select-scrollable.vp1440x240`, `combobox-demo.vp1440x240`)
- Hovered/active option chrome: **Gated** (`*.highlight-first.open.json`)
- Keyboard focus (active option) chrome: **Gated** (`*.focus-first.open.json`, including cmdk/aria-activedescendant)

Evidence anchors:

- Gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs`
  (`assert_select_scrollable_listbox_option_height_matches`,
  `assert_select_scrollable_scroll_button_height_matches`,
  `assert_combobox_demo_listbox_option_height_matches`)
- Goldens (state):
  - `goldens/shadcn-web/v4/new-york-v4/select-demo.highlight-first.open.json`
  - `goldens/shadcn-web/v4/new-york-v4/select-scrollable.highlight-first.open.json`
  - `goldens/shadcn-web/v4/new-york-v4/combobox-demo.highlight-first.open.json`
  - `goldens/shadcn-web/v4/new-york-v4/select-demo.focus-first.open.json`
  - `goldens/shadcn-web/v4/new-york-v4/select-scrollable.focus-first.open.json`
  - `goldens/shadcn-web/v4/new-york-v4/combobox-demo.focus-first.open.json`
  - `goldens/shadcn-web/v4/new-york-v4/command-dialog.focus-first.open.json`
- Gates (state): `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome.rs`
  (`web_vs_fret_select_demo_highlighted_option_chrome_matches_web`,
  `web_vs_fret_select_scrollable_highlighted_option_chrome_matches_web`,
  `web_vs_fret_combobox_demo_highlighted_option_chrome_matches_web`,
  `web_vs_fret_select_demo_focused_option_chrome_matches_web`,
  `web_vs_fret_combobox_demo_focused_option_chrome_matches_web`,
  `web_vs_fret_command_dialog_focused_item_chrome_matches_web`, and `*_dark` variants)

## Overlays: popover / tooltip / hover card

Goal: gate panel sizing (padding/border outcomes) under constrained viewports early, so "clamp/shift/flip"
changes do not hide styling regressions.

### Popover (`popover-demo*`)

- Open state: **Gated**
- Constrained height (viewport clamp): **Gated** (`popover-demo.vp1440x240.open.json`)
- Panel size (portal `w/h`): **Gated** (`popover-demo.open.json`, `popover-demo.vp1440x240.open.json`)

Evidence anchors:

- Goldens: `goldens/shadcn-web/v4/new-york-v4/popover-demo*.open.json`
- Placement gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs`
  (`web_vs_fret_popover_demo_overlay_placement_matches`, `web_vs_fret_popover_demo_overlay_placement_matches_tiny_viewport`)
- Panel size gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome.rs`
  (`web_vs_fret_popover_demo_panel_size_matches_web`, `web_vs_fret_popover_demo_panel_size_matches_web_dark`,
  `web_vs_fret_popover_demo_tiny_viewport_panel_size_matches_web`, `web_vs_fret_popover_demo_tiny_viewport_panel_size_matches_web_dark`)

### Tooltip (`tooltip-demo*`)

- Open state: **Gated**
- Constrained height (viewport clamp): **Gated** (`tooltip-demo.vp1440x240.open.json`)
- Panel size (portal `w/h`): **Gated** (`tooltip-demo.open.json`, `tooltip-demo.vp1440x240.open.json`)

Evidence anchors:

- Goldens: `goldens/shadcn-web/v4/new-york-v4/tooltip-demo*.open.json`
- Placement gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs`
  (`web_vs_fret_tooltip_demo_overlay_placement_matches`, `web_vs_fret_tooltip_demo_overlay_placement_matches_tiny_viewport`)
- Panel size gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome.rs`
  (`web_vs_fret_tooltip_demo_panel_size_matches_web`, `web_vs_fret_tooltip_demo_panel_size_matches_web_dark`,
  `web_vs_fret_tooltip_demo_tiny_viewport_panel_size_matches_web`, `web_vs_fret_tooltip_demo_tiny_viewport_panel_size_matches_web_dark`)

### HoverCard (`hover-card-demo*`)

- Open state: **Gated**
- Constrained height (viewport clamp): **Gated** (`hover-card-demo.vp1440x240.open.json`)
- Panel size (portal `w/h`): **Gated** (`hover-card-demo.open.json`, `hover-card-demo.vp1440x240.open.json`)

Evidence anchors:

- Goldens: `goldens/shadcn-web/v4/new-york-v4/hover-card-demo*.open.json`
- Placement gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs`
  (`web_vs_fret_hover_card_demo_overlay_placement_matches`, `web_vs_fret_hover_card_demo_overlay_placement_matches_tiny_viewport`)
- Panel size gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome.rs`
  (`web_vs_fret_hover_card_demo_panel_size_matches_web`, `web_vs_fret_hover_card_demo_panel_size_matches_web_dark`,
  `web_vs_fret_hover_card_demo_tiny_viewport_panel_size_matches_web`, `web_vs_fret_hover_card_demo_tiny_viewport_panel_size_matches_web_dark`)

## Controls: pressed / disabled states

### Button (`button-default*`)

- Disabled opacity: **Gated** (`button-default.disabled.json`)
- Hovered background chrome: **Gated** (`button-default.hover.json`)
- Pressed background chrome: **Gated** (`button-default.pressed.json`)

Evidence anchors:

- Goldens (state):
  - `goldens/shadcn-web/v4/new-york-v4/button-default.hover.json`
  - `goldens/shadcn-web/v4/new-york-v4/button-default.pressed.json`
  - `goldens/shadcn-web/v4/new-york-v4/button-default.disabled.json`
- Gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_button.rs`
  (`web_vs_fret_button_default_hover_matches_web`, `web_vs_fret_button_default_pressed_matches_web`, `web_vs_fret_button_default_disabled_matches_web`)

## Charts (wrapper UI + interaction snapshots)

- Tooltip panel geometry (wrapper): **Gated**
- Tooltip internal row layout (advanced): **Gated** (`chart-tooltip-advanced` item rows + “Total” row bounds)
- Legend panel geometry (wrapper): **Gated**
- Interactive hover tooltip + cursor rect (scripted): **Gated** (`*.hover-mid`)
- Full chart engine rendering parity (axes/ticks/marks/hit-test): **Not gated** (not implemented as a shadcn chart engine yet)

Evidence anchors:

- Goldens: `goldens/shadcn-web/v4/new-york-v4/chart-*.json`
- Audit: `docs/audits/shadcn-chart.md`
- Gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs`

## DPI / font metrics

- Cross-DPI geometry stability: **Not gated**
- Cross-font metrics stability (“weird metrics” fonts): **Not gated**

Rationale: add once the interaction-state gates are stable; keep the matrix small and deterministic.
