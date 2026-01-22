# shadcn/ui v4 Audit - Context Menu

This audit compares Fret's shadcn-aligned `ContextMenu` against the upstream shadcn/ui v4 docs and
examples in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/context-menu.mdx`
- Reference implementation (Radix base): `repo-ref/ui/apps/v4/registry/bases/radix/ui/context-menu.tsx`
- Reference example: `repo-ref/ui/apps/v4/registry/bases/radix/examples/context-menu-example.tsx`

Key upstream behaviors/surfaces:

- Rich content surface similar to dropdown menus: groups, labels, separators, shortcuts, disabled
  items, checkbox items, radio items, submenus, portals.
- Open policy: opened via a trigger region (right click / long press), not a normal button click.
- Dismissal: outside press + Escape.
- Keyboard: roving navigation + typeahead; trigger can be invoked via keyboard as well.

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/context_menu.rs`
- Overlay policy substrate: `ecosystem/fret-ui-kit/src/overlay_controller.rs`
- Dismissible popover policy: `ecosystem/fret-ui-kit/src/window_overlays/mod.rs`

## Audit checklist

### Trigger policy

- Pass: Opens on right click.
- Pass: (macOS) ctrl + left click opens.
- Pass: Shift+F10 opens (fallback keyboard path).
- Note: There is no dedicated `ContextMenu` key in `fret_core::KeyCode` yet.

### Placement

- Pass: Anchored to the last pointer-down position recorded in the trigger region.
- Pass: Keyboard-open fallback anchors at the trigger bounds origin when no pointer position is
  available.
- Pass: Placement uses `anchored_panel_bounds_sized` and clamps to an inset viewport rect.

### Dismissal & focus

- Pass: Dismissible menu via `window_overlays` (outside press + Escape).
- Pass: On open, focus moves to the first focusable descendant in the menu (via overlay policy),
  enabling keyboard navigation.
- Pass: Selecting an item dispatches the command (if any) and closes the menu.
- Pass: `ContextMenu::modal(bool)` is supported (default `true`).
  - `modal=true`: blocks underlay pointer interaction while open (Radix `disableOutsidePointerEvents`).
  - `modal=false`: outside-press dismissal becomes click-through.
- Note: Fret exposes an explicit `close_on_select` policy per item; upstream Radix typically relies
  on `onSelect(e) { e.preventDefault() }` to keep menus open for toggles.

### Keyboard navigation & typeahead

- Pass: Uses `RovingFlex` + APG navigation + prefix typeahead.
- Pass: Restores focus back to the trigger on close (Radix-style menu default).

### Visual parity (shadcn)

- Partial: Token usage roughly aligns with popover/menu defaults; remaining parity gaps are mostly
  around any missing fine-grained layout details.

## Missing surfaces (significant)

Still missing (relative to upstream shadcn/ui v4):

_None tracked at this time._

## Implemented surfaces (notable)

- Pass: Submenus (single-level) with Radix-style pointer grace intent (safe-hover corridor) via
  `fret-ui-kit::primitives::menu::sub` + `menu::root::submenu_pointer_move_handler`.
- Pass: Group semantics structure matches upstream (`role="group"` is the direct parent of grouped
  menu items) by using `ElementKind::SemanticFlex` (avoids inserting a separate semantics wrapper
  layer above layout).
- Pass: Chevron-right submenu affordance icon parity.
- Pass: Destructive item variant styling via `ContextMenuItemVariant::Destructive`.

## Validation

- Contract test: `context_menu_items_have_collection_position_metadata_excluding_separators`
- Interaction test: `context_menu_opens_on_shift_f10`
- Interaction test: `context_menu_submenu_opens_on_arrow_right_without_pointer_move`
- Web placement gate (layout engine v2): `cargo nextest run -p fret-ui-shadcn --test radix_web_overlay_geometry`
- Web placement gate (root): `web_vs_fret_context_menu_demo_overlay_placement_matches`
  (consumes `goldens/shadcn-web/v4/new-york-v4/context-menu-demo.open.json`).
