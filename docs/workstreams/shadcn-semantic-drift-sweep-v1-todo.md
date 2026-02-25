---
title: Shadcn Semantic Drift Sweep (v1) — TODO
status: draft
date: 2026-02-24
---

# Shadcn Semantic Drift Sweep (v1) — TODO

Workstream entry:

- `docs/workstreams/shadcn-semantic-drift-sweep-v1.md`

## Audit / inventory

- [ ] Produce a “responsive decision table” for all viewport/container queries in
  `ecosystem/fret-ui-shadcn/src/`:
  - [ ] Viewport-driven (device shell) — keep viewport:
    - e.g. “Drawer on mobile” patterns (ADR 0232).
  - [ ] Container-driven (panel width) — use container query regions:
    - e.g. layouts inside docking/panels (ADR 0231).
  - [ ] Mixed/unclear — write down the decision and leave an evidence anchor to upstream.

- [ ] Collect upstream evidence anchors in `repo-ref/ui` for each responsive decision that differs
  from web parity.

## Responsive decision table (seed)

Note: `repo-ref/` is local state (not committed). See `docs/repo-ref.md`.

| Surface | Fret | Upstream evidence | Query semantics | Decision | Gate |
| --- | --- | --- | --- | --- | --- |
| Field: `orientation="responsive"` | `ecosystem/fret-ui-shadcn/src/field.rs` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/field.tsx` (`@container/field-group`, `@md/field-group:*`), `repo-ref/ui/apps/v4/content/docs/components/field.mdx` | Container (panel width) | Keep container queries (ADR 0231) | `ecosystem/fret-ui-shadcn/tests/field_responsive_orientation.rs` (test: `field_orientation_responsive_follows_container_md_breakpoint`) |
| AlertDialog footer stacking (`sm:`) | `ecosystem/fret-ui-shadcn/src/alert_dialog.rs` | `repo-ref/ui/apps/v4/registry/base-mira/ui/alert-dialog.tsx` (`sm:flex-row sm:justify-end`) | Viewport (device shell) | Keep viewport queries (ADR 0232) | `ecosystem/fret-ui-shadcn/src/alert_dialog.rs` (test: `alert_dialog_footer_stacks_on_base_viewport_and_rows_on_sm`) |
| Empty padding (`md:p-12`) | `ecosystem/fret-ui-shadcn/src/empty.rs` | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/empty.tsx` (`p-6 ... md:p-12`) | Container (panel width) | Keep container queries (ADR 0231) | `ecosystem/fret-ui-shadcn/tests/empty_responsive_padding.rs` (test: `empty_padding_switches_at_md_using_container_queries`) |
| Drawer layout (side width, max height) | `ecosystem/fret-ui-shadcn/src/drawer.rs` | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/drawer.tsx` (`w-3/4`, `sm:max-w-sm`, `max-h-[80vh]`) | Viewport (device shell) | Keep viewport bounds + viewport breakpoints (ADR 0232) | `ecosystem/fret-ui-shadcn/tests/drawer_layout_invariants.rs` (tests: `drawer_side_panel_width_tracks_viewport_fraction_and_caps`, `drawer_bottom_height_caps_at_80vh_and_edge_gap`) |
| Dialog content sizing + footer row (`sm:`) | `ecosystem/fret-ui-shadcn/src/dialog.rs` | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/dialog.tsx` (`sm:max-w-lg`, `sm:flex-row`, `sm:justify-end`) | Viewport (device shell) | Keep viewport semantics (ADR 0232) | `ecosystem/fret-ui-shadcn/src/dialog.rs` (test: `dialog_footer_stacks_on_base_viewport_and_rows_on_sm`) |
| Calendar multi-month layout (`md:flex-row`) | `ecosystem/fret-ui-shadcn/src/calendar.rs` | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/calendar.tsx` (`md:flex-row` on `.months`) | Mixed (container; viewport in popovers) | Keep mixed: container query regions (ADR 0231) for panel width; prefer viewport `md` when in `PopoverContent` to avoid circular sizing | `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/calendar.rs` (multi-month layout coverage) |
| Popover (placement + motion; no breakpoints) | `ecosystem/fret-ui-shadcn/src/popover.rs` | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/popover.tsx` | Viewport (device shell) | Treat as viewport-level overlay geometry (not container responsive) | `ecosystem/fret-ui-shadcn/src/popover.rs` (extensive interaction tests) |
| DropdownMenu content max-height (available height) | `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs` | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/dropdown-menu.tsx` (`max-h-(--radix-dropdown-menu-content-available-height)`) | Viewport (device shell) | Keep viewport-derived available-height constraints (Radix popper vars) | `ecosystem/fret-ui-shadcn/tests/dropdown_menu_available_height.rs` (test: `dropdown_menu_content_height_clamps_to_available_height`) |
| ContextMenu content max-height (available height) | `ecosystem/fret-ui-shadcn/src/context_menu.rs` | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/context-menu.tsx` (`max-h-(--radix-context-menu-content-available-height)`) | Viewport (device shell) | Keep viewport-derived available-height constraints (Radix popper vars) | `ecosystem/fret-ui-shadcn/tests/context_menu_available_height.rs` (test: `context_menu_content_height_clamps_to_available_height`) |
| Combobox “responsive” (Drawer on mobile) | `ecosystem/fret-ui-shadcn/src/combobox.rs` | `repo-ref/ui/apps/v4/registry/new-york-v4/examples/combobox-responsive.tsx` (`useMediaQuery("(min-width: 768px)")`) | Viewport (device shell) | Keep viewport breakpoint (ADR 0232) | `ecosystem/fret-ui-shadcn/tests/combobox_responsive_breakpoint.rs` (test: `combobox_responsive_switches_between_drawer_and_popover_at_md_breakpoint`) |
| Sidebar mobile vs desktop (`md:block`) | `ecosystem/fret-ui-shadcn/src/sidebar.rs` | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/sidebar.tsx` (`useIsMobile`, `hidden md:block`) | Viewport (device shell) | Keep viewport breakpoint (ADR 0232) | `tools/diag-scripts/ui-gallery-sidebar-mobile-controlled-open-sync.json` |
| Shadcn Extras: Kanban (`sm:` spacing/width) | `ecosystem/fret-ui-shadcn/src/extras/kanban.rs` | `repo-ref/kibo/packages/kanban` | Viewport (device shell) | Keep viewport breakpoint (extras demo parity) | `tools/diag-scripts/ui-gallery-shadcn-extras-kanban-dnd.json` |
| NavigationMenu `md:*` behavior | `ecosystem/fret-ui-shadcn/src/navigation_menu.rs` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/navigation-menu.tsx` (`md:absolute`, `md:w-[var(--radix-…)]`) | Mixed (upstream viewport; editor layouts may prefer container) | Add a query-source knob: default viewport parity, optional container (editor-first) | `ecosystem/fret-ui-shadcn/src/navigation_menu.rs` (test: `navigation_menu_md_breakpoint_query_can_follow_viewport_or_container_width`) |

