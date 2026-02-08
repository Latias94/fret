# shadcn/ui v4 Audit - Dropdown Menu

This audit compares Fret's shadcn-aligned `DropdownMenu` against the upstream shadcn/ui v4 docs and
examples in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/dropdown-menu.mdx`
- Reference implementation (Radix base): `repo-ref/ui/apps/v4/registry/bases/radix/ui/dropdown-menu.tsx`
- Reference examples: `repo-ref/ui/apps/v4/registry/bases/radix/examples/dropdown-menu-example.tsx`

Key upstream behaviors/surfaces:

- Rich content surface: groups, labels, separators, shortcuts, disabled items, destructive variant,
  checkbox items, radio items, submenus, and a portal-based content root.
- Content sizing: `align="start"`, `sideOffset=4`, width tracks trigger width, and max-height
  tracks available height with internal scroll.
- Keyboard: roving navigation + typeahead; focus styling is part of the visual parity.
- Dialog interop: examples mention `modal={false}` for opening dialogs from menus.

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`
- Overlay policy substrate: `ecosystem/fret-ui-kit/src/overlay_controller.rs`
- Dismissible popover policy: `ecosystem/fret-ui-kit/src/window_overlays/mod.rs`
- Roving + typeahead policy helpers: `ecosystem/fret-ui-kit/src/declarative/action_hooks.rs`

## Audit checklist

### Placement & sizing

- Pass: Per-window overlay root (portal-like), rendered via `OverlayController`.
- Pass: Anchored placement via the Radix-shaped popper facade (deterministic flip + clamp).
  - Placement policy: `fret_ui_kit::primitives::popper::PopperContentPlacement`
  - Solver: `crates/fret-ui/src/overlay_placement/solver.rs`
- Pass: `align="start"` respects the direction provider: under RTL, the content aligns to the
  trigger's logical start edge (Radix/Floating parity).
- Pass: Default `side_offset` aligns with upstream (`4`).
- Pass: Viewport size tracks trigger width (with a minimum) and clamps height to available space;
  internal scrolling is via `Scroll` (Y-axis).

### Dismissal & focus

- Pass: Dismissible popover (outside press + Escape) via `window_overlays`.
- Pass: On open, focus moves to the first focusable descendant (driven by overlay policy), enabling
  keyboard navigation inside the menu.
- Pass: Controlled/uncontrolled open state parity is available via
  `DropdownMenu::new_controllable(cx, open, default_open)` (Base UI / Radix `open` + `defaultOpen`).
- Pass: Open lifecycle callbacks are available via `DropdownMenu::on_open_change` and
  `DropdownMenu::on_open_change_complete` (Base UI `onOpenChange` + `onOpenChangeComplete`).
- Pass: `DropdownMenu::modal(bool)` is supported (default `true`).
  - `modal=true`: blocks underlay pointer interaction while open (Radix `disableOutsidePointerEvents`).
  - `modal=false`: outside-press dismissal becomes click-through.
- Pass: root-level disabled gate is now supported (`DropdownMenu::disabled(bool)`) and blocks
  trigger keyboard-open choreography.
- Pass: when root disabled is enabled, content stays hidden even if the controlled `open` model
  value is `true` (render-time gate, matching disabled-root expectations from Base UI/Radix family).
- Note: Fret exposes an explicit `close_on_select` policy per item; upstream Radix typically relies
  on `onSelect(e) { e.preventDefault() }` to keep menus open for toggles.

### Keyboard navigation & typeahead

- Pass: `RovingFlex` + APG-style navigation (`roving_nav_apg`) are wired.
- Pass: Prefix typeahead with timeout is wired.

### Visual parity (shadcn)

- Partial: Menu background/foreground now align with popover tokens (`bg-popover text-popover-foreground`).
- Pass: Hover/pressed/focused highlight uses `accent` tokens (Radix `data-[highlighted]`-style outcome).
- Pass: Destructive item variants keep destructive foreground + use a `destructive/10`-style highlight background.
- Pass: `inset` is supported for items/labels (left padding parity with upstream `data-inset`).
- Pass: Leading icons are aligned within a fixed 16×16 slot; when any row provides a leading icon,
  the menu reserves the slot across the panel for consistent label alignment.
- Pass: Checkbox/radio items reserve a stable indicator slot (`pl-8` / `left-2`) so label alignment does
  not jitter when toggling selection.

### Missing surfaces (significant)

Still missing (relative to upstream shadcn/ui v4):

_None tracked at this time._

### Submenus

- Pass: Submenus are supported via `DropdownMenuItem::submenu(Vec<DropdownMenuEntry>)` and open on
  hover/activate (pointer click), and can be opened/closed with ArrowRight/ArrowLeft.
