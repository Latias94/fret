---
title: Shadcn Semantic Drift Sweep (v1) — TODO
status: draft
date: 2026-02-24
---

# Shadcn Semantic Drift Sweep (v1) — TODO

Workstream entry:

- `docs/workstreams/shadcn-semantic-drift-sweep-v1.md`

## Audit / inventory

- [x] Produce a “responsive decision table” for all viewport/container queries in
  `ecosystem/fret-ui-shadcn/src/`:
  - [x] Viewport-driven (device shell) — keep viewport:
    - e.g. “Drawer on mobile” patterns (ADR 0232).
  - [x] Container-driven (panel width) — use container query regions:
    - e.g. layouts inside docking/panels (ADR 0231).
  - [x] Mixed/unclear — write down the decision and leave an evidence anchor to upstream.

- [x] Collect upstream evidence anchors in `repo-ref/ui` for each responsive decision that differs
  from web parity (and `repo-ref/kibo` for shadcn extras).

## Responsive decision table (seed)

Note: `repo-ref/` is local state (not committed). See `docs/repo-ref.md`.

| Surface | Fret | Upstream evidence | Query semantics | Decision | Gate |
| --- | --- | --- | --- | --- | --- |
| Field: `orientation="responsive"` | `ecosystem/fret-ui-shadcn/src/field.rs` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/field.tsx` (`@container/field-group`, `@md/field-group:*`), `repo-ref/ui/apps/v4/content/docs/components/field.mdx` | Container (panel width) | Keep container queries (ADR 0231); use viewport fallback when container width is temporarily unknown | `ecosystem/fret-ui-shadcn/tests/field_responsive_orientation.rs` (test: `field_orientation_responsive_follows_container_md_breakpoint`) |
| AlertDialog footer stacking (`sm:`) | `ecosystem/fret-ui-shadcn/src/alert_dialog.rs` | `repo-ref/ui/apps/v4/registry/base-mira/ui/alert-dialog.tsx` (`sm:flex-row sm:justify-end`) | Viewport (device shell) | Keep viewport queries (ADR 0232) | `ecosystem/fret-ui-shadcn/src/alert_dialog.rs` (test: `alert_dialog_footer_stacks_on_base_viewport_and_rows_on_sm`) |
| Empty padding (`md:p-12`) | `ecosystem/fret-ui-shadcn/src/empty.rs` | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/empty.tsx` (`p-6 ... md:p-12`) | Container (panel width) | Keep container queries (ADR 0231) | `ecosystem/fret-ui-shadcn/tests/empty_responsive_padding.rs` (test: `empty_padding_switches_at_md_using_container_queries`) |
| Drawer layout (side width, max height) | `ecosystem/fret-ui-shadcn/src/drawer.rs` | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/drawer.tsx` (`w-3/4`, `sm:max-w-sm`, `max-h-[80vh]`) | Viewport (device shell) | Keep viewport bounds + viewport breakpoints (ADR 0232) | `ecosystem/fret-ui-shadcn/tests/drawer_layout_invariants.rs` (tests: `drawer_side_panel_width_tracks_viewport_fraction_and_caps`, `drawer_bottom_height_caps_at_80vh_and_edge_gap`) |
| Dialog content sizing + footer row (`sm:`) | `ecosystem/fret-ui-shadcn/src/dialog.rs` | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/dialog.tsx` (`sm:max-w-lg`, `sm:flex-row`, `sm:justify-end`) | Viewport (device shell) | Keep viewport semantics (ADR 0232) | `ecosystem/fret-ui-shadcn/src/dialog.rs` (test: `dialog_footer_stacks_on_base_viewport_and_rows_on_sm`) |
| Calendar multi-month layout (`md:flex-row`) | `ecosystem/fret-ui-shadcn/src/calendar.rs` | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/calendar.tsx` (`md:flex-row` on `.months`) | Mixed (container; viewport in popovers) | Keep mixed: container query regions (ADR 0231) for panel width; prefer viewport `md` when in `PopoverContent` to avoid circular sizing | `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/calendar.rs` (multi-month layout coverage) |
| CalendarMultiple multi-month layout (`md:flex-row`) | `ecosystem/fret-ui-shadcn/src/calendar_multiple.rs` | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/calendar.tsx` (`md:flex-row` on `.months`) | Mixed (container; viewport in popovers) | Same as `Calendar` (shared month-layout policy) | `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/calendar.rs` (multi-month layout coverage) |
| CalendarRange multi-month layout (`md:flex-row`) | `ecosystem/fret-ui-shadcn/src/calendar_range.rs` | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/calendar.tsx` (`md:flex-row` on `.months`) | Mixed (container; viewport in popovers) | Same as `Calendar` (shared month-layout policy) | `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/calendar.rs` (multi-month layout coverage) |
| Popover (placement + motion; no breakpoints) | `ecosystem/fret-ui-shadcn/src/popover.rs` | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/popover.tsx` | Viewport (device shell) | Treat as viewport-level overlay geometry (not container responsive) | `ecosystem/fret-ui-shadcn/src/popover.rs` (extensive interaction tests) |
| DropdownMenu content max-height (available height) | `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs` | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/dropdown-menu.tsx` (`max-h-(--radix-dropdown-menu-content-available-height)`) | Viewport (device shell) | Keep viewport-derived available-height constraints (Radix popper vars) | `ecosystem/fret-ui-shadcn/tests/dropdown_menu_available_height.rs` (test: `dropdown_menu_content_height_clamps_to_available_height`) |
| ContextMenu content max-height (available height) | `ecosystem/fret-ui-shadcn/src/context_menu.rs` | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/context-menu.tsx` (`max-h-(--radix-context-menu-content-available-height)`) | Viewport (device shell) | Keep viewport-derived available-height constraints (Radix popper vars) | `ecosystem/fret-ui-shadcn/tests/context_menu_available_height.rs` (test: `context_menu_content_height_clamps_to_available_height`) |
| Combobox “responsive” (Drawer on mobile) | `ecosystem/fret-ui-shadcn/src/combobox.rs` | `repo-ref/ui/apps/v4/registry/new-york-v4/examples/combobox-responsive.tsx` (`useMediaQuery("(min-width: 768px)")`) | Viewport (device shell) | Keep viewport breakpoint (ADR 0232) | `ecosystem/fret-ui-shadcn/tests/combobox_responsive_breakpoint.rs` (test: `combobox_responsive_switches_between_drawer_and_popover_at_md_breakpoint`) |
| DataTable faceted filter “show label badges” (`lg:*`) | `ecosystem/fret-ui-shadcn/src/data_table_recipes.rs` | `repo-ref/ui/apps/v4/app/(app)/examples/tasks/components/data-table-faceted-filter.tsx` (`lg:hidden` / `hidden lg:flex`) | Dual-mode (viewport default; container optional) | Keep explicit knob (`DataTableToolbarResponsiveQuery`), default viewport parity, allow container region for docking/panels | `ecosystem/fret-ui-shadcn/tests/data_table_toolbar_faceted_responsive.rs` + `tools/diag-scripts/ui-gallery-data-table-toolbar-faceted-responsive.json` |
| Sidebar mobile vs desktop (`md:block`) | `ecosystem/fret-ui-shadcn/src/sidebar.rs` | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/sidebar.tsx` (`useIsMobile`, `hidden md:block`) | Viewport (device shell) | Keep viewport breakpoint (ADR 0232) | `tools/diag-scripts/ui-gallery-sidebar-mobile-controlled-open-sync.json` |
| Shadcn Extras: Kanban (`sm:` spacing/width) | `ecosystem/fret-ui-shadcn/src/extras/kanban.rs` | `repo-ref/kibo/packages/kanban` | Viewport (device shell) | Keep viewport breakpoint (extras demo parity) | `tools/diag-scripts/ui-gallery-shadcn-extras-kanban-dnd.json` |
| Shadcn Extras: Marquee base cycle width | `ecosystem/fret-ui-shadcn/src/extras/marquee.rs` | `repo-ref/kibo/packages/marquee` | Container (panel width; viewport fallback when unknown) | Default to container query region width; keep explicit `cycle_width_px` override (ADR 0231) | `ecosystem/fret-ui-shadcn/src/extras/marquee.rs` (test: `marquee_default_cycle_width_prefers_region_over_viewport`) |
| NavigationMenu `md:*` behavior | `ecosystem/fret-ui-shadcn/src/navigation_menu.rs` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/navigation-menu.tsx` (`md:absolute`, `md:w-[var(--radix-…)]`) | Mixed (upstream viewport; editor layouts may prefer container) | Add a query-source knob: default viewport parity, optional container (editor-first); use viewport fallback when container width is temporarily unknown | `ecosystem/fret-ui-shadcn/src/navigation_menu.rs` (test: `navigation_menu_md_breakpoint_query_can_follow_viewport_or_container_width`) + `apps/fret-ui-gallery/src/ui/pages/navigation_menu.rs` (toggle) + `tools/diag-scripts/ui-gallery-navigation-menu-md-breakpoint-query-source-toggle.json` |