## Responsive drift: DataTable “LG show labels”

- [ ] Confirm upstream behavior and intent:
  - [x] `repo-ref/ui/apps/v4/app/(app)/examples/tasks/components/data-table-faceted-filter.tsx`
    uses `lg:hidden` / `hidden lg:flex`.
  - [x] `repo-ref/ui/apps/v4/app/(app)/examples/tasks/components/data-table-toolbar.tsx` uses
    `lg:w-[250px]`.
- [ ] Decide Fret behavior for editor-grade layouts:
  - [ ] Option A (parity-first): keep viewport `LG` (matches web Tailwind semantics).
  - [ ] Option B (editor-first): switch to container query so the toolbar adapts to panel width.
  - [x] Option C (dual-mode): expose an explicit “query source” knob in the recipe layer
    (viewport vs container region id), defaulting to parity-first.
- [x] Add a regression gate for the chosen behavior:
  - [x] unit test (layout invariant), and/or
    - Evidence: `ecosystem/fret-ui-shadcn/tests/data_table_toolbar_faceted_responsive.rs`
  - [x] `tools/diag-scripts/ui-gallery-data-table-toolbar-faceted-responsive.json` (window resize;
    asserts `data-table-toolbar-faceted-status-badge-*` invariants via `test_id`).

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

## Token read sweep: replace unnecessary `Theme` clones with snapshots

- [ ] Sweep `Theme::global(&*cx.app).clone()` callsites in `ecosystem/fret-ui-shadcn/src/`:
  - [x] Convert an initial batch to `Theme::global(&*cx.app).snapshot()` when only token reads are needed:
    - Evidence: `ecosystem/fret-ui-shadcn/src/{alert_dialog,avatar,badge,button,button_group,combobox,command,data_table,dialog,dropdown_menu,native_select,pagination,popover,select,sheet}.rs`
    - Progress note: reduced remaining `Theme::global(...).clone()` callsites from 135 → 89.
  - [ ] Continue converting remaining callsites (prioritize hot paths: `input`, `select`, `sheet`, `popover`, `dropdown_menu`).
  - [ ] Keep `Theme` where name/metadata APIs are required (but avoid long-lived borrows across
    `cx.*` calls).
- [ ] Add a small unit/perf-adjacent test or diagnostic note if this sweep reduces allocation
  churn on common views.

## Docs / closure

- [ ] Update the drift inventory in `docs/workstreams/shadcn-semantic-drift-sweep-v1.md` as new
  issues are found.
- [ ] For any “hard-to-change” contract additions (theme metadata, new token namespaces), add/update
  ADRs and evidence anchors.
