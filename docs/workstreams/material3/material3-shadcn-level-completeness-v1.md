# Material 3 shadcn-level completeness matrix v1

Status: Active
Last updated: 2026-06-01

This matrix tracks `ecosystem/fret-ui-material3` against the completeness bar we use for mature
`fret-ui-shadcn` recipes. It is not a visual-copying exercise: Material 3 remains spec/token-driven,
while shadcn is used as a Fret-side reference for API breadth, stable parts, semantics, behavior
gates, and gallery teaching surfaces.

## Axes

- API: public authoring surface covers the normal upstream use cases without bespoke app code.
- Parts: stable `test_id` anchors exist for root/trigger/content/important slots.
- Semantics: roles, labels, relations, and collection metadata are exposed.
- Behavior: keyboard/focus/dismiss/selection behavior is policy-complete for the component family.
- Tokens/style: Material tokens and ADR 0220-style override surfaces cover expected visual slots.
- Motion/ink: state layers, ripple, indicator, presence, or overlay motion are foundation-backed.
- Gallery/gates: first-party snippet plus targeted tests or diag scripts prove the important claims.

Legend: `Complete`, `Strong`, `Partial`, `MVP`, `Gap`.

## Component Matrix

| Component family | API | Parts | Semantics | Behavior | Tokens/style | Motion/ink | Gallery/gates | Notes |
|---|---|---:|---:|---:|---:|---:|---:|---|
| Button / FAB / IconButton | Strong | Strong | Strong | Strong | Strong | Strong | Strong | Main residual is broader expressive variants rather than shadcn-level surface shape. |
| Checkbox / Radio / Switch | Strong | Strong | Strong | Strong | Strong | Strong | Strong | Choice controls share `WidgetStates::SELECTED` and Material indication. |
| Slider / RangeSlider | Strong | Strong | Strong | Strong | Strong | Strong | Strong | Keep future work focused on higher-density examples and AT validation. |
| SegmentedButton | Strong | Strong | Strong | Strong | Strong | Strong | Strong | Single and multi-select are covered. |
| Tabs | Strong | Strong | Strong | Strong | Strong | Strong | Strong | 2026-06-01: gained force-mounted `TabPanel` content while keeping inactive panels out of semantics. Residuals: presence motion and richer overflow polish only if a concrete app gate needs them. |
| Menu / DropdownMenu | Strong | Strong | Strong | Strong | Strong | Strong | Strong | 2026-06-01: gained `MenuGroup`, clamped scrollable DropdownMenu viewports, and kit-backed Material submenu wiring with LTR/RTL keyboard gates. Residual: pointer-corridor diagnostics only if product hover flows need stronger evidence. |
| Select / ExposedDropdown / Autocomplete | Strong | Strong | Strong | Strong | Strong | Strong | Strong | Motion gates cover Select, Autocomplete, and ExposedDropdown chevron + overlay open/close fade/scale. |
| TextField | Strong | Strong | Strong | Strong | Strong | Strong | Strong | Floating-label and token work are mature; keep extending field-family compositions. |
| Dialog / BottomSheet | Strong | Strong | Strong | Strong | Strong | Strong | Strong | Dialog gates cover open/close fade + rise/scale; BottomSheet gates cover open/close sheet-height slide without panel fade. |
| Tooltip | Strong | Strong | Strong | Strong | Strong | Strong | Strong | Rich tooltips now expose an action slot that opts into hit-testable tooltip content while plain/descriptive tooltips remain click-through by default. |
| Snackbar | Strong | Strong | Strong | Strong | Strong | Strong | Strong | 2026-06-01: gained public `SnackbarStyle`, host `.style(...)`, direct Material style adoption in the shared toast renderer, and style override paint/layout gates. Residuals: richer app-level queue policy examples. |
| NavigationBar / Rail / Drawer | Strong | Strong | Strong | Strong | Strong | Strong | Strong | Routed-content composition, modal drawer close-after-selection, and NavigationBar RTL physical ordering are covered; continue polishing adaptive/RTL drawer details. |
| TopAppBar | Strong | Strong | Strong | Strong | Strong | Strong | Strong | Scroll behavior policy is in Material ecosystem code; nested-scroll consumption remains a future mechanism trigger. |
| DatePicker / TimePicker | Strong | Strong | Strong | Strong | Strong | Strong | Strong | Motion gates cover modal fade/rise/scale and TimePicker docked clock-face transitions; residuals are broader composition/demo coverage. |
| SearchBar / SearchView | Strong | Strong | Strong | Strong | Strong | Strong | Strong | `SearchBarStyle`/`SearchViewStyle` now cover visual slots; motion gates cover hover/ripple, docked fade/height, and full-screen geometry expand/collapse. |