- Pass: Pointer grace intent is implemented using a Radix-style polygon corridor (tracked via
  `fret-ui-kit::primitives::menu::pointer_grace_intent`), plus a short close delay on unsafe leave.
- Note: Submenu rendering no longer depends on pointer movement; keyboard-opened submenus render even
  when the pointer hasn't moved since the menu opened.
- Pass: Keyboard focus transfer is wired for submenus:
  - ArrowRight opens the submenu and transfers focus to the first enabled submenu item.
  - ArrowLeft closes the submenu and restores focus to the trigger item.

Notes on API mapping:

- Fret provides a single `DropdownMenu::into_element(trigger, entries)` entry point, rather than a
  DOM-like `Trigger`/`Content` component split.
- Groups/labels/shortcuts/destructive variant are modeled via:
  `DropdownMenuEntry::{Group,Label}` and `DropdownMenuItem::{trailing,variant}`.

## Validation

- Contract test: `dropdown_menu_items_have_collection_position_metadata_excluding_separators`
- Contract test: `dropdown_menu_new_controllable_uses_controlled_model_when_provided`
- Contract test: `dropdown_menu_new_controllable_applies_default_open`
- Contract test: `dropdown_menu_open_change_events_emit_change_and_complete_after_settle`
- Contract test: `dropdown_menu_open_change_events_complete_without_animation`
- Interaction test: `dropdown_menu_disabled_blocks_arrow_key_open_from_trigger`
- Interaction test: `dropdown_menu_disabled_hides_content_even_when_open_model_true`
  (ensures `pos_in_set`/`set_size` exclude separators).
- Interaction test: `dropdown_menu_submenu_opens_on_hover_and_closes_on_leave`
- Keyboard test: `dropdown_menu_submenu_opens_on_arrow_right_without_pointer_move`
- Direction test: `dropdown_menu_align_start_respects_direction_provider`
- Web placement gate (root): `web_vs_fret_dropdown_menu_demo_overlay_placement_matches`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-demo.open.json`).
- Web menu row height gate (root): `web_vs_fret_dropdown_menu_demo_menu_item_height_matches`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-demo.open.json`).
- Web item padding + shortcut alignment gate (root): `web_vs_fret_dropdown_menu_demo_profile_item_padding_and_shortcut_match`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-demo.open.json`).
- Web checkbox indicator slot inset gate (root): `web_vs_fret_dropdown_menu_checkboxes_checkbox_indicator_slot_inset_matches_web`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-checkboxes.open.json`).
- Web menu content inset gate (checkboxes): `web_vs_fret_dropdown_menu_checkboxes_menu_content_insets_match`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-checkboxes.open.json`).
- Web menu row height gate (checkboxes): `web_vs_fret_dropdown_menu_checkboxes_menu_item_height_matches`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-checkboxes.open.json`).
- Web radio indicator slot inset gate (root): `web_vs_fret_dropdown_menu_radio_group_radio_indicator_slot_inset_matches_web`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-radio-group.open.json`).
- Web menu content inset gate (radio group): `web_vs_fret_dropdown_menu_radio_group_menu_content_insets_match`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-radio-group.open.json`).
- Web menu row height gate (radio group): `web_vs_fret_dropdown_menu_radio_group_menu_item_height_matches`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-radio-group.open.json`).
- Web menu content inset gate (root): `web_vs_fret_dropdown_menu_demo_menu_content_insets_match`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-demo.open.json`).
- Web surface colors gate (root): `web_vs_fret_dropdown_menu_demo_surface_colors_match_web`,
  `web_vs_fret_dropdown_menu_demo_surface_colors_match_web_dark`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-demo.open.json`).
- Web panel shadow gate (root, `shadow-md`): `web_vs_fret_dropdown_menu_demo_shadow_matches_web`,
  `web_vs_fret_dropdown_menu_demo_shadow_matches_web_dark`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-demo.open.json`).
- Web panel shadow gate (root, constrained viewport, `shadow-md`): `web_vs_fret_dropdown_menu_demo_small_viewport_shadow_matches_web`,
  `web_vs_fret_dropdown_menu_demo_small_viewport_shadow_matches_web_dark`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-demo.vp1440x320.open.json`).
- Web surface colors gate (root, constrained viewport): `web_vs_fret_dropdown_menu_demo_small_viewport_surface_colors_match_web`,
  `web_vs_fret_dropdown_menu_demo_small_viewport_surface_colors_match_web_dark`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-demo.vp1440x320.open.json`).
