# shadcn/ui new-york-v4 Alignment Audit (Fret)

This audit tracks visual/behavior alignment gaps between:

- Upstream baseline: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/*.tsx`
- Fret recipes: `ecosystem/fret-ui-shadcn/src/*.rs`

Goal: align **default outcomes** (spacing, sizing, truncation, focus ring, indicator slots, overlay chrome)
for the `new-york-v4` preset, without expanding `fret-ui` mechanism scope.

For coverage status (what is gated vs only has goldens), see:

- `docs/audits/shadcn-new-york-v4-coverage.md`
- Chart-specific audit: `docs/audits/shadcn-chart.md`

Coverage snapshot (time of writing):

- shadcn-web `v4/new-york-v4`: `385/448` keys referenced (`85.9%`)

Heuristic “where we already have gates” (top key families by prefix):

- `calendar` (34), `input` (27), `button` (26), `form` (19), `navigation` (17), `sidebar` (16)

## Executive summary (current status + next targets)

This audit intentionally separates “breadth coverage” (what is gated at all) from “depth conformance”
(how close we are to upstream across all variants, viewports, DPIs, and font metrics).

### What is currently high-signal (already gated somewhere)

These areas have multiple geometry + overlay placement/chrome gates and tend to catch regressions early:

- **Menus / listboxes**: `DropdownMenu`, `Select`, `Menubar` (panel chrome, placement, constrained viewport
  max-height behavior, and menu row height as a styling outcome).
- **Sidebar blocks**: sidebar menu-button heights (including the “collapsed overrides size” rule), and a dialog
  portal placement gate for `sidebar-13` (`*.open.json`).
- **Calendar (single-month variants)**: day grid geometry + ARIA label strings; outside-days + week number
  behaviors that are easy to drift when refactoring.
- **Typography**: breadth gates for `typography-*` pages (geometry + recorded text style inputs; inline-code padding tolerates sub-pixel rounding).

These gates do **not** imply full parity; they are simply the most effective early tripwires we have so far.

Recent breadth wins:

- **Auth blocks**: `login-*`, `signup-*`, `otp-*` now have shell container gates; `otp-01/02/03/05` also gate the
  InputOtp row geometry (slot sizes + gaps).
- **Recurring layout families**: `textarea-*`, `empty-*`, `resizable-*`, `native-select-*` now have baseline layout gates.
- **Field + date + skeleton edges**: `field-responsive`, `button-as-child`, `date-picker-with-range`, `skeleton-*` now have web-vs-fret layout gates.
- **Dashboard block shell**: `dashboard-01` now has a shell geometry gate (sidebar width + header inset geometry).
- **Chart tooltip/legend wrapper**: initial `chart-tooltip-*` + `chart-*-legend` panel geometry gates (min-width + padding + line-height outcomes).
  - Tooltip variants now include `chart-tooltip-label-none`, `chart-tooltip-label-custom`, `chart-tooltip-label-formatter`, `chart-tooltip-icons`,
    `chart-tooltip-formatter`, `chart-tooltip-advanced`.
  - Pie legend variant includes `chart-pie-legend` (recharts wrapper + shadcn `*:basis-1/4` layout).

### Largest remaining gaps (by golden family)

From `tools/golden_coverage.ps1 -GroupMissingByPrefix`:

- `chart` (63 variants): large surface area; likely needs a dedicated alignment push.

### Recommended next alignment targets (P0 order)

1. **Chart push**
   - Treat `chart-*` as a dedicated sprint (surface area is large; likely needs new audit notes + more selective gates).
   - Plan + scope: `docs/audits/shadcn-chart.md`.

When these are in place, it becomes much more cost-effective to add **DPI** and **viewport** variants as a
second wave (because we can keep the matrix small and stable).

## How to validate

- Run the component gallery: `cargo run -p fret-demo --bin components_gallery`
- Validate controls at multiple DPIs and with a “weird metrics” UI font (e.g. a Nerd Font) to catch
  baseline/centering issues.

## Viewport strategy (golden gates)

We prioritize **breadth of component coverage** first (one canonical desktop viewport per example),
then layer in a small set of “stress” viewports to catch responsive and clamping behavior.

Recommended minimal viewport matrix:

- **Desktop baseline**: large enough to avoid incidental clamping (the default web goldens).
- **Mobile width** (`375px` wide): catches truncation, wrap, and `justify-between` slot behavior.
- **Constrained height** (`375x320`-ish): catches overlay max-height, scroll buttons, and “menu height”
  regressions that are styling outcomes (padding/border/row heights).

This keeps the test suite tractable while still guarding the highest-risk geometry behaviors early.

## DPI / font metrics strategy

We treat DPI and font metrics variance as a second dimension (orthogonal to viewport size):

- Viewport variants (width/height constraints) catch **layout policy** bugs (clamp/flip/max-height,
  scroll buttons, truncation, `justify-between` slot behavior).
- DPI and font variance catch **measurement + baseline** bugs (text ascent/descent, icon centering,
  "1px off" rounding drift) that can pass at one scale but fail at another.

Current policy:

- Keep the golden gate matrix small while we are still filling breadth.
- For high-risk pages (menus, listboxes, calendars, typography), run spot checks at a few DPIs and
  a "weird metrics" font during alignment work, then add targeted gates only when we can keep them
  stable and deterministic.

## Infra notes (golden + scroll + paint cache)

- Scroll offsets are applied as a children-only render transform. `PaintCx::paint()` must see the
  parent node's children transform even while the widget is temporarily removed from the node
  (during `with_widget_mut`).
- `PaintCacheKey` includes the node's children render transform so cache replay cannot cross
  scroll/transform changes without invalidation.
- Regression test: `crates/fret-ui/src/tree/tests/scroll_invalidation.rs`
  (`scroll_offset_changes_do_not_replay_paint_cache`).
- Web golden extractor notes:
  - `goldens/shadcn-web/scripts/extract-golden.mts` exports `aria-invalid` and supports scripted
    attribute injection steps for variant snapshots (e.g. `*.invalid.json`).

## Radix primitives: occlusion + focus restore

These are core correctness behaviors that many shadcn components depend on (menus, popovers,
navigation-menu, tooltip/hover-card, etc.). They should be treated as “must stay aligned” contracts
and guarded by focused regression tests.

- Pointer occlusion (`disableOutsidePointerEvents`): suppress hit-tested pointer dispatch (including
  mouse move) to underlay layers while the overlay is open, but still allow wheel routing and keep
  pointer-move observers active for overlay policies (e.g. menu safe corridor).
  - Implementation: `crates/fret-ui/src/tree/dispatch.rs`
  - Regression: `ecosystem/fret-ui-kit/src/window_overlays/tests.rs`
    (`non_modal_overlay_can_disable_outside_pointer_events_while_open`)
- Focus restore (non-modal overlays): closing/unmounting must not override a new underlay focus
  (outside press can legitimately move focus under the pointer).
  - Implementation: `ecosystem/fret-ui-kit/src/window_overlays/render.rs`
  - Regressions: `ecosystem/fret-ui-kit/src/window_overlays/tests.rs`
    (`popover_outside_press_closes_without_overriding_new_focus`,
    `non_modal_overlay_does_not_restore_focus_when_focus_moves_to_underlay_on_unmount`)

## Global baseline rules (new-york-v4)

These patterns appear repeatedly across upstream components:

- Control height: `h-9` (default), `h-8` (sm), `h-10` (lg) for buttons; select trigger uses `h-9`/`h-8`.
- Radius: `rounded-md` for controls; `rounded-lg` for larger containers (e.g. dialog).
- Truncation: value/title areas use `min-w-0` + `overflow-hidden` + `whitespace-nowrap` + `truncate`
  (or `line-clamp-1`), especially for trigger/value slots inside a `justify-between` row.
- Icon sizing: default `size-4`, often with `opacity-50` for down chevrons.
- Indicator slots: checkbox/radio/select indicators are positioned via `absolute` + reserved padding
  (`pl-8` or `pr-8`) so layout does not jitter when toggling selection.
- Focus ring: `focus-visible:ring-[3px]` with `ring/50` and `border-ring`.

Fret mapping intent:

- Use `fret-ui-kit::recipes::input::resolve_input_chrome` (and/or shadcn theme presets) for shared
  control chrome and ring behavior.
- Use recipe-level layout helpers to guarantee “min width 0 + truncation” on value/title slots.
- Prefer stable indicator slot layout (reserve space, avoid `SpaceBetween` relying on icon presence).

## Component checklist (high impact)

### `Select`

- Upstream: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/select.tsx`
- Fret: `ecosystem/fret-ui-shadcn/src/select.rs`
- Gaps to check:
  - Trigger: ensure value slot truncation is always enabled (no premature ellipsis).
  - Trigger: chevron icon size/opacity and gap to value.
  - Content: `p-1`, `rounded-md`, `border`, `shadow-md`, max-height behavior.
  - Content: web uses a “min width = trigger, but allow wider for long items” outcome; Fret matches
    this via a longest-item width probe in item-aligned mode (so listbox width can exceed trigger).
  - Items: `py-1.5`, `pl-2`, `pr-8`, `gap-2`, `rounded-sm`.
  - Selected indicator: absolute slot (`right-2`, `size-3.5`) + reserve `pr-8`.
  - Scroll buttons: `py-1` with centered `size-4` chevrons; when scrollable, Radix scrolls the
    viewport so the first selectable option sits directly under the up-button (the group label is
    scrolled behind it).

Recent fixes:

- Trigger sizing now matches new-york-v4 defaults (no forced `w-full`/`min-w` on the trigger; dropdown min width defaults to `8rem`).
- Trigger chrome/content width now tracks the trigger width mode (auto vs fixed), preventing “ellipsis even when there is space” cases.
- `fret-icons-radix` now vendors `chevron-up.svg`, so Radix-backed semantic `ui.chevron.up` resolves correctly.
- Trigger `aria-invalid` border + focus ring (including shadcn's invalid ring override colors) now match shadcn-web (`select-demo.invalid`, `select-demo.invalid-focus`).

Conformance gates:

- Chrome: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome.rs` (`web_vs_fret_select_panel_chrome_matches`).
- Trigger chrome + focus ring: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs` (`web_vs_fret_select_scrollable_trigger_chrome_matches`, `web_vs_fret_select_demo_aria_invalid_border_color_matches`, `web_vs_fret_select_demo_focus_ring_matches`, `web_vs_fret_select_demo_aria_invalid_focus_ring_matches`).
- Placement: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` (`web_vs_fret_select_scrollable_overlay_placement_matches`, `web_vs_fret_select_scrollable_small_viewport_overlay_placement_matches`).
- Scroll buttons + viewport inset: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` (`web_vs_fret_select_scrollable_listbox_option_insets_match`, `web_vs_fret_select_scrollable_small_viewport_listbox_option_insets_match`).

### `Calendar`

- Upstream: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/calendar.tsx`
- Fret: `ecosystem/fret-ui-shadcn/src/calendar.rs`, `ecosystem/fret-ui-shadcn/src/calendar_range.rs`, `ecosystem/fret-ui-shadcn/src/calendar_multiple.rs`
- Gaps to check:
  - Grid row count (4/5/6 weeks) under different months/week-start values.
  - Cell sizing: `--cell-size` drives nav buttons, caption height/padding, and day cell min width.
  - `showOutsideDays=false`: outside-month slots are `invisible` but still occupy grid space (do not
    collapse columns).
  - `showWeekNumber=true`: week number column shifts the weekday + day grid by one cell.
  - ARIA label strings for day buttons (including `Today, ` prefix + `, selected` suffix).

Recent fixes:

- Calendar month grid now matches `react-day-picker`'s compact row behavior via `month_grid_compact`.
- Multi-month (`numberOfMonths=2`) parity is now gated for `calendar-02/05/07/09/11/12`:
  - Shared absolute `nav` over the months container.
  - Month-bounds gating (`startMonth`/`endMonth`) and `disableNavigation` parity (`calendar-11`).
  - Locale-aware month titles + day labels for Spanish (`calendar-12`).
- Multiple selection (`mode="multiple"`) parity is now gated for `calendar-03` (including `required` + `max` selection policy).
- Popover date picker composition open-state overlay placement is now gated for `calendar-22.open` through `calendar-26.open` (`align="start"`, `sideOffset=4`).
- Day buttons support `--cell-size` variants (per-golden) via `Calendar::{cell_size, show_week_number}` and
  nav button sizing.
- `showOutsideDays=false` now keeps invisible outside-day placeholders (a11y-hidden) so x/y geometry matches.
- Day button `aria-label` now matches shadcn-web goldens including `Today, ` prefix and `, selected` suffix.
- Week numbers use `fret-ui-headless::calendar::week_number` (DayPicker-like `getWeek` defaults).

Conformance gates:

- Layout + a11y labels: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs` (`web_vs_fret_layout_calendar_demo_day_grid_geometry_and_a11y_labels_match_web`).
- Variant geometry (single-month): `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs` (`web_vs_fret_layout_calendar_*_geometry_matches`).
- Variant geometry (multi-month): `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs` (`web_vs_fret_layout_calendar_{02,03,05,07,09,11,12}_geometry_matches`).
- Popover open-state placement: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` (`web_vs_fret_calendar_22_open_overlay_placement_matches`).

### `DropdownMenu`

- Upstream: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/dropdown-menu.tsx`
- Fret: `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`
- Gaps to check:
  - Content: `p-1`, `rounded-md`, `border`, `shadow-md`, max-height behavior.
  - Content: width is `min-w-[8rem]` but can grow to fit longer labels (e.g. `breadcrumb-responsive`).
  - Item row: `px-2 py-1.5`, `gap-2`, `rounded-sm`, destructive focus background tint.
  - Checkbox/radio indicators: absolute left slot (`left-2`, `size-3.5`) + reserve `pl-8`.
  - SubTrigger: right chevron `ml-auto size-4`, `data-[state=open]` accent background.
  - Shortcut: `ml-auto text-xs tracking-widest` alignment.

Recent fixes:

- Destructive item focus tint now matches upstream (`destructive/10` in light, `destructive/20` in dark) via seeded theme tokens.
- Menu panel width now grows beyond the `8rem` floor when labels are longer (using a longest-label estimate for popper sizing).
- Submenu hover-corridor stability now matches Radix pointer-grace behavior: the submenu trigger element is keyed by overlay id and the last-known trigger anchor is cached so close-delay transitions do not transiently drop the anchor (which would collapse the submenu portal).

Conformance gates:

- Chrome: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome.rs` (`web_vs_fret_dropdown_menu_panel_chrome_matches`).
- Shadow (`shadow-md`): `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome.rs`
  (`web_vs_fret_dropdown_menu_demo_shadow_matches_web`, `web_vs_fret_dropdown_menu_demo_shadow_matches_web_dark`).
- SubContent shadow (`shadow-lg`): `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome.rs`
  (`web_vs_fret_dropdown_menu_demo_submenu_shadow_matches_web`, `web_vs_fret_dropdown_menu_demo_submenu_shadow_matches_web_dark`).
- Placement: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` (`web_vs_fret_dropdown_menu_demo_overlay_placement_matches`, `web_vs_fret_dropdown_menu_checkboxes_overlay_placement_matches`, `web_vs_fret_dropdown_menu_radio_group_overlay_placement_matches`, `web_vs_fret_dropdown_menu_demo_small_viewport_overlay_placement_matches`, `web_vs_fret_dropdown_menu_demo_submenu_overlay_placement_matches`, `web_vs_fret_dropdown_menu_demo_submenu_small_viewport_overlay_placement_matches`).
- Menu row height: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` (`web_vs_fret_dropdown_menu_demo_small_viewport_menu_item_height_matches`).
- Item row padding + shortcut alignment: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs`
  (`web_vs_fret_dropdown_menu_demo_profile_item_padding_and_shortcut_match`).
- Checkbox/radio indicator slot inset: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs`
  (`web_vs_fret_dropdown_menu_checkboxes_checkbox_indicator_slot_inset_matches_web`, `web_vs_fret_dropdown_menu_radio_group_radio_indicator_slot_inset_matches_web`).
- Menu content insets: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` (`web_vs_fret_dropdown_menu_demo_small_viewport_menu_content_insets_match`, `web_vs_fret_dropdown_menu_demo_submenu_small_viewport_menu_content_insets_match`).

### `Command` / `CommandDialog`

- Upstream: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/command.tsx`
- Upstream demos: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/command.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/command-dialog.tsx`
- Fret: `ecosystem/fret-ui-shadcn/src/command.rs`
- Notes:
  - `CommandDialog` is a `Dialog`-backed recipe that composes `CommandPalette` into a modal.
  - The upstream demo opens via a key chord (`Ctrl/Cmd+J`); the golden extractor treats it as `openAction=keys`.
- Conformance gates:
  - Layout (command demo): `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs`
    (`web_vs_fret_layout_command_demo_input_height_matches`, `web_vs_fret_layout_command_demo_listbox_height_matches`,
    `web_vs_fret_layout_command_demo_listbox_option_height_matches`, `web_vs_fret_layout_command_demo_listbox_option_insets_match`).
  - Chrome: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome.rs` (`web_vs_fret_command_dialog_panel_chrome_matches`).
  - Placement: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` (`web_vs_fret_command_dialog_overlay_center_matches`).
  - List metrics: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs`
    (`web_vs_fret_command_dialog_input_height_matches`, `web_vs_fret_command_dialog_listbox_height_matches`,
    `web_vs_fret_command_dialog_listbox_option_height_matches`, `web_vs_fret_command_dialog_listbox_option_insets_match`).

### `ContextMenu`

- Upstream: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/context-menu.tsx`
- Fret: `ecosystem/fret-ui-shadcn/src/context_menu.rs`
- Gaps to check:
  - Content: `p-1`, `rounded-md`, `border`, `shadow-md`, max-height behavior.
  - SubContent: `rounded-md`, `border`, `shadow-lg`, and submenu focus/close rules.
  - Item rows: `rounded-sm` + focus tint + destructive variant tint.

Recent fixes:

- Panel chrome now matches upstream `rounded-md` (radius token) and `shadow-md` / `shadow-lg` split.
- Conformance gates:
  - Chrome: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome.rs` (`web_vs_fret_context_menu_panel_chrome_matches`).
  - Shadow (`shadow-md`): `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome.rs`
    (`web_vs_fret_context_menu_demo_shadow_matches_web`, `web_vs_fret_context_menu_demo_shadow_matches_web_dark`).
  - SubContent shadow (`shadow-lg`): `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome.rs`
    (`web_vs_fret_context_menu_demo_submenu_shadow_matches_web`, `web_vs_fret_context_menu_demo_submenu_shadow_matches_web_dark`).
  - Placement: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` (`web_vs_fret_context_menu_demo_overlay_placement_matches`, `web_vs_fret_context_menu_demo_small_viewport_overlay_placement_matches`, `web_vs_fret_context_menu_demo_submenu_overlay_placement_matches`, `web_vs_fret_context_menu_demo_submenu_small_viewport_overlay_placement_matches`).
  - Menu row height: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` (`web_vs_fret_context_menu_demo_small_viewport_menu_item_height_matches`).
  - Checkbox/radio indicator slot inset: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs`
    (`web_vs_fret_context_menu_demo_checkbox_indicator_slot_inset_matches_web`, `web_vs_fret_context_menu_demo_radio_indicator_slot_inset_matches_web`).
  - Item row padding + shortcut/chevron alignment: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs`
    (`web_vs_fret_context_menu_demo_back_item_padding_and_shortcut_match`).
  - Menu content insets: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` (`web_vs_fret_context_menu_demo_small_viewport_menu_content_insets_match`, `web_vs_fret_context_menu_demo_submenu_small_viewport_menu_content_insets_match`).

### `Menubar`

- Upstream: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/menubar.tsx`
- Fret: `ecosystem/fret-ui-shadcn/src/menubar.rs`
- Gaps to check:
  - Root: `h-9`, `rounded-md`, `border`, `p-1`, `shadow-xs`, `gap-1`.
  - Content: `rounded-md`, `border`, `p-1`, `shadow-md`.
  - SubContent: `rounded-md`, `border`, `p-1`, `shadow-lg`.

Recent fixes:

- Root panel chrome now matches upstream `rounded-md` + `shadow-xs` (new-york-v4 baseline).
- Menu panels now match upstream `rounded-md` + `p-1` and `shadow-md` / `shadow-lg` split.
- Menu panel width now grows beyond the `min-w-[12rem]` baseline when long checkbox/radio labels
  require it (e.g. View/Profiles menus), matching upstream sizing behavior.
- Note: The current "grow to fit" sizing path uses a deterministic text-width heuristic. If we need
  stronger 1:1 guarantees across font stacks, we should expose a text measurement service and use
  real glyph metrics instead of a character-count estimate.
- Conformance gates:
  - Chrome: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome.rs` (`web_vs_fret_menubar_panel_chrome_matches`).
  - Root shadow (`shadow-xs`): `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome.rs`
    (`web_vs_fret_menubar_root_shadow_matches_web`, `web_vs_fret_menubar_root_shadow_matches_web_dark`).
  - Shadow (`shadow-md`): `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome.rs`
    (`web_vs_fret_menubar_demo_shadow_matches_web`, `web_vs_fret_menubar_demo_shadow_matches_web_dark`).
  - SubContent shadow (`shadow-lg`): `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome.rs`
    (`web_vs_fret_menubar_demo_submenu_shadow_matches_web`, `web_vs_fret_menubar_demo_submenu_shadow_matches_web_dark`).
  - Placement: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` (`web_vs_fret_menubar_demo_overlay_placement_matches`, `web_vs_fret_menubar_demo_view_overlay_placement_matches`, `web_vs_fret_menubar_demo_profiles_overlay_placement_matches`, `web_vs_fret_menubar_demo_small_viewport_overlay_placement_matches`, `web_vs_fret_menubar_demo_submenu_overlay_placement_matches`, `web_vs_fret_menubar_demo_submenu_small_viewport_overlay_placement_matches`).
  - Menu row height: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` (`web_vs_fret_menubar_demo_menu_item_height_matches`, `web_vs_fret_menubar_demo_view_menu_item_height_matches`, `web_vs_fret_menubar_demo_profiles_menu_item_height_matches`, `web_vs_fret_menubar_demo_small_viewport_menu_item_height_matches`).
  - Checkbox/radio indicator slot inset: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs`
    (`web_vs_fret_menubar_demo_view_checkbox_indicator_slot_inset_matches_web`, `web_vs_fret_menubar_demo_profiles_radio_indicator_slot_inset_matches_web`).
  - Item row padding + shortcut/chevron alignment: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs`
    (`web_vs_fret_menubar_demo_item_padding_and_shortcut_match`).
  - Menu content insets: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` (`web_vs_fret_menubar_demo_menu_content_insets_match`, `web_vs_fret_menubar_demo_view_menu_content_insets_match`, `web_vs_fret_menubar_demo_profiles_menu_content_insets_match`, `web_vs_fret_menubar_demo_small_viewport_menu_content_insets_match`, `web_vs_fret_menubar_demo_submenu_small_viewport_menu_content_insets_match`).

Note: for menu-like overlays (DropdownMenu / ContextMenu / Menubar), the placement gate also asserts
the portal panel `w/h` against the shadcn-web portal wrapper geometry (so “menu height” regressions
are caught as layout/style outcomes, not just placement drift).

Radix focus restore semantics are also gated (radix-web timelines):

- DropdownMenu: OutsidePress + Escape restore focus to the trigger.
- Menubar: OutsidePress clears focus; Escape restores focus to the trigger.
- ContextMenu: OutsidePress + Escape clear focus.

Implementation anchors: dismiss-cause-aware focus restore in `ecosystem/fret-ui-kit/src/window_overlays/render.rs`, with per-wrapper policy in `ecosystem/fret-ui-shadcn/src/menubar.rs` and `ecosystem/fret-ui-shadcn/src/context_menu.rs`.

### `NavigationMenu`

- Upstream: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/navigation-menu.tsx`
- Fret: `ecosystem/fret-ui-shadcn/src/navigation_menu.rs`
- Gaps to check:
  - `viewport=false` content panel: `rounded-md border shadow` with `mt-1.5` offset and `p-2 pr-2.5`.
  - `viewport=true` viewport panel: `rounded-md border shadow` with zoom motion `zoom-in-90` / `zoom-out-95`.

Recent fixes:

- `viewport=false` chrome/placement now match shadcn-web `navigation-menu-demo` open snapshots, including hover-switch (`home-then-hover-components`).
- `viewport=true` viewport geometry now matches shadcn-web mobile snapshots, including “click then hover” switching (`home-mobile-then-hover-components`) and the viewport/indicator chrome gates.
- Conformance gates:
  - Chrome: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome.rs` (`web_vs_fret_navigation_menu_demo_panel_chrome_matches`, `web_vs_fret_navigation_menu_demo_viewport_*`, `web_vs_fret_navigation_menu_demo_indicator_*`).
  - Placement + geometry: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` (`web_vs_fret_navigation_menu_demo_*`, including mobile viewport height/width and hover-switch coverage).

### `Input`

- Upstream: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/input.tsx`
- Fret: `ecosystem/fret-ui-shadcn/src/input.rs`
- Gaps to check:
  - Ensure `min-w-0` equivalent for flex layouts.
  - Focus ring thickness (`3px`) and border color keys.
  - Placeholder color and selection colors.
- Recent fixes:
  - `aria-invalid=true` border color now matches shadcn-web (`input-demo.invalid`) and is gated via
    `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs`
    (`web_vs_fret_input_demo_aria_invalid_border_color_matches`).
  - Focus ring (`ring-[3px]`) and ring color overrides now match shadcn-web focus variants
    (`input-demo.focus`, `input-demo.invalid-focus`) and are gated via
    `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs`
    (`web_vs_fret_input_demo_focus_ring_matches`, `web_vs_fret_input_demo_aria_invalid_focus_ring_matches`).
- Note: shadcn's `aria-invalid:ring-*` is a ring color override; the ring only becomes visible when
  `focus-visible:ring-[3px]` is also active.

### `InputGroup`

- Upstream: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/input-group.tsx`
- Fret: `ecosystem/fret-ui-shadcn/src/input_group.rs`
- Notes:
  - Upstream draws chrome (border + `shadow-xs`) on the group root and makes the inner control
    borderless (`border-0`, `rounded-none`, `bg-transparent`, `shadow-none`).
  - The group root uses `has-[[data-slot=input-group-control]:focus-visible]...` for focus-within
    ring/border. Fret maps this via container-level focus-within chrome:
    `ContainerProps.focus_within=true` + `focus_border_color` + `focus_ring`.
- Recent fixes:
  - Inline addon layout now matches shadcn-web geometry: addons participate in normal flex flow
    instead of absolute slots; input padding switches to `pl-2` / `pr-2` when an inline addon is
    present.
  - `aria-invalid=true` border color now matches shadcn-web (`input-group-demo.invalid`) and is
    gated via `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs`
    (`web_vs_fret_input_group_demo_aria_invalid_border_color_matches`).
  - Focus ring (`ring-[3px]`) and ring color overrides now match shadcn-web focus variants
    (`input-group-demo.focus`, `input-group-demo.invalid-focus`) and are gated via
    `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs`
    (`web_vs_fret_input_group_demo_focus_ring_matches`, `web_vs_fret_input_group_demo_aria_invalid_focus_ring_matches`).
  - Block-end addons (`align=block-end`) are now supported for textarea-driven input groups, and
    `ml-auto` on flex children now produces the expected "push to end" outcome in declarative flex.
  - Kbd-in-addon negative margin outcomes (v4 `has-[>kbd]`) now match upstream via explicit hints.
  - Block addon dividers (`border-b`/`border-t`) now match upstream padding outcomes for block-start
    and block-end addons.
  - `InputGroupText` is now available as a first-class shadcn surface for spacing-focused tests.
  - `InputGroupButton` is now available (xs/sm/icon sizes) to support input-group compositions.
  - `InputGroupButton` no longer forces a fill-width content row for text buttons, matching the
    upstream shrink-to-fit behavior.
- Conformance gates:
  - Layout: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs`
    (`web_vs_fret_layout_input_group_dropdown_height`, `web_vs_fret_layout_input_group_icon_geometry_matches`,
    `web_vs_fret_layout_input_group_spinner_geometry_matches`, `web_vs_fret_layout_input_group_button_geometry_matches`,
    `web_vs_fret_layout_input_group_tooltip_geometry_matches`, `web_vs_fret_layout_spinner_input_group_geometry_matches`,
    `web_vs_fret_layout_empty_input_group_geometry_matches`, `web_vs_fret_layout_kbd_input_group_geometry_matches`,
    `web_vs_fret_layout_input_group_textarea_geometry_matches`,
    `web_vs_fret_layout_input_group_custom_geometry_matches`,
    `web_vs_fret_layout_input_group_demo_block_end_geometry_matches`,
    `web_vs_fret_layout_input_group_text_currency_geometry_matches`,
    `web_vs_fret_layout_input_group_text_url_geometry_matches`,
    `web_vs_fret_layout_input_group_text_email_geometry_matches`,
    `web_vs_fret_layout_input_group_text_textarea_count_geometry_matches`).
- Gaps to check next:
  - Placeholder + selection colors for group control content (ensure they match upstream shadcn tokens).

### `Textarea`

- Upstream: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/textarea.tsx`
- Fret: `ecosystem/fret-ui-shadcn/src/textarea.rs`
- Notes:
  - Upstream uses the same control chrome taxonomy as `Input` (`border-input`, `shadow-xs`, `focus-visible:ring-[3px]`),
    including `aria-invalid:border-destructive` and `aria-invalid:ring-destructive/*` overrides.
- Recent fixes:
  - `aria-invalid=true` border color now matches shadcn-web (`textarea-demo.invalid`) and is gated via
    `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs`
    (`web_vs_fret_textarea_demo_aria_invalid_border_color_matches`).
  - Focus ring (`ring-[3px]`) and ring color overrides now match shadcn-web focus variants
    (`textarea-demo.focus`, `textarea-demo.invalid-focus`) and are gated via
    `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs`
    (`web_vs_fret_textarea_demo_focus_ring_matches`, `web_vs_fret_textarea_demo_aria_invalid_focus_ring_matches`).
  - Note: shadcn's `aria-invalid:ring-*` is a ring color override; the ring only becomes visible when
    `focus-visible:ring-[3px]` is also active.

### `Breadcrumb`

- Upstream: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/breadcrumb.tsx`
- Fret: `ecosystem/fret-ui-shadcn/src/breadcrumb.rs`
- Gaps to check:
  - Separator icon sizing (`ChevronRight` at `size-3.5`).
  - Ellipsis footprint (`size-9`) and icon centering (`MoreHorizontal` at `size-4`).
  - Dropdown composition: ensure `breadcrumb-demo` / `breadcrumb-dropdown` match trigger sizing and
    menu placement (portal panel size + clamping behavior).
  - Responsive composition: `breadcrumb-responsive` uses `DropdownMenu` on desktop and `Drawer` on
    mobile, and gates both overlay outcomes (`menu` width + `dialog` insets).
  - Mobile truncation: `max-w-20 truncate md:max-w-none` on the trailing link/page should clamp to
    `80px` at small viewports without wrapping.
  - Responsive gap (`gap-1.5` vs `sm:gap-2.5`): Fret currently aligns to the desktop (`sm`) outcome.
- Conformance gates:
  - Layout: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs`
    (`web_vs_fret_layout_breadcrumb_separator_geometry`, `web_vs_fret_layout_breadcrumb_ellipsis_geometry`,
    `web_vs_fret_layout_breadcrumb_dropdown_trigger_geometry`, `web_vs_fret_layout_breadcrumb_demo_toggle_trigger_geometry`,
    `web_vs_fret_layout_breadcrumb_responsive_mobile_truncation_geometry`).
  - Overlay placement: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs`
    (`web_vs_fret_breadcrumb_demo_overlay_placement_matches`, `web_vs_fret_breadcrumb_demo_small_viewport_overlay_placement_matches`,
    `web_vs_fret_breadcrumb_dropdown_overlay_placement_matches`, `web_vs_fret_breadcrumb_dropdown_small_viewport_overlay_placement_matches`,
    `web_vs_fret_breadcrumb_responsive_overlay_placement_matches`, `web_vs_fret_breadcrumb_responsive_mobile_drawer_overlay_insets_match`).

### `Button`

- Upstream: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/button.tsx`
- Fret: `ecosystem/fret-ui-shadcn/src/button.rs`
- Gaps to check:
  - Size: `h-9` baseline, icon-only sizing (`size-9`) behavior.
  - Variant mapping: outline uses border + shadow-xs; destructive uses dedicated ring color.
  - Focus ring thickness (`3px`) and ring/border keys.

Recent fixes:

- Focus ring (`ring-[3px]`) now matches shadcn-web focus variant (`button-demo.focus`) and is gated via
  `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs`
  (`web_vs_fret_button_demo_focus_ring_matches`).

Conformance gates:

- Chrome: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs` (`web_vs_fret_button_demo_control_chrome_matches`).
- Focus ring: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs` (`web_vs_fret_button_demo_focus_ring_matches`).

### `ButtonGroup`

- Upstream: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/button-group.tsx`
- Fret: `ecosystem/fret-ui-shadcn/src/button_group.rs`
- Gaps to check:
  - Nested group spacing: `has-[>[data-slot=button-group]]:gap-2` (8px) should be reflected in Fret flex gap.
  - Border merge: `border-l-0` on non-first buttons (avoids double borders).
  - Radius merge: `rounded-l-none` / `rounded-r-none` on middle buttons (keeps only outer corners rounded).
  - Input/select compositions: group-style merges that include non-button controls require per-edge border and per-corner radius overrides without introducing generic slot/asChild (see ADR 0117).
  - Scope: do not introduce generic slot/asChild support (see ADR 0117).
- Conformance gates:
  - Chrome + layout gap: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs`
    (`web_vs_fret_button_group_demo_button_chrome_matches`).
  - Split button separator: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs`
    (`web_vs_fret_button_group_split_chrome_matches`).
  - Vertical orientation: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs`
    (`web_vs_fret_button_group_orientation_vertical_chrome_matches`).
  - Nested groups: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs`
    (`web_vs_fret_button_group_nested_geometry_and_chrome_match`).
  - Group separators: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs`
    (`web_vs_fret_button_group_separator_geometry_and_chrome_match`).
  - Size variants: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs`
    (`web_vs_fret_button_group_size_geometry_and_chrome_match`).
  - Dropdown split: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs`
    (`web_vs_fret_button_group_dropdown_geometry_and_chrome_match`).
  - Popover split: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs`
    (`web_vs_fret_button_group_popover_geometry_and_chrome_match`).
  - Input + button: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs`
    (`web_vs_fret_button_group_input_geometry_and_chrome_match`).
  - Select + input: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs`
    (`web_vs_fret_button_group_select_geometry_and_chrome_match`).
  - Input group: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs`
    (`web_vs_fret_button_group_input_group_geometry_matches`).

### `Toggle` / `ToggleGroup`

- Upstream: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/toggle.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/ui/toggle-group.tsx`
- Fret: `ecosystem/fret-ui-shadcn/src/toggle.rs`, `ecosystem/fret-ui-shadcn/src/toggle_group.rs`
- Gaps to check:
  - Size variants: `h-8` (sm), `h-9` (default), `h-10` (lg) and padding/icon centering.
  - Variant mapping: outline border behavior and focus ring thickness.
  - ToggleGroup semantics:
    - `Single`: items expose radio semantics (`role=radio`, `aria-checked`) and should be discoverable by `aria-label`.
    - `Multiple`: items expose pressed semantics (`aria-pressed`) and should be discoverable by `aria-label`.
- Conformance gates:
  - Layout: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs`
    (`web_vs_fret_layout_toggle_sm_geometry_matches`, `web_vs_fret_layout_toggle_lg_geometry_matches`,
    `web_vs_fret_layout_toggle_outline_geometry_matches`, `web_vs_fret_layout_toggle_disabled_geometry_matches`,
    `web_vs_fret_layout_toggle_with_text_height_matches`,
    `web_vs_fret_layout_toggle_group_sm_heights_match`, `web_vs_fret_layout_toggle_group_lg_heights_match`,
    `web_vs_fret_layout_toggle_group_outline_heights_match`, `web_vs_fret_layout_toggle_group_disabled_heights_match`,
    `web_vs_fret_layout_toggle_group_single_heights_match`, `web_vs_fret_layout_toggle_group_spacing_heights_match`).

### `Tabs`

- Upstream: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/tabs.tsx`
- Fret: `ecosystem/fret-ui-shadcn/src/tabs.rs`
- Gaps to check:
  - TabsList: `h-9`, `rounded-lg`, `p-[3px]`, `bg-muted`.
  - Trigger: `flex-1`, `h-[calc(100%-1px)]`, active background/border behavior.

### `DataTable`

- Upstream example block: `repo-ref/ui/apps/v4/registry/new-york-v4/blocks/dashboard-01/components/data-table.tsx`
- Upstream table primitives: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/table.tsx`
- Fret: `ecosystem/fret-ui-shadcn/src/data_table.rs`
- Gaps to check:
  - Wrapper: `overflow-hidden rounded-lg border` (container chrome).
  - Header row: `bg-muted` and fixed/sticky behavior (Fret uses a separate header + wheel sync).
  - Empty state: `h-24 text-center` equivalent for zero rows.
  - First/utility columns: fixed-width affordances (drag handles, checkboxes) without layout jitter.

### `Tooltip`

- Upstream: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/tooltip.tsx`
- Fret: `ecosystem/fret-ui-shadcn/src/tooltip.rs`
- Gaps to check:
  - Content chrome: `bg-foreground text-background`, `rounded-md`, `px-3 py-1.5`, `text-xs`.
  - Arrow: diamond rotated 45deg, size `2.5`, minor translate.

### `Dialog`

- Upstream: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/dialog.tsx`
- Fret: `ecosystem/fret-ui-shadcn/src/dialog.rs`
- Gaps to check:
  - Overlay: `bg-black/50` (not fully opaque).
  - Content: centered, `rounded-lg`, `border`, `p-6`, `shadow-lg`, close button slot.

Conformance gates:

- Chrome: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome.rs`
  (`web_vs_fret_dialog_demo_panel_chrome_matches`).
- Placement: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs`
  (`web_vs_fret_dialog_demo_overlay_center_matches`).

### `AlertDialog`

- Upstream: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/alert-dialog.tsx`
- Fret: `ecosystem/fret-ui-shadcn/src/alert_dialog.rs`
- Gaps to check:
  - Safety defaults: overlay click-to-dismiss disabled by default.
  - Cancel focus preference (first `AlertDialogCancel`).

Conformance gates:

- Chrome: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome.rs`
  (`web_vs_fret_alert_dialog_demo_panel_chrome_matches`).
- Placement: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs`
  (`web_vs_fret_alert_dialog_demo_overlay_center_matches`).

### `Sheet`

- Upstream: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/sheet.tsx`
- Fret: `ecosystem/fret-ui-shadcn/src/sheet.rs`
- Gaps to check:
  - Side-specific border edge (right=`border-l`, left=`border-r`, top=`border-b`, bottom=`border-t`).
  - Side-specific sizing (`sm:max-w-sm` for left/right) and motion.

Conformance gates:

- Chrome: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome.rs`
  (`web_vs_fret_sheet_demo_panel_chrome_matches`, `web_vs_fret_sheet_side_panel_chrome_matches`, `web_vs_fret_sheet_side_right_panel_chrome_matches`, `web_vs_fret_sheet_side_bottom_panel_chrome_matches`, `web_vs_fret_sheet_side_left_panel_chrome_matches`).
- Placement: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs`
  (`web_vs_fret_sheet_demo_overlay_insets_match`, `web_vs_fret_sheet_side_top_overlay_insets_match`, `web_vs_fret_sheet_side_right_overlay_insets_match`, `web_vs_fret_sheet_side_bottom_overlay_insets_match`, `web_vs_fret_sheet_side_left_overlay_insets_match`).