## Current Batches

- 2026-05-31 Menu breadth:
  - `MenuLabel`, rich menu item slots, checkbox/radio item models, two-line row metrics, stable part
    ids, and gallery examples.
  - Gates: `menu_state`, `menu` / `dropdown_menu` lib tests, automation-surface test, clippy, layering.
- 2026-06-01 Menu group and long-menu viewport hardening:
  - `MenuGroup` now exposes a structural grouping API with group semantics while preserving parent
    menu collection metadata for nested items.
  - `DropdownMenu::max_height(...)` and the default Material dropdown max-height token path clamp
    tall overlays into a stable `.viewport` scroll part instead of sizing the popover to all rows.
  - The menu gallery now demonstrates grouped Edit/View sections.
  - Gates:
    `menu_state::{menu_group_wraps_entries_without_skewing_collection_metadata,
    dropdown_menu_long_content_uses_scrollable_material_viewport}`.
- 2026-06-01 Menu submenu wiring:
  - `MenuSubTrigger`, `MenuSubContent`, `MenuSub`, `MenuItem::submenu(...)`, and
    `DropdownMenu::submenu_min_width(...)` now expose a Material entry-tree authoring path for
    nested menus.
  - `DropdownMenu` reuses the existing `fret-ui-kit` Radix-aligned submenu policy models for hover
    intent, logical inline-end keyboard opening, close/focus restore, submenu content semantics, and
    long submenu viewport clamping.
  - The menu gallery now demonstrates a real Settings submenu instead of a static trailing chevron.
  - Gates:
    `menu_state::{dropdown_menu_submenu_opens_on_arrow_right_and_arrow_left_restores_trigger_focus,
    dropdown_menu_submenu_uses_rtl_inline_end_key_and_clamps_long_content}`.
- 2026-06-01 Conformance gate deepening:
  - The old crate-root minimum touch target source scan was replaced by a rendered Headless Surface
    gate that verifies Tabs, NavigationBar, NavigationRail, NavigationDrawer, Menu, and List row
    semantic bounds meet the Material 48dp touch target policy.
  - Literal Material token coverage is now owned by `tokens::coverage`, keeping source inventory and
    extraction locality in the token layer instead of `lib.rs`.
  - v30 injection gained the missing button/menu/search/date Material/Fret-specific token aliases
    used by recipe token modules.
  - Gates: `--lib::material3_literal_md_tokens_resolve_in_v30_theme` and
    `minimum_touch_target::navigable_material_rows_enforce_minimum_touch_target_at_runtime`.
- 2026-06-01 Token manifest and fixture hardening:
  - `material3_token_usage_manifest_v1.json` is now the structured source of truth for literal
    Material token usage across 121 recipe/foundation/interaction/token-module sources.
  - `tokens::coverage` validates manifest drift against source text, then resolves manifest tokens
    against v30; source scanning is no longer the coverage source of truth.
  - `tokens::usage` now centralizes the source inventory, literal/token-template classifier,
    internal test-token filtering, and template expansion rules shared by `tokens::coverage` and
    `material3_token_audit`.
  - `material3_token_audit` now reports the same 121 audited source files as the manifest gate,
    eliminating generated-preset/test-token false positives from the missing-injection report.
  - `material3_token_audit -- --check-usage-manifest` now gives maintainers a command-level stale
    fixture gate, and `--update-usage-manifest` regenerates the fixture from `tokens::usage`.
  - `minimum_touch_target` now consumes `material3_touch_target_cases_v1.json`, keeping future
    Material 48dp conformance additions case-only when they share the same harness.
  - v30 injection gained the real menu shape, navigation sizing, and time-picker separator aliases
    exposed by the wider manifest.
  - Gates: `--lib::material3_token_usage_manifest_matches_literal_md_sources`,
    `--lib::material3_literal_md_tokens_resolve_in_v30_theme`,
    `material3_token_audit -- --no-material-missing --limit 5`,
    `material3_token_audit -- --check-usage-manifest`, and
    `minimum_touch_target::navigable_material_rows_enforce_minimum_touch_target_at_runtime`.
- 2026-05-31 Tabs panel ownership:
  - `TabPanel`, `.panel/.panels`, active tabpanel semantics, `labelled_by` relation to the selected
    tab, derived tab `controls`, and gallery snippet content panels.
  - Gates: `tabs_state`, tabs automation-surface test, clippy, layering.