- Web panel shadow gate (root, tiny viewport, `shadow-md`): `web_vs_fret_dropdown_menu_demo_tiny_viewport_shadow_matches_web`,
  `web_vs_fret_dropdown_menu_demo_tiny_viewport_shadow_matches_web_dark`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-demo.vp1440x240.open.json`).
- Web surface colors gate (root, tiny viewport): `web_vs_fret_dropdown_menu_demo_tiny_viewport_surface_colors_match_web`,
  `web_vs_fret_dropdown_menu_demo_tiny_viewport_surface_colors_match_web_dark`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-demo.vp1440x240.open.json`).
- Web surface colors gate (checkboxes): `web_vs_fret_dropdown_menu_checkboxes_surface_colors_match_web`,
  `web_vs_fret_dropdown_menu_checkboxes_surface_colors_match_web_dark`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-checkboxes.open.json`).
- Web panel shadow gate (checkboxes, `shadow-md`): `web_vs_fret_dropdown_menu_checkboxes_shadow_matches_web`,
  `web_vs_fret_dropdown_menu_checkboxes_shadow_matches_web_dark`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-checkboxes.open.json`).
- Web surface colors gate (radio group): `web_vs_fret_dropdown_menu_radio_group_surface_colors_match_web`,
  `web_vs_fret_dropdown_menu_radio_group_surface_colors_match_web_dark`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-radio-group.open.json`).
- Web panel shadow gate (radio group, `shadow-md`): `web_vs_fret_dropdown_menu_radio_group_shadow_matches_web`,
  `web_vs_fret_dropdown_menu_radio_group_shadow_matches_web_dark`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-radio-group.open.json`).
- Web placement gate (root, constrained viewport): `web_vs_fret_dropdown_menu_demo_small_viewport_overlay_placement_matches`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-demo.vp1440x320.open.json`).
- Web scroll state gate (root, constrained viewport): `web_vs_fret_dropdown_menu_demo_small_viewport_scroll_state_matches`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-demo.vp1440x320.open.json`).
- Web scroll state gate (root, tiny viewport): `web_vs_fret_dropdown_menu_demo_tiny_viewport_scroll_state_matches`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-demo.vp1440x240.open.json`).
- Web menu row height gate (root, constrained viewport): `web_vs_fret_dropdown_menu_demo_small_viewport_menu_item_height_matches`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-demo.vp1440x320.open.json`).
- Web menu content inset gate (root, constrained viewport): `web_vs_fret_dropdown_menu_demo_small_viewport_menu_content_insets_match`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-demo.vp1440x320.open.json`).
- Web placement gate (submenu): `web_vs_fret_dropdown_menu_demo_submenu_overlay_placement_matches`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-demo.submenu-kbd.open.json`).
- Wheel scroll anchor stability gate: while emulating `scrollIntoView({ block: "center" })` for
  submenu triggers (via wheel), the root menu panel origin remains stable under wheel input
  (asserted in `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` during
  `web_vs_fret_dropdown_menu_demo_submenu_*`).
- Underlay scroll anchor stability gate: when the trigger lives inside a scrolling underlay, the
  menu panel tracks the trigger after wheel-driven scroll updates (validated in
  `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` via
  `fret_dropdown_menu_tracks_trigger_when_underlay_scrolls`).
- Web submenu panel shadow gate (`shadow-lg`): `web_vs_fret_dropdown_menu_demo_submenu_shadow_matches_web`,
  `web_vs_fret_dropdown_menu_demo_submenu_shadow_matches_web_dark`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-demo.submenu-kbd.open.json`).
- Web submenu surface colors gate: `web_vs_fret_dropdown_menu_demo_submenu_surface_colors_match_web`,
  `web_vs_fret_dropdown_menu_demo_submenu_surface_colors_match_web_dark`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-demo.submenu.open.json`).
- Web submenu surface colors gate (kbd): `web_vs_fret_dropdown_menu_demo_submenu_kbd_surface_colors_match_web`,
  `web_vs_fret_dropdown_menu_demo_submenu_kbd_surface_colors_match_web_dark`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-demo.submenu-kbd.open.json`).
- Web submenu surface colors gate (kbd, tiny viewport): `web_vs_fret_dropdown_menu_demo_submenu_kbd_tiny_viewport_surface_colors_match_web`,
  `web_vs_fret_dropdown_menu_demo_submenu_kbd_tiny_viewport_surface_colors_match_web_dark`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-demo.submenu-kbd-vp1440x240.open.json`).
