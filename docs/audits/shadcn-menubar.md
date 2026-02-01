# shadcn/ui v4 Audit - Menubar

This audit compares Fret's shadcn-aligned `Menubar` against the upstream shadcn/ui v4 docs and
examples in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/menubar.mdx`
- Reference implementation (Radix base): `repo-ref/ui/apps/v4/registry/bases/radix/ui/menubar.tsx`
- Reference example: `repo-ref/ui/apps/v4/registry/bases/radix/examples/menubar-example.tsx`

Key upstream behaviors/surfaces:

- Menubar root + menu triggers with roving focus across the bar (ArrowLeft/ArrowRight).
- Menu content: groups, labels, separators, shortcuts, disabled items, checkbox/radio items,
  submenus.
- Open policy: click-to-open; hover switches menus when one is already open.
- Dismissal: Escape + outside press.

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/menubar.rs`
- Overlay policy substrate: `ecosystem/fret-ui-kit/src/overlay_controller.rs`
- Dismissible popover policy: `ecosystem/fret-ui-kit/src/window_overlays/mod.rs`

## Audit checklist

### Open/close policy

- Pass: Click-to-open per menu trigger.
- Pass: Hover switches the open menu when a menu is already active.
- Pass: Outside press + Escape dismiss via menu policy; outside-press dismissal is click-through and
  underlay pointer interaction remains enabled (Radix Menubar uses `Menu.modal=false`).
- Note: Fret exposes an explicit `close_on_select` policy per item; upstream Radix typically relies
  on `onSelect(e) { e.preventDefault() }` to keep menus open for toggles.
- Pass: Close auto-focus outcomes match Radix Menubar’s `onCloseAutoFocus` policy:
  - Escape close restores focus to the menubar trigger (keeps keyboard roving continuity).
  - Outside-press close leaves focus to the user agent (`<body/>`); in Fret this is modeled as
    `UiTree` focus `None`.
  - Upstream anchor: `repo-ref/primitives/packages/react/menubar/src/menubar.tsx` (`MenubarContent`).

### Placement & sizing

- Pass: Anchored placement to the trigger bounds (`Side::Bottom`, `Align::Start`).
- Pass: Panel sizing is derived from the entry list (row padding + line height) and is clamped by
  the available window bounds; overflow scrolls on Y, matching shadcn's `max-h-(--available-height)`
  + `overflow-y-auto` outcome (best-effort, renderer-driven).
- Pass: Panel width uses a shadcn-like `min-width` baseline but grows when long checkbox/radio labels
  require more space (mirrors upstream `min-w-*` behavior rather than a fixed `w-*`).
- Note: The "grow to fit long labels" width currently uses a deterministic text-width heuristic to
  avoid introducing a renderer-coupled measurement dependency into the recipe layer. If this proves
  brittle across fonts, consider plumbing a text measurement service through the UI host/runtime and
  switching the heuristic to real glyph metrics.

### Keyboard navigation

- Pass: Menu content supports roving + typeahead.
- Pass: Menubar trigger row supports ArrowLeft/ArrowRight roving (wrap) between triggers.

### Visual parity

- Partial: Token usage aligns with shadcn-ish defaults (border/background/radius), but some
  higher-fidelity slots are still text-based or consumer-provided (see missing surfaces).
- Pass: `inset` is supported for items/labels (left padding parity with upstream `data-inset`).
- Pass: Destructive item variants are supported via `MenubarItemVariant::Destructive`.
- Pass: Submenu triggers render a right chevron (`ids::ui::CHEVRON_RIGHT`) to match upstream affordance.
- Pass: Root and submenu panels animate with shadcn’s overlay motion taxonomy (fade + zoom, plus slide-in on enter).
- Pass: Leading icons are aligned within a fixed 16×16 slot; when any row provides a leading icon,
  the menu reserves the slot across the panel for consistent label alignment.

## Missing surfaces (significant)

Still missing (relative to upstream shadcn/ui v4):

_None tracked at this time._

## Validation

- Interaction test: `menubar_hover_switches_open_menu`
- Interaction test: `menubar_triggers_roving_moves_focus_with_arrow_keys`
- Contract test: `menubar_items_have_collection_position_metadata_excluding_separators`
- Submenu openSteps parity (web-vs-fret): `menubar-demo.submenu-kbd*` follows the extractor semantics (`scrollIntoView({ block: "center" })` + focus + ArrowRight),
  while `menubar-demo.submenu` opens via hover after driving the submenu open-delay timer from effects.
- Radix Web overlay geometry gate: `cargo nextest run -p fret-ui-shadcn --test radix_web_overlay_geometry`
  (`radix_web_menubar_open_geometry_matches_fret`).
- shadcn-web chrome gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_chrome`
  (`web_vs_fret_menubar_panel_chrome_matches`; consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.open.json`).
- shadcn-web root shadow gate (`shadow-xs`): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_chrome`
  (`web_vs_fret_menubar_root_shadow_matches_web`, `web_vs_fret_menubar_root_shadow_matches_web_dark`;
  consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.open.json`).