- 2026-06-01 Tabs force-mounted panel presence:
  - `TabPanel::force_mount(true)` keeps inactive panel subtrees mounted for retained state while
    routing visibility/interactivity through the existing `fret-ui-kit` Tabs primitive and
    `fret-ui` interactivity gate.
  - Material now renders every active or force-mounted panel, writes explicit panel `test_id`s to
    the `TabPanel` semantics node itself, and preserves the default inactive-unmounted behavior.
  - Scrollable Tabs were audited against the current gates; edge padding, minimum tab width, and
    indicator geometry remain covered, with no new core scroll mechanism introduced in this batch.
  - Gates: `tabs_state::tabs_force_mounted_panels_stay_mounted_but_only_active_panel_is_semantic`
    plus the existing scrollable primary/secondary metric tests.
- 2026-06-01 Snackbar style/API hardening:
  - `SnackbarStyle` now follows ADR 0220 for container/supporting/action/close colors plus
    container shape, padding, and single/two-line heights.
  - `fret-ui-kit` toast rendering now consumes direct `ToastLayerStyle` background/foreground,
    text, border, action, cancel, and close style fields instead of hardcoding the Sonner skin.
  - Gates: `window_overlays::tests::toast::toast_layer_style_direct_colors_are_painted`,
    `snackbar_state` including `snackbar_style_overrides_paint_and_layout_contract`.
- 2026-06-01 Tooltip visual style/API and rich-action hardening:
  - `TooltipStyle` follows ADR 0220 for plain/rich surface colors, text colors/text styles, shape,
    padding, max width, min size, rich elevation/shadow, rich text gap, and rich action label/row
    slots.
  - `RichTooltip::action_element(...)` now exposes the Material rich tooltip action slot and uses a
    `fret-ui-kit` tooltip hit-test opt-in so the action can receive pointer input without changing
    the plain tooltip click-through default.
  - Gallery examples now include styled plain/rich variants and an action-bearing rich tooltip.
  - Gates: `tooltip_state` including
    `plain_tooltip_style_overrides_paint_and_layout_contract` and
    `rich_tooltip_style_overrides_paint_parts_and_layout_contract`, plus
    `rich_tooltip_action_slot_is_hit_testable_and_stable`.
- 2026-06-01 Search style/API and motion evidence calibration:
  - `SearchBarStyle` now covers search field surface, shape, sizing, row padding/gap, input text,
    icon colors, and state-layer color.
  - `SearchViewStyle` now covers overlay container/divider/body/header slots and forwards
    `SearchBarStyle` to docked/full-screen search headers.
  - Gates: `search_bar_motion` and `search_view_behavior`, including style paint/layout tests plus
    existing hover/ripple and docked/full-screen motion tests.
- 2026-06-01 Date/Time motion evidence calibration:
  - DatePicker modal motion is already covered by fixed-frame fade + rise/scale tests.
  - TimePicker modal motion is covered for dial and input modes; docked clock-face transitions are
    covered by crossfade and selector-translation tests.
  - Gates: `date_picker_motion`, `time_picker_motion`.
- 2026-06-01 Dialog/BottomSheet motion hardening:
  - Dialog motion evidence was already present in `dialog_state` for open/close fade + rise/scale.
  - BottomSheet motion now asserts both open and close sheet-height slide and guards against
    accidental panel fade; scrim alpha remains token-driven paint, not panel opacity.
  - Gates: `dialog_state::dialog_scrim_and_panel_animate_on_open_close_frames`,
    `bottom_sheet_motion::modal_bottom_sheet_slides_from_own_height_without_panel_fade`.
- 2026-06-01 Select/Autocomplete motion evidence calibration:
  - Select motion gates cover chevron rotation and overlay fade/scale on open and close.
  - Autocomplete and ExposedDropdown motion gates cover chevron rotation and popup fade/scale on
    open and close.
  - Gates: `select_behavior::select_chevron_rotates_on_first_open_frame`,
    `autocomplete_motion::{autocomplete_popup_and_chevron_animate_on_open_close_frames,
    exposed_dropdown_popup_and_chevron_animate_on_open_close_frames}`.
- 2026-06-01 Field-overlay composition inside Dialog:
  - `Select` and `Autocomplete` now have dialog-nested overlay gates that prove popover-above-modal
    stacking, first-Escape nested dismissal, focus restoration/retention inside the dialog, and
    second-Escape modal dismissal.
  - Gates: `material3_overlay_interactions::{select_inside_dialog_closes_inner_popover_before_modal_dialog,
    autocomplete_inside_dialog_escape_closes_inner_popover_before_modal_dialog}` plus diag scripts
    `ui-gallery-material3-dialog-select-nested-overlay.json` and
    `ui-gallery-material3-autocomplete-dialog-nested-overlay.json`.
