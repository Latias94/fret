# shadcn/ui v4 Audit - Context Menu


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- MUI Base UI: https://github.com/mui/base-ui
- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `ContextMenu` against upstream shadcn/ui v4 (`new-york-v4`)
and Base UI context-menu behavior contracts.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/base/context-menu.mdx`
- Reference implementation (`new-york-v4`):
  `repo-ref/ui/apps/v4/registry/new-york-v4/ui/context-menu.tsx`
- Reference example (`new-york-v4`):
  `repo-ref/ui/apps/v4/registry/new-york-v4/examples/context-menu-demo.tsx`
- Base UI source (trigger/root):
  - `repo-ref/base-ui/packages/react/src/context-menu/root/ContextMenuRoot.tsx`
  - `repo-ref/base-ui/packages/react/src/context-menu/trigger/ContextMenuTrigger.tsx`

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
- Pass: Shift+F10 opens (keyboard path).
- Pass: `ContextMenu` key opens (keyboard path).
- Pass: root-level disabled gate now blocks pointer + keyboard open paths (`ContextMenu::disabled`).

### Placement

- Pass: Anchored to the last pointer-down position recorded in the trigger region.
- Pass: Keyboard-open fallback anchors at the trigger bounds origin when no pointer position is
  available.
- Pass: Placement uses `anchored_panel_bounds_sized` and clamps to an inset viewport rect.
- Note: Default-style ownership remains split on purpose: the trigger surface stays caller-owned,
  while menu panel chrome / placement defaults remain recipe-owned and explicit panel width
  overrides belong on `ContextMenuContent::min_width(...)`.

### Dismissal & focus

- Pass: Dismissible menu via `window_overlays` (outside press + Escape).
- Pass: On open, focus moves to the first focusable descendant in the menu (via overlay policy),
  enabling keyboard navigation.
- Pass: Selecting an item dispatches the command (if any) and closes the menu.
- Pass: The default copyable root path is now
  `ContextMenu::uncontrolled(cx).compose().trigger(...).content(...).entries(...)` for the common
  uncontrolled case, while controlled/uncontrolled open state parity remains available via
  `ContextMenu::from_open(open)` and `ContextMenu::new_controllable(cx, open, default_open)`
  (Base UI / Radix `open` + `defaultOpen`).
- Pass: Open lifecycle callbacks are available via `ContextMenu::on_open_change` and
  `ContextMenu::on_open_change_complete` (Base UI `onOpenChange` + `onOpenChangeComplete`).
- Pass: `ContextMenu::modal(bool)` is supported (default `true`).
  - `modal=true`: blocks underlay pointer interaction while open (Radix `disableOutsidePointerEvents`).
  - `modal=false`: outside-press dismissal becomes click-through.
- Pass: `ContextMenu::compose()` is now the default recipe-level composition bridge, while
  `build_parts(...)` / `into_element_parts(...)` remain available as lower-level closure adapters.
- Pass: `ContextMenuTrigger::build(child)` keeps the trigger on the typed late-landing path for
  first-party snippets and copyable examples.
- Pass: The typed `compose()` root keeps the explicit menu-entry model while moving the final root
  landing seam back to the true call site instead of forcing extracted helpers through
  `build_parts(...) -> AnyElement`.
- Pass: UI Gallery docs-aligned snippets now consistently prefer the typed `compose()` root lane
  instead of mixing the older compact `build(...)` root into the default teaching surface.
- Pass: `DropdownMenuSide::{InlineStart, InlineEnd}` is now supported on
  `ContextMenuContent::side(...)` and `DropdownMenuContent::side(...)`, closing the remaining RTL
  docs-surface gap against the upstream Base UI `side="inline-end"` example.
- Pass: UI Gallery docs-backed examples now use a caller-owned dashed context region closer to the
  upstream docs examples, while `Usage` intentionally remains the leaner copyable root example.
- Pass: UI Gallery docs-backed examples now adapt trigger copy to the committed primary pointer
  capability (`Right click here` vs `Long press here` / `Long press (...)`), matching the upstream
  docs examples more closely without changing the existing long-press mechanism.
- Note: Fret exposes an explicit `close_on_select` policy per item; upstream Radix typically relies
  on `onSelect(e) { e.preventDefault() }` to keep menus open for toggles.
- Note: No extra generic heterogeneous children API is planned right now; the explicit
  `ContextMenuEntry` tree is the intentional structured equivalent of upstream nested menu children,
  and widening this family to generic children would add hidden scope/collection contracts without
  unlocking new behavior.

### Keyboard navigation & typeahead

- Pass: Uses `RovingFlex` + APG navigation + prefix typeahead.
- Pass: Close does not restore focus to the trigger (matches Radix `ContextMenu`'s
  `onCloseAutoFocus(e) { e.preventDefault() }` outcome). In web goldens focus returns to `<body/>`;
  in Fret this is modeled as `UiTree` focus `None`.

### Visual parity (shadcn)

- Partial: Token usage roughly aligns with popover/menu defaults; remaining parity gaps are mostly
  around any missing fine-grained layout details.

## Missing surfaces (significant)

No currently confirmed significant surface gaps relative to the current shadcn/Base UI docs
surface.

- The previously investigated native touch long-press path on the UI Gallery `Basic` example now
  passes again through
  `tools/diag-scripts/ui-gallery/context-menu/ui-gallery-context-menu-basic-touch-long-press-open.json`,
  so it should remain treated as a regression gate rather than an open component/runtime gap.

## Implemented surfaces (notable)

- Pass: Submenus (single-level) with Radix-style pointer grace intent (safe-hover corridor) via
  `fret-ui-kit::primitives::menu::sub` + `menu::root::submenu_pointer_move_handler`.
- Pass: Group semantics structure matches upstream (`role="group"` is the direct parent of grouped
  menu items) by using `ElementKind::SemanticFlex` (avoids inserting a separate semantics wrapper
  layer above layout).
- Pass: Chevron-right submenu affordance icon parity.
- Pass: Destructive item variant styling via `ContextMenuItemVariant::Destructive`.
- Pass: UI Gallery page order now mirrors the upstream shadcn docs path first (`Demo`, `Usage`,
  docs examples through `RTL`, then `API Reference`), with `Notes` kept as a Fret-specific
  follow-up.
- Pass: UI Gallery example triggers now visually track the upstream docs surface more closely with a
  dashed context region instead of per-example outline buttons.
- Pass: UI Gallery example triggers now also keep the upstream fine/coarse pointer wording for
  right-click vs long-press affordances.
- Pass: The UI Gallery RTL example now uses logical `inline-end` placement on
  `ContextMenuContent::side(...)` instead of stopping at direction-provider-only parity.

## Validation

- Contract test: `context_menu_items_have_collection_position_metadata_excluding_separators`
- Contract test: `context_menu_new_controllable_uses_controlled_model_when_provided`
- Contract test: `context_menu_uncontrolled_multiple_instances_do_not_share_open_model`
- Contract test: `context_menu_new_controllable_applies_default_open`
- Contract test: `context_menu_open_change_events_emit_change_and_complete_after_settle`
- Contract test: `context_menu_open_change_events_complete_without_animation`
- Interaction test: `context_menu_opens_on_shift_f10`
- Interaction test: `context_menu_opens_on_context_menu_key`
- Interaction test: `context_menu_disabled_blocks_pointer_and_keyboard_open`
- Touch long-press diag gate: `tools/diag-scripts/ui-gallery/context-menu/ui-gallery-context-menu-basic-touch-long-press-open.json`
- Interaction test: `context_menu_submenu_opens_on_arrow_right_without_pointer_move`
- Submenu openSteps parity (web-vs-fret): `context-menu-demo.submenu-kbd*` follows the extractor semantics (`scrollIntoView({ block: "center" })` + focus + ArrowRight),
  while `context-menu-demo.submenu` opens via hover after driving the submenu open-delay timer from effects.
- Wheel scroll anchor stability gate: while emulating `scrollIntoView({ block: "center" })` for
  submenu triggers (via wheel), the root menu panel origin remains stable under wheel input
  (asserted in `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` during
  `web_vs_fret_context_menu_demo_submenu_*`).
- Underlay scroll anchor stability gate: when the context menu is opened via a pointer location,
  the menu panel stays anchored to the original pointer position even if the underlay scrolls
  (validated in `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` via
  `fret_context_menu_does_not_move_when_underlay_scrolls`).
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
- Web surface colors gate (root): `web_vs_fret_context_menu_demo_surface_colors_match_web`,
  `web_vs_fret_context_menu_demo_surface_colors_match_web_dark`
  (consumes `goldens/shadcn-web/v4/new-york-v4/context-menu-demo.open.json`).
- Web panel shadow gate (root, constrained viewport, `shadow-md`): `web_vs_fret_context_menu_demo_small_viewport_shadow_matches_web`,
  `web_vs_fret_context_menu_demo_small_viewport_shadow_matches_web_dark`
  (consumes `goldens/shadcn-web/v4/new-york-v4/context-menu-demo.vp1440x320.open.json`).
- Web surface colors gate (root, constrained viewport): `web_vs_fret_context_menu_demo_small_viewport_surface_colors_match_web`,
  `web_vs_fret_context_menu_demo_small_viewport_surface_colors_match_web_dark`
  (consumes `goldens/shadcn-web/v4/new-york-v4/context-menu-demo.vp1440x320.open.json`).
- Web panel shadow gate (root, tiny viewport, `shadow-md`): `web_vs_fret_context_menu_demo_tiny_viewport_shadow_matches_web`,
  `web_vs_fret_context_menu_demo_tiny_viewport_shadow_matches_web_dark`
  (consumes `goldens/shadcn-web/v4/new-york-v4/context-menu-demo.vp1440x240.open.json`).
- Web surface colors gate (root, tiny viewport): `web_vs_fret_context_menu_demo_tiny_viewport_surface_colors_match_web`,
  `web_vs_fret_context_menu_demo_tiny_viewport_surface_colors_match_web_dark`
  (consumes `goldens/shadcn-web/v4/new-york-v4/context-menu-demo.vp1440x240.open.json`).
- Web placement gate (submenu): `web_vs_fret_context_menu_demo_submenu_overlay_placement_matches`
  (consumes `goldens/shadcn-web/v4/new-york-v4/context-menu-demo.submenu-kbd.open.json`).
- Web submenu panel shadow gate (`shadow-lg`): `web_vs_fret_context_menu_demo_submenu_shadow_matches_web`,
  `web_vs_fret_context_menu_demo_submenu_shadow_matches_web_dark`
  (consumes `goldens/shadcn-web/v4/new-york-v4/context-menu-demo.submenu-kbd.open.json`).
- Web submenu panel shadow gate (kbd, constrained viewport, `shadow-lg`): `web_vs_fret_context_menu_demo_submenu_kbd_small_viewport_shadow_matches_web`,
  `web_vs_fret_context_menu_demo_submenu_kbd_small_viewport_shadow_matches_web_dark`
  (consumes `goldens/shadcn-web/v4/new-york-v4/context-menu-demo.submenu-kbd-vp1440x320.open.json`).
- Web submenu surface colors gate (kbd, constrained viewport): `web_vs_fret_context_menu_demo_submenu_kbd_small_viewport_surface_colors_match_web`,
  `web_vs_fret_context_menu_demo_submenu_kbd_small_viewport_surface_colors_match_web_dark`
  (consumes `goldens/shadcn-web/v4/new-york-v4/context-menu-demo.submenu-kbd-vp1440x320.open.json`).
- Web submenu panel shadow gate (kbd, tiny viewport, `shadow-lg`): `web_vs_fret_context_menu_demo_submenu_kbd_tiny_viewport_shadow_matches_web`,
  `web_vs_fret_context_menu_demo_submenu_kbd_tiny_viewport_shadow_matches_web_dark`
  (consumes `goldens/shadcn-web/v4/new-york-v4/context-menu-demo.submenu-kbd-vp1440x240.open.json`).
- Web submenu surface colors gate (kbd, tiny viewport): `web_vs_fret_context_menu_demo_submenu_kbd_tiny_viewport_surface_colors_match_web`,
  `web_vs_fret_context_menu_demo_submenu_kbd_tiny_viewport_surface_colors_match_web_dark`
  (consumes `goldens/shadcn-web/v4/new-york-v4/context-menu-demo.submenu-kbd-vp1440x240.open.json`).
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