- shadcn-web shadow gate (`shadow-md`): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_chrome`
  (`web_vs_fret_menubar_demo_shadow_matches_web`, `web_vs_fret_menubar_demo_shadow_matches_web_dark`;
  consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.open.json`).
- shadcn-web shadow gate (`shadow-md`, View menu): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_chrome`
  (`web_vs_fret_menubar_demo_view_shadow_matches_web`, `web_vs_fret_menubar_demo_view_shadow_matches_web_dark`;
  consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.view.open.json`).
- shadcn-web shadow gate (`shadow-md`, Profiles menu): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_chrome`
  (`web_vs_fret_menubar_demo_profiles_shadow_matches_web`, `web_vs_fret_menubar_demo_profiles_shadow_matches_web_dark`;
  consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.profiles.open.json`).
- shadcn-web shadow gate (`shadow-md`, constrained viewport): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_chrome`
  (`web_vs_fret_menubar_demo_small_viewport_shadow_matches_web`, `web_vs_fret_menubar_demo_small_viewport_shadow_matches_web_dark`;
  consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.vp1440x320.open.json`).
- shadcn-web shadow gate (`shadow-md`, tiny viewport): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_chrome`
  (`web_vs_fret_menubar_demo_tiny_viewport_shadow_matches_web`, `web_vs_fret_menubar_demo_tiny_viewport_shadow_matches_web_dark`;
  consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.vp1440x240.open.json`).
- shadcn-web surface colors gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_chrome`
  (`web_vs_fret_menubar_demo_surface_colors_match_web`, `web_vs_fret_menubar_demo_surface_colors_match_web_dark`;
  consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.open.json`).
- shadcn-web surface colors gate (View menu): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_chrome`
  (`web_vs_fret_menubar_demo_view_surface_colors_match_web`, `web_vs_fret_menubar_demo_view_surface_colors_match_web_dark`;
  consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.view.open.json`).
- shadcn-web surface colors gate (Profiles menu): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_chrome`
  (`web_vs_fret_menubar_demo_profiles_surface_colors_match_web`, `web_vs_fret_menubar_demo_profiles_surface_colors_match_web_dark`;
  consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.profiles.open.json`).
- shadcn-web surface colors gate (constrained viewport): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_chrome`
  (`web_vs_fret_menubar_demo_small_viewport_surface_colors_match_web`, `web_vs_fret_menubar_demo_small_viewport_surface_colors_match_web_dark`;
  consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.vp1440x320.open.json`).
- shadcn-web surface colors gate (tiny viewport): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_chrome`
  (`web_vs_fret_menubar_demo_tiny_viewport_surface_colors_match_web`, `web_vs_fret_menubar_demo_tiny_viewport_surface_colors_match_web_dark`;
  consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.vp1440x240.open.json`).
- shadcn-web placement gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement`
  (`web_vs_fret_menubar_demo_overlay_placement_matches`; consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.open.json`).
- shadcn-web placement gate (View menu): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement`
  (`web_vs_fret_menubar_demo_view_overlay_placement_matches`; consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.view.open.json`).
- shadcn-web placement gate (Profiles menu): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement`
  (`web_vs_fret_menubar_demo_profiles_overlay_placement_matches`; consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.profiles.open.json`).
- shadcn-web placement gate (constrained viewport): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement`
  (`web_vs_fret_menubar_demo_small_viewport_overlay_placement_matches`; consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.vp1440x320.open.json`).
- shadcn-web scroll state gate (constrained viewport): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement`
  (`web_vs_fret_menubar_demo_small_viewport_scroll_state_matches`; consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.vp1440x320.open.json`).
- shadcn-web scroll state gate (tiny viewport): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement`
  (`web_vs_fret_menubar_demo_tiny_viewport_scroll_state_matches`; consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.vp1440x240.open.json`).
- shadcn-web menu row height gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement`
  (`web_vs_fret_menubar_demo_menu_item_height_matches`; consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.open.json`).
- shadcn-web menu row height gate (View menu): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement`
  (`web_vs_fret_menubar_demo_view_menu_item_height_matches`; consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.view.open.json`).
- shadcn-web menu row height gate (Profiles menu): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement`
  (`web_vs_fret_menubar_demo_profiles_menu_item_height_matches`; consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.profiles.open.json`).
- shadcn-web item padding + shortcut/chevron alignment gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement`
  (`web_vs_fret_menubar_demo_item_padding_and_shortcut_match`; consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.open.json`).
- shadcn-web checkbox indicator slot inset gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement`
  (`web_vs_fret_menubar_demo_view_checkbox_indicator_slot_inset_matches_web`; consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.view.open.json`).
- shadcn-web radio indicator slot inset gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement`
  (`web_vs_fret_menubar_demo_profiles_radio_indicator_slot_inset_matches_web`; consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.profiles.open.json`).