## Responsive drift: DataTable “LG show labels”

- [x] Confirm upstream behavior and intent:
  - [x] `repo-ref/ui/apps/v4/app/(app)/examples/tasks/components/data-table-faceted-filter.tsx`
    uses `lg:hidden` / `hidden lg:flex`.
  - [x] `repo-ref/ui/apps/v4/app/(app)/examples/tasks/components/data-table-toolbar.tsx` uses
    `lg:w-[250px]`.
- [x] Decide Fret behavior for editor-grade layouts:
  - [ ] Option A (parity-first): keep viewport `LG` (matches web Tailwind semantics).
  - [ ] Option B (editor-first): switch to container query so the toolbar adapts to panel width.
  - [x] Option C (dual-mode): expose an explicit “query source” knob in the recipe layer
    (viewport vs container region id), defaulting to parity-first.
- [x] Add a regression gate for the chosen behavior:
  - [x] unit test (layout invariant), and/or
    - Evidence: `ecosystem/fret-ui-shadcn/tests/data_table_toolbar_faceted_responsive.rs`
  - [x] `tools/diag-scripts/ui-gallery-data-table-toolbar-faceted-responsive.json` (window resize;
    asserts `data-table-toolbar-faceted-status-badge-*` invariants via `test_id`).
  - [x] Expose a gallery toggle for the query source (makes drift review interactive):
    - `apps/fret-ui-gallery/src/ui/previews/gallery/data/table_legacy.rs`