- 2026-06-01 Field-overlay composition inside ModalBottomSheet:
  - The BottomSheet gallery now renders real `TextField`, `Select`, and `Autocomplete` controls with
    caller-owned full-width sheet content layout, stable field/listbox/item anchors, and a diag gate
    that catches zero-width/hit-test regressions before interaction.
  - Gates: `material3_overlay_interactions::field_overlays_inside_modal_bottom_sheet_close_before_sheet`
    plus `ui-gallery-material3-bottom-sheet-fields-nested-overlays.json`.
- 2026-06-01 SearchView + Menu sibling overlay composition:
  - Docked `SearchView` now keeps input focus when suggestions open by assigning input initial focus
    to its non-modal popover request.
  - The Menu gallery includes a Search + Menu composition with caller-owned `SearchView` width,
    stable Search/panel/suggestion/menu anchors, and a diag gate covering query editing, sibling
    overlay dismissal, menu role, first-item focus, and `Escape` focus restore.
  - Gates: `material3_overlay_interactions::search_view_and_dropdown_menu_arbitrate_sibling_popovers`
    plus `ui-gallery-material3-search-menu-sibling-popovers.json`.
- 2026-06-01 SearchView edge/full-screen composition:
  - Docked `SearchView` now has a focused bottom-edge collision test proving the shared popper
    solver flips above the input, clamps height, and keeps the overlay inside the window collision
    boundary.
  - Full-screen `SearchView` now has a sibling `DropdownMenu` composition test proving modal-layer
    ownership, overlay-header focus, query preservation, sibling trigger blocking while modal, and
    menu focus takeover after dismissal.
  - The Material Menu gallery includes bottom-edge and full-screen SearchView repro anchors, with a
    diag script covering real-page panel bounds, dialog role, modal blocking, and post-dismiss menu
    focus.
  - Gates:
    `search_view_behavior::search_view_docked_overlay_flips_and_clamps_near_viewport_bottom`,
    `material3_overlay_interactions::search_view_full_screen_blocks_sibling_menu_until_dismissed`,
    `material3_search_view_surface`, and
    `ui-gallery-material3-search-view-edge-fullscreen-composition.json`.
- 2026-06-01 Navigation routed-content composition:
  - `NavigationBar`, `NavigationRail`, and `NavigationDrawer` gallery snippets now render
    caller-owned route panels with stable route-panel and active-route anchors.
  - A shared-route regression test proves Bar/Rail/Drawer selected semantics stay synchronized
    across one route model and that stale route panels unmount as destinations change.
  - Gates: `material3_navigation_interactions::navigation_surfaces_drive_routed_panel_content`
    plus `ui-gallery-material3-navigation-routed-content.json`.
- 2026-06-01 NavigationBar RTL direction hardening:
  - `NavigationBar` now resolves the Material theme default layout direction and provides it to its
    subtree, matching the foundation pattern already used by `Tabs`.
  - RTL now proves both logical keyboard movement (`ArrowLeft` moves to the next destination) and
    physical destination-order mirroring.
  - Gates: `navigation_state::{navigation_bar_rtl_arrow_left_moves_to_next_logical_destination,
    navigation_bar_rtl_theme_direction_mirrors_physical_destination_order}`.
- 2026-06-01 NavigationDrawer RTL slot mirroring:
  - `NavigationDrawer` now resolves the Material theme default layout direction and provides it to
    its subtree, so drawer item text/flex layout receives the same direction context as Bar/Tabs.
  - Drawer items keep the Compose Material3 logical row order (`icon -> label -> badge`) while
    using logical inline padding (`start=16dp`, `end=24dp`) through the shared Material logical-edge
    helper.
  - Gates:
    `navigation_state::navigation_drawer_rtl_theme_direction_mirrors_item_slots_and_padding`.
- 2026-06-01 ModalNavigationDrawer routed-content composition:
  - `NavigationDrawerItem::on_select(...)` exposes the caller-owned destination activation hook
    needed to model Compose-style `selected = item; drawerState.close()` flows without baking modal
    close policy into the drawer recipe.
  - The ModalNavigationDrawer gallery now renders caller-owned route panels and closes the drawer on
    explicit destination activation while preserving route selection and focus restoration.
  - Gates:
    `material3_navigation_interactions::modal_navigation_drawer_drives_routed_content_and_closes_on_destination_activation`
    plus `ui-gallery-material3-modal-navigation-drawer-routed-content.json`.

## Next Recommended Focus

1. Add Tabs presence motion or richer scroll affordances only after a concrete product gate proves
   the current force-mounted presence contract or scrollable metric coverage is insufficient.
2. Add a pointer-corridor diagnostics replay for Material submenus only if a product flow depends
   on high-speed hover travel between root menu rows and submenu panels.
3. Extend real-device/mobile IME and inset proof for overlay-heavy Material flows if mobile search
   and bottom-sheet surfaces become product priorities.