- shadcn-web menu content inset gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement`
  (`web_vs_fret_menubar_demo_menu_content_insets_match`; consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.open.json`).
- shadcn-web menu content inset gate (View menu): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement`
  (`web_vs_fret_menubar_demo_view_menu_content_insets_match`; consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.view.open.json`).
- shadcn-web menu content inset gate (Profiles menu): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement`
  (`web_vs_fret_menubar_demo_profiles_menu_content_insets_match`; consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.profiles.open.json`).
- shadcn-web menu row height gate (constrained viewport): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement`
  (`web_vs_fret_menubar_demo_small_viewport_menu_item_height_matches`; consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.vp1440x320.open.json`).
- shadcn-web menu content inset gate (constrained viewport): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement`
  (`web_vs_fret_menubar_demo_small_viewport_menu_content_insets_match`; consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.vp1440x320.open.json`).
- shadcn-web submenu placement gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement`
  (`web_vs_fret_menubar_demo_submenu_overlay_placement_matches`; consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.submenu-kbd.open.json`).
- shadcn-web submenu shadow gate (`shadow-lg`): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_chrome`
  (`web_vs_fret_menubar_demo_submenu_shadow_matches_web`, `web_vs_fret_menubar_demo_submenu_shadow_matches_web_dark`;
  consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.submenu-kbd.open.json`).
- shadcn-web submenu surface colors gate (hover): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_chrome`
  (`web_vs_fret_menubar_demo_submenu_surface_colors_match_web`, `web_vs_fret_menubar_demo_submenu_surface_colors_match_web_dark`;
  consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.submenu.open.json`).
- shadcn-web submenu shadow gate (kbd, constrained viewport, `shadow-lg`): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_chrome`
  (`web_vs_fret_menubar_demo_submenu_kbd_small_viewport_shadow_matches_web`, `web_vs_fret_menubar_demo_submenu_kbd_small_viewport_shadow_matches_web_dark`;
  consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.submenu-kbd-vp1440x320.open.json`).
- shadcn-web submenu surface colors gate (kbd, constrained viewport): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_chrome`
  (`web_vs_fret_menubar_demo_submenu_kbd_small_viewport_surface_colors_match_web`, `web_vs_fret_menubar_demo_submenu_kbd_small_viewport_surface_colors_match_web_dark`;
  consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.submenu-kbd-vp1440x320.open.json`).
- shadcn-web submenu shadow gate (kbd, tiny viewport, `shadow-lg`): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_chrome`
  (`web_vs_fret_menubar_demo_submenu_kbd_tiny_viewport_shadow_matches_web`, `web_vs_fret_menubar_demo_submenu_kbd_tiny_viewport_shadow_matches_web_dark`;
  consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.submenu-kbd-vp1440x240.open.json`).
- shadcn-web submenu surface colors gate (kbd, tiny viewport): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_chrome`
  (`web_vs_fret_menubar_demo_submenu_kbd_tiny_viewport_surface_colors_match_web`, `web_vs_fret_menubar_demo_submenu_kbd_tiny_viewport_surface_colors_match_web_dark`;
  consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.submenu-kbd-vp1440x240.open.json`).
- shadcn-web submenu first visible item gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement`
  (`web_vs_fret_menubar_demo_submenu_first_visible_matches`; consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.submenu-kbd.open.json`).
- shadcn-web submenu first visible item gate (constrained viewport): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement`
  (`web_vs_fret_menubar_demo_submenu_small_viewport_first_visible_matches`; consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.submenu-kbd-vp1440x320.open.json`).
- shadcn-web submenu menu row height gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement`
  (`web_vs_fret_menubar_demo_submenu_menu_item_height_matches`; consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.submenu-kbd.open.json`).
- shadcn-web submenu menu row height gate (hover): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement`
  (`web_vs_fret_menubar_demo_submenu_hover_menu_item_height_matches`; consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.submenu.open.json`).
- shadcn-web submenu menu content inset gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement`
  (`web_vs_fret_menubar_demo_submenu_menu_content_insets_match`; consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.submenu-kbd.open.json`).
- shadcn-web submenu menu content inset gate (hover): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement`
  (`web_vs_fret_menubar_demo_submenu_hover_menu_content_insets_match`; consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.submenu.open.json`).
- shadcn-web submenu placement gate (constrained viewport): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement`
  (`web_vs_fret_menubar_demo_submenu_small_viewport_overlay_placement_matches`; consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.submenu-kbd-vp1440x320.open.json`).
- shadcn-web submenu menu content inset gate (constrained viewport): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement`
  (`web_vs_fret_menubar_demo_submenu_small_viewport_menu_content_insets_match`; consumes `goldens/shadcn-web/v4/new-york-v4/menubar-demo.submenu-kbd-vp1440x320.open.json`).