## Responsive drift: Marquee base width (viewport vs container)

- [x] Confirm intent:
  - [x] If used inside docking/panels, the cycle width should follow the local container width,
    not the window viewport width.
- [x] Decide default semantics:
  - [x] Switch default to container query region width with viewport fallback when unknown (ADR
    0231).
- [x] Add a regression gate for the chosen behavior:
  - [x] `ecosystem/fret-ui-shadcn/src/extras/marquee.rs` (unit tests)

## Theme metadata drift: remove theme-name heuristics

- [x] Inventory all callsites using theme-name heuristics:
  - [x] `ecosystem/fret-ui-shadcn/src/*` (search: `theme.name.`).
- [x] Choose a stable strategy:
  - [x] Add a theme metadata field to `ThemeConfig` + `Theme` (app/theme-owned).
  - [ ] Prefer explicit token keys for “dark variant” values and remove heuristics.
  - [ ] Where necessary, treat per-window environment `ColorScheme` (ADR 0232) as a hint, not the
    source of truth (theme content remains app-owned per ADR 0032).
- [x] Migrate the callsites and add at least one regression test covering:
  - invalid ring alpha selection,
  - inactive tabs foreground selection, or
  - any other behavior currently keyed off the name heuristic.

### Inventory: remaining `theme.color_scheme` branches (post-name-heuristics)

These are cases where recipe code branches on `theme.color_scheme` for variant behavior.
Long-term, we prefer to express these via explicit shadcn theme tokens so recipes become pure
"token reads" and custom themes can override the behavior without code changes.

Invalid ring variants (destructive `/20` vs `/40`):

- `ecosystem/fret-ui-shadcn/src/checkbox.rs` (aria-invalid focus ring)
- `ecosystem/fret-ui-shadcn/src/combobox.rs` (aria-invalid focus ring)
- `ecosystem/fret-ui-shadcn/src/input.rs` (aria-invalid focus ring)
- `ecosystem/fret-ui-shadcn/src/input_group.rs` (aria-invalid focus ring)
- `ecosystem/fret-ui-shadcn/src/input_otp.rs` (invalid ring color)
- `ecosystem/fret-ui-shadcn/src/native_select.rs` (aria-invalid focus ring)
- `ecosystem/fret-ui-shadcn/src/radio_group.rs` (aria-invalid focus ring)
- `ecosystem/fret-ui-shadcn/src/select.rs` (aria-invalid focus ring)
- `ecosystem/fret-ui-shadcn/src/textarea.rs` (aria-invalid focus ring)

Tabs trigger inactive foreground:

- `ecosystem/fret-ui-shadcn/src/tabs.rs` (light: `foreground`, dark: `muted-foreground`)

RadioGroup choice-card checked background alpha:

- `ecosystem/fret-ui-shadcn/src/radio_group.rs` (light: `primary/5`, dark: `primary/10` equivalent)

### Proposal: replace `theme.color_scheme` branches with explicit component token keys

Approach:

- [x] Add **component-owned color keys** to shadcn theme presets (`shadcn_themes.rs`) that encode
  the scheme-specific choice.
- [x] Update recipes to prefer the component key and fall back to the existing branch when the
  key is missing (custom themes).

Candidate keys (landed):

- `component.control.invalid_ring` (light: `destructive/20`, dark: `destructive/40`)
- `component.tabs.trigger.fg_inactive` (light: `foreground`, dark: `muted-foreground`)
- `component.radio_group.choice_card.checked_bg` (light: `primary` @ 0.05, dark: `primary` @ 0.10)

Evidence:

- Theme preset seeding + tests: `ecosystem/fret-ui-shadcn/src/shadcn_themes.rs`
- Fallback centralization (custom-theme compatibility): `ecosystem/fret-ui-shadcn/src/theme_variants.rs`
- Callsite migrations:
  - Invalid ring: `ecosystem/fret-ui-shadcn/src/{checkbox,combobox,input,input_group,input_otp,native_select,radio_group,select,textarea}.rs`
  - Tabs inactive fg: `ecosystem/fret-ui-shadcn/src/tabs.rs`
  - Radio choice-card checked bg: `ecosystem/fret-ui-shadcn/src/radio_group.rs`

## Token read sweep: replace unnecessary `Theme` clones with snapshots

- [x] Sweep `Theme::global(&*cx.app).clone()` callsites in `ecosystem/fret-ui-shadcn/src/`:
  - [x] Convert an initial batch to `Theme::global(&*cx.app).snapshot()` when only token reads are needed:
    - Evidence: `ecosystem/fret-ui-shadcn/src/{accordion,alert_dialog,avatar,badge,button,button_group,calendar,calendar_hijri,calendar_multiple,calendar_range,carousel,chart,checkbox,collapsible,combobox,combobox_chips,command,context_menu,data_grid,data_grid_canvas,data_table,data_table_recipes,date_picker_with_presets,date_range_picker,dialog,drawer,dropdown_menu,empty,field,form_field,hover_card,input_otp,kbd,media_image,menubar,navigation_menu,native_select,pagination,popover,progress,radio_group,resizable,scroll_area,select,sheet,shortcut_hint,skeleton,slider,spinner,tabs,textarea,toggle_group,tooltip}.rs`, `ecosystem/fret-ui-shadcn/src/extras/{announcement,avatar_stack,banner,kanban,marquee,rating,relative_time,tags,ticker}.rs`
    - Progress note: reduced remaining `Theme::global(...).clone()` callsites from 135 → 0.
  - [x] Continue converting remaining callsites (prioritize hot paths: `input`, `select`, `sheet`, `popover`, `dropdown_menu`).
  - [ ] Keep `Theme` where name/metadata APIs are required (but avoid long-lived borrows across
    `cx.*` calls).
- [ ] Add a small unit/perf-adjacent test or diagnostic note if this sweep reduces allocation
  churn on common views.

- [x] Add a regression guard to prevent reintroducing `Theme::global(...).clone()` callsites:
  - `ecosystem/fret-ui-shadcn/tests/no_theme_global_clone_regression.rs`
  - Note: the guard is intentionally scoped to `ecosystem/fret-ui-shadcn/src/` (production-ish
    paths). Integration tests under `ecosystem/fret-ui-shadcn/tests/` may still use a cloned
    `Theme` when it keeps borrow scopes simple.

## Reduced motion drift: continuous animations should not request frames

- [x] Skeleton pulse respects reduced motion (no RAF requests):
  - `ecosystem/fret-ui-shadcn/src/skeleton.rs`
  - `ecosystem/fret-ui-shadcn/tests/reduced_motion_continuous_frames.rs`
- [x] Spinner rotation respects reduced motion (no RAF requests):
  - `ecosystem/fret-ui-shadcn/src/spinner.rs`
  - `ecosystem/fret-ui-shadcn/tests/reduced_motion_continuous_frames.rs`

## Docs / closure

- [ ] Update the drift inventory in `docs/workstreams/shadcn-semantic-drift-sweep-v1.md` as new
  issues are found.
- [ ] For any “hard-to-change” contract additions (theme metadata, new token namespaces), add/update
  ADRs and evidence anchors.