- Web menu row height gate (root): `web_vs_fret_context_menu_demo_menu_item_height_matches`
  (consumes `goldens/shadcn-web/v4/new-york-v4/context-menu-demo.open.json`).
- Web checkbox/radio indicator slot inset gate (root): `web_vs_fret_context_menu_demo_checkbox_indicator_slot_inset_matches_web`,
  `web_vs_fret_context_menu_demo_radio_indicator_slot_inset_matches_web`
  (consumes `goldens/shadcn-web/v4/new-york-v4/context-menu-demo.open.json`).
- Web item padding + shortcut alignment gate (root): `web_vs_fret_context_menu_demo_back_item_padding_and_shortcut_match`
  (consumes `goldens/shadcn-web/v4/new-york-v4/context-menu-demo.open.json`).
- Web menu content inset gate (root): `web_vs_fret_context_menu_demo_menu_content_insets_match`
  (consumes `goldens/shadcn-web/v4/new-york-v4/context-menu-demo.open.json`).
- Web panel shadow gate (root, `shadow-md`): `web_vs_fret_context_menu_demo_shadow_matches_web`,
  `web_vs_fret_context_menu_demo_shadow_matches_web_dark`
  (consumes `goldens/shadcn-web/v4/new-york-v4/context-menu-demo.open.json`).
- Web placement gate (submenu): `web_vs_fret_context_menu_demo_submenu_overlay_placement_matches`
  (consumes `goldens/shadcn-web/v4/new-york-v4/context-menu-demo.submenu-kbd.open.json`).
- Web submenu panel shadow gate (`shadow-lg`): `web_vs_fret_context_menu_demo_submenu_shadow_matches_web`,
  `web_vs_fret_context_menu_demo_submenu_shadow_matches_web_dark`
  (consumes `goldens/shadcn-web/v4/new-york-v4/context-menu-demo.submenu-kbd.open.json`).
- Web menu content inset gate (submenu): `web_vs_fret_context_menu_demo_submenu_menu_content_insets_match`
  (consumes `goldens/shadcn-web/v4/new-york-v4/context-menu-demo.submenu-kbd.open.json`).
- Web menu content inset gate (submenu, hover): `web_vs_fret_context_menu_demo_submenu_hover_menu_content_insets_match`
  (consumes `goldens/shadcn-web/v4/new-york-v4/context-menu-demo.submenu.open.json`).
- Web placement gate (submenu, constrained viewport): `web_vs_fret_context_menu_demo_submenu_small_viewport_overlay_placement_matches`
  (consumes `goldens/shadcn-web/v4/new-york-v4/context-menu-demo.submenu-kbd-vp1440x320.open.json`).
- Web placement gate (root, constrained viewport): `web_vs_fret_context_menu_demo_small_viewport_overlay_placement_matches`
  (consumes `goldens/shadcn-web/v4/new-york-v4/context-menu-demo.vp1440x320.open.json`).
- Web scroll state gate (root, constrained viewport): `web_vs_fret_context_menu_demo_small_viewport_scroll_state_matches`
  (consumes `goldens/shadcn-web/v4/new-york-v4/context-menu-demo.vp1440x320.open.json`).
- Web scroll state gate (root, tiny viewport): `web_vs_fret_context_menu_demo_tiny_viewport_scroll_state_matches`
  (consumes `goldens/shadcn-web/v4/new-york-v4/context-menu-demo.vp1440x240.open.json`).
- Web menu row height gate (root, constrained viewport): `web_vs_fret_context_menu_demo_small_viewport_menu_item_height_matches`
  (consumes `goldens/shadcn-web/v4/new-york-v4/context-menu-demo.vp1440x320.open.json`).
- Web menu content inset gate (root, constrained viewport): `web_vs_fret_context_menu_demo_small_viewport_menu_content_insets_match`
  (consumes `goldens/shadcn-web/v4/new-york-v4/context-menu-demo.vp1440x320.open.json`).
- Web menu content inset gate (submenu, constrained viewport): `web_vs_fret_context_menu_demo_submenu_small_viewport_menu_content_insets_match`
  (consumes `goldens/shadcn-web/v4/new-york-v4/context-menu-demo.submenu-kbd-vp1440x320.open.json`).