- Web submenu panel shadow gate (kbd, constrained viewport, `shadow-lg`): `web_vs_fret_dropdown_menu_demo_submenu_kbd_small_viewport_shadow_matches_web`,
  `web_vs_fret_dropdown_menu_demo_submenu_kbd_small_viewport_shadow_matches_web_dark`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-demo.submenu-kbd-vp1440x320.open.json`).
- Web submenu surface colors gate (kbd, constrained viewport): `web_vs_fret_dropdown_menu_demo_submenu_kbd_small_viewport_surface_colors_match_web`,
  `web_vs_fret_dropdown_menu_demo_submenu_kbd_small_viewport_surface_colors_match_web_dark`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-demo.submenu-kbd-vp1440x320.open.json`).
- Web submenu panel shadow gate (kbd, tiny viewport, `shadow-lg`): `web_vs_fret_dropdown_menu_demo_submenu_kbd_tiny_viewport_shadow_matches_web`,
  `web_vs_fret_dropdown_menu_demo_submenu_kbd_tiny_viewport_shadow_matches_web_dark`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-demo.submenu-kbd-vp1440x240.open.json`).
- Web first visible item gate (submenu): `web_vs_fret_dropdown_menu_demo_submenu_first_visible_matches`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-demo.submenu-kbd.open.json`).
- Web first visible item gate (submenu, constrained viewport): `web_vs_fret_dropdown_menu_demo_submenu_small_viewport_first_visible_matches`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-demo.submenu-kbd-vp1440x320.open.json`).
- Web menu row height gate (submenu): `web_vs_fret_dropdown_menu_demo_submenu_menu_item_height_matches`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-demo.submenu-kbd.open.json`).
- Web menu row height gate (submenu, hover): `web_vs_fret_dropdown_menu_demo_submenu_hover_menu_item_height_matches`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-demo.submenu.open.json`).
- Web menu content inset gate (submenu): `web_vs_fret_dropdown_menu_demo_submenu_menu_content_insets_match`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-demo.submenu-kbd.open.json`).
- Web menu content inset gate (submenu, hover): `web_vs_fret_dropdown_menu_demo_submenu_hover_menu_content_insets_match`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-demo.submenu.open.json`).
- Web placement gate (submenu, constrained viewport): `web_vs_fret_dropdown_menu_demo_submenu_small_viewport_overlay_placement_matches`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-demo.submenu-kbd-vp1440x320.open.json`).
- Web menu content inset gate (submenu, constrained viewport): `web_vs_fret_dropdown_menu_demo_submenu_small_viewport_menu_content_insets_match`
  (consumes `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-demo.submenu-kbd-vp1440x320.open.json`; note: this variant captures a scrolled menu state driven by the golden extraction openSteps).

Notes on scripted openSteps alignment:

- The web golden extractor scrolls the submenu trigger into view via `scrollIntoView({ block: "center" })` before focusing it and pressing ArrowRight.
- The Fret regression tests emulate this by (a) scrolling the menu via a deterministic wheel delta to center the trigger and (b) establishing focus via a pointer down/up with `is_click=false` (rather than relying on `UiTree::set_focus`, which is not equivalent under scroll/roving focus).
- Hover-driven submenus use delayed open timers; the test harness explicitly delivers timer events (because these tests do not run through the desktop runner's timer scheduling).

- Web overlay chrome gates (composition demos that use `DropdownMenu`): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_chrome`
  - `web_vs_fret_button_group_demo_surface_colors_match_web`
  - `web_vs_fret_button_group_demo_surface_colors_match_web_dark`
  - `web_vs_fret_button_group_demo_shadow_matches_web`
  - `web_vs_fret_button_group_demo_shadow_matches_web_dark`
  - `web_vs_fret_button_group_demo_submenu_kbd_surface_colors_match_web`
  - `web_vs_fret_button_group_demo_submenu_kbd_surface_colors_match_web_dark`
  - `web_vs_fret_button_group_demo_submenu_kbd_shadow_matches_web`
  - `web_vs_fret_button_group_demo_submenu_kbd_shadow_matches_web_dark`
  - `web_vs_fret_combobox_dropdown_menu_surface_colors_match_web`
  - `web_vs_fret_combobox_dropdown_menu_surface_colors_match_web_dark`
  - `web_vs_fret_combobox_dropdown_menu_shadow_matches_web`
  - `web_vs_fret_combobox_dropdown_menu_shadow_matches_web_dark`

## Follow-ups (recommended)

- Add icon/indicator slot conventions for menu rows (leading icon, checkmark/radio indicator, trailing shortcut).
- Decide whether dropdown menus need a “modal” option (or whether non-modal is the canonical Fret
  behavior).
- Consider adding a component-facing focus state for `Pressable` (mechanism-only) so menus can
  style the active item like shadcn (background highlight, not just focus ring).
- If we want Radix-level submenu ergonomics, consider:
  - intent-based safe-hover (`safePolygon` velocity heuristics), and
  - explicit keyboard focus transfer between parent menu and submenu.
