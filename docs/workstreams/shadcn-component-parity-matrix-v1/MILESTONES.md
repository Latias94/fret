---
title: Shadcn Component Parity Matrix v1 Milestones
status: active
date: 2026-05-26
---

# Milestones

## M0 - Lane Opened

Status: complete on 2026-05-25.

The lane owns the component-level harness matrix. It does not reopen
`component-parity-fact-harness-v1`; that lane remains the packet-shape foundation.

## M1 - Initial Matrix Generated

Status: complete on 2026-05-25.

Completed criteria:

- The generator produces a 59-component matrix from repo-local evidence.
- The matrix distinguishes `regression_locked`, `harness_hardening`, `coverage_targeted`,
  `inventory_only`, and `not_in_harness`.
- The first summary shows the current automation ceiling:
  - 18 components have source refs,
  - 14 have upstream DOM/CSS snapshots,
  - 18 have Fret layout evidence,
  - 10 have Fret bundle semantics evidence,
  - 1 has Fret text/paint evidence,
  - 15 have behavior scripts,
  - 5 have responsive/non-desktop coverage.

## M2 - First Matrix-Driven Repair Seed

Status: complete on 2026-05-25.

`drawer.bottom_sheet.mobile` is promoted from `coverage_targeted` to `regression_locked`.

Completed criteria:

- The Drawer recipe fix is locked by `drawer_layout_invariants`.
- The mechanism fixture now names and asserts the true visible-lane rule:
  `responsive-drawer-bottom-sheet-caps-visible-lane`.
- Drawer overlay placement and chrome each have a focused web-golden gate.
- `drawer_agent_packet_p0_v1.json` is included in the matrix generator default packet inputs.
- The regenerated matrix reports 10 `regression_locked` components and 7 `coverage_targeted`
  components.

## M3 - Continue P0 Sweep

Status: complete on 2026-05-25.

`calendar` is promoted from `coverage_targeted` to `regression_locked` after fixing day-grid
vertical spacing in the multiple and range recipes.

Completed criteria:

- `calendar-04` range geometry no longer drifts upward by one `week_row_gap`.
- `calendar-03` multiple geometry no longer drifts upward by one `week_row_gap`.
- Calendar range paint, multiple hover/text-centering, and range focus-visible chrome gates pass.
- `calendar_agent_packet_p0_v1.json` is included in the matrix generator default packet inputs.
- The regenerated matrix reports 11 `regression_locked` components and 6 `coverage_targeted`
  components.

## M4 - Continue P0 Sweep

Status: complete on 2026-05-25.

`select.open.desktop` is promoted from `coverage_targeted` to `regression_locked` after fixing the
open Select item-aligned geometry.

Completed criteria:

- The policy layer now derives item-aligned `items_height` from the largest available listbox,
  visual listbox, and scroll extent source.
- The headless item-aligned solver preserves natural items height when a leading label forces a
  top clamp.
- The shadcn Select recipe keeps Radix-style scroll buttons in normal vertical flow and preserves
  the viewport bottom padding in the scroll extent.
- Select web-golden layout, overlay placement, overlay chrome, keyboard commit, and pointer commit
  gates pass.
- `select_agent_packet_p0_v1.json` is included in the matrix generator default packet inputs.
- The regenerated matrix reports 12 `regression_locked` components and 5 `coverage_targeted`
  components.

## M5 - Continue P0 Sweep

Status: complete on 2026-05-25.

`combobox.open.mobile` and `combobox.open.desktop` are promoted from `coverage_targeted` to
`regression_locked` after fixing the UI Gallery compact-shell carrier width.

Completed criteria:

- The 375x240 Gallery shell no longer leaves the component story behind a fixed 280px sidebar.
- The responsive combobox trigger is horizontally reachable in the mobile effective viewport.
- The mobile diag script passes and captures schema2 bundle, layout sidecar, screenshot, AI packet,
  and share zip evidence.
- The desktop companion diag script still passes bottom/start overlay placement after the carrier
  fix.
- `combobox_agent_packet_p0_v1.json` is included in the matrix generator default packet inputs.
- The regenerated matrix reports 13 `regression_locked` components and 4 `coverage_targeted`
  components.

## M6 - Continue P0 Sweep

Status: complete on 2026-05-25.

`popover.command.desktop` is promoted from `coverage_targeted` to `regression_locked` after
closing the Popover command-shell mechanism and behavior evidence gaps.

Completed criteria:

- The promoted mechanism fixture `popover-command-shell-wraps-hover-region-max-height` passes and
  keeps the PopoverContent shell from regressing to the placement fallback height.
- The upstream DOM evidence includes the source-backed `combobox-popover.open` command-in-popover
  snapshot plus the official `popover-demo.open` snapshot.
- The Popover relation/action diagnostics script passes and captures schema2 bundle, layout
  sidecar, screenshot, AI packet, and share zip evidence.
- `popover_agent_packet_p0_v1.json` is included in the matrix generator default packet inputs.
- The regenerated matrix reports 14 `regression_locked` components and 3 `coverage_targeted`
  components.

## M7 - Continue P0 Sweep

Status: complete on 2026-05-25.

`dropdown-menu.submenu.mobile` is promoted from `coverage_targeted` to `regression_locked` after
fixing Dropdown Menu label text-style drift and closing submenu semantics/placement evidence.

Completed criteria:

- `DropdownMenuLabel` now matches upstream shadcn `text-sm font-medium` section-label styling
  instead of the muted `text-xs` group-label lane.
- The constrained root-menu web-golden fixture no longer has the former 3.67px
  `top_to_first_item` drift.
- The submenu smoke diagnostics script passes with schema2 bundle, nested submenu placement traces,
  AI packet, and share zip evidence.
- `dropdown_menu_agent_packet_p0_v1.json` is included in the matrix generator default packet inputs.
- The regenerated matrix reports 15 `regression_locked` components and 2 `coverage_targeted`
  components.

## M8 - Continue P0 Sweep

Status: complete on 2026-05-25.

`input.docs-demo.desktop` is promoted from `coverage_targeted` to `regression_locked` after adding
bundle semantics, behavior-script, and control-id relation evidence.

Completed criteria:

- The existing v1 report still proves upstream DOM + Fret layout parity for the 320x36 docs-demo
  Input control.
- The new `ui-gallery-input-demo-relation-action-state` diagnostics script passes and proves
  `text_field` role, enabled focus/set_value actions, click focus, value mutation, layout sidecar,
  schema2 bundle, AI packet, and share zip evidence.
- The `input_control_id_uses` unit gate passes and keeps FieldLabel/FieldDescription relation
  semantics on the concrete TextInput node.
- `input_agent_packet_p0_v1.json` is included in the matrix generator default packet inputs.
- The regenerated matrix reports 16 `regression_locked` components and 1 `coverage_targeted`
  component.

## M9 - Continue P0 Sweep

Status: complete on 2026-05-25.

`data-table.policy.desktop` is promoted from `coverage_targeted` to `regression_locked` after
connecting the upstream DOM snapshot and rerunning the Data Table policy diagnostics suite.

Completed criteria:

- The upstream `data-table-demo` DOM/CSS golden is attached as the docs-demo source reference for
  filter input, view-options trigger, table shell, row selection, row actions, and pagination.
- The `ui-gallery-data-table` suite passed 8/8 scripts in a fresh reuse-launch run, covering default
  recipe smoke, pagination metadata, guide checkbox-only selection, row-actions menu stability,
  header screenshot/layout capture, list-like pointer selection, and page smoke.
- The suite summary records no reason codes, no blocking reasons, no focus mismatches, and no lint
  errors or warnings for the 8 script rows.
- `data_table_agent_packet_p0_v1.json` is included in the matrix generator default packet inputs.
- The regenerated matrix reports 17 `regression_locked` components and 0 `coverage_targeted`
  components.

## M10 - Close Harness Hardening

Status: complete on 2026-05-25.

`button-group.docs-demo.desktop` is promoted from `harness_hardening` to `regression_locked` after
superseding the pilot packet with a behavior-script-backed packet.

Completed criteria:

- The previous pilot packet remains the source of Button Group layout, semantics, upstream DOM, and
  text/paint proof for input, dropdown, select, and text compositions.
- The `ui-gallery-button-group-text-label-control-action-state` diagnostics script passes and
  proves ButtonGroupText roles, disabled text-label actions, enabled TextInput focus/set_value
  actions, `controls` / `labelled_by` relation edges, click-to-focus behavior, value mutation,
  layout sidecar, screenshot, schema2 bundle, AI packet, and share zip evidence.
- `button_group_agent_packet_p0_v1.json` supersedes the old pilot packet in the matrix generator
  default packet inputs with zero repair, hardening, and gate queue counts.
- The manifest records the behavior script under `button-group.docs-demo.desktop`, making the
  `BEHAV` axis visible in the regenerated matrix.
- The regenerated matrix reports 18 `regression_locked` components and no `harness_hardening` or
  `coverage_targeted` components.

## M11 - Continue Inventory Sweep

Status: complete on 2026-05-25.

`progress.docs-demo.desktop` is promoted from `inventory_only` to `regression_locked` after adding a
component-matrix packet for the existing Progress recipe, gallery, and diagnostics gates.

Completed criteria:

- The manifest records the upstream shadcn docs page, new-york source, docs-demo example, and
  `progress-demo` web golden under the Progress slice.
- The `ui-gallery-progress-numeric-semantics` diagnostics script passes and now captures a layout
  sidecar alongside the schema2 bundle, AI packet, and share zip.
- The `ui-gallery-progress-docs-smoke` diagnostics script passes and captures docs-path section,
  bundle, screenshot, AI packet, and share zip evidence.
- Focused recipe, snapshot, web-golden layout, web-golden chrome, gallery docs-surface, and gallery
  label-row layout gates pass.
- `progress_agent_packet_p0_v1.json` is included in the matrix generator default packet inputs with
  zero repair, hardening, and gate queue counts.
- The regenerated matrix reports 19 `regression_locked` components and 35 `inventory_only`
  components.

## M12 - Continue Inventory Sweep

Status: complete on 2026-05-25.

`badge.docs-demo.desktop` is promoted from `inventory_only` to `regression_locked` after correcting
the Badge proof surface and connecting a component-matrix packet.

Completed criteria:

- The manifest records upstream shadcn Badge docs/source refs plus default, secondary, destructive,
  and outline new-york web goldens.
- Badge recipe tests pass with foreground assertions that read inherited foreground/currentColor
  instead of assuming every label stamps an explicit `TextProps.color`.
- Badge snapshot and web-vs-Fret layout/chrome gates pass with style-aware text metrics, avoiding
  the old `FakeServices` 10x10 text metric that collapsed the docs link chip height.
- Runtime diagnostics pass for link render/action state and link hover screenshot evidence:
  `1779708458584` and `1779710381522`.
- `badge_agent_packet_p0_v1.json` is included in the matrix generator default packet inputs with
  zero repair, hardening, and gate queue counts.
- The regenerated matrix reports 20 `regression_locked` components and 34 `inventory_only`
  components.

## M13 - Continue Inventory Sweep

Status: complete on 2026-05-25.

`button.docs-demo.desktop` is promoted from `inventory_only` to `regression_locked` after connecting
the existing Button recipe, web-golden, gallery, semantic-link, and text-paint evidence.

Completed criteria:

- The manifest records upstream shadcn Button docs/source refs plus docs-demo, variant, focus,
  hover, pressed, disabled, icon, loading, size, rounded, with-icon, and as-child web goldens.
- Button recipe tests pass for variant/size chrome, state styling, action hooks, toggle helpers,
  child composition, link rendering, disabled behavior, and currentColor child propagation.
- Button web-vs-Fret layout and chrome gates pass for as-child/link geometry, grid auto-track
  intrinsic width, docs-demo chrome, shadows, focus ring, icon/loading/rounded/with-icon variants,
  and size variants.
- Runtime diagnostics pass for semantic-link ButtonRender behavior in run `1779711226806`, covering
  role, label, focus/invoke actions, keyboard dispatch, pointer dispatch, layout sidecar,
  screenshot, schema2 bundle, AI packet, and share zip evidence.
- `button_agent_packet_p0_v1.json` is included in the matrix generator default packet inputs with
  zero repair, hardening, and gate queue counts.
- The regenerated matrix reports 21 `regression_locked` components and 33 `inventory_only`
  components.

## M14 - Continue Inventory Sweep

Status: complete on 2026-05-25.

`accordion.docs-demo.desktop` is promoted from `inventory_only` to `regression_locked` after
connecting the existing Accordion recipe, web-golden, gallery docs-surface, and runtime diagnostics
evidence.

Completed criteria:

- The manifest records upstream shadcn Accordion docs/source refs plus the `accordion-demo` web
  golden.
- Accordion recipe tests pass for the focused filter, covering item/trigger/content composition,
  disabled and focusable-disabled policy, text-label roles, chevron/icon styling, border/card
  variants, RTL, and action wiring.
- Accordion web-vs-Fret layout gates pass for the docs-demo trigger/content geometry covered by
  the current golden.
- Runtime diagnostics pass for the typed-children Usage lane in run `1779712643033`, covering
  expanded=true, click-to-close expanded=false with panel unmount, and click-to-open expanded=true
  with panel remount.
- Runtime docs smoke passes in run `1779712752307`, covering Demo, Usage, Basic, Multiple,
  Disabled, Focusable Disabled, Borders, Card, RTL, and API Reference sections.
- `accordion_agent_packet_p0_v1.json` is included in the matrix generator default packet inputs
  with zero repair, hardening, and gate queue counts.
- The regenerated matrix reports 22 `regression_locked` components and 32 `inventory_only`
  components.

## M15 - Continue Inventory Sweep

Status: complete on 2026-05-25.

`alert.docs-demo.desktop` is promoted from `inventory_only` to `regression_locked` after connecting
the existing Alert recipe, web-golden, gallery docs-surface, and runtime diagnostics evidence.

Completed criteria:

- The manifest records upstream shadcn Alert docs/source refs plus the `alert-demo` and
  `alert-destructive` web goldens.
- Alert recipe tests pass for root role/chrome, title/description typography, icon currentColor,
  source-aligned grid tracks, action slot placement, builder paths, RTL, and destructive styling.
- Alert web-vs-Fret layout and chrome gates pass for w-full root geometry, default chrome, icon
  geometry/color, and destructive chrome.
- Runtime docs smoke passes in run `1779713738257`, covering page navigation, Usage, the three
  docs-demo rows, API Reference, and rich-title follow-up anchors.
- Runtime link activation passes in run `1779713789729`, covering role/label/value/action exposure,
  keyboard activation, pointer activation, layout sidecar, screenshots, bundle, AI packet, and
  share zip.
- Runtime action non-overlap passes in run `1779714219021`, covering With Actions title/button and
  badge lanes after calibrating the stale minimum-width threshold to the current docs card width.
- `alert_agent_packet_p0_v1.json` is included in the matrix generator default packet inputs with
  zero repair, hardening, and gate queue counts.
- The regenerated matrix reports 23 `regression_locked` components and 31 `inventory_only`
  components.

## M16 - Continue Inventory Sweep

Status: complete on 2026-05-25.

`alert-dialog.docs-demo.desktop` is promoted from `inventory_only` to `regression_locked` after
connecting existing Alert Dialog recipe, policy, web chrome, gallery docs-surface, upstream golden,
and runtime diagnostics evidence.

Completed criteria:

- The manifest records upstream shadcn/Radix Alert Dialog docs/source refs plus static, open,
  short-desktop, mobile, and Radix open-cancel goldens.
- Alert Dialog focused recipe and primitive tests pass for part composition, scoped action/cancel
  buttons, modal policy, least-destructive initial focus, focus restore, selectable description
  spans, responsive header/media grids, RTL, and open-change callbacks.
- Alert Dialog web-vs-Fret overlay chrome passes for the docs-demo panel chrome fixture.
- Runtime docs smoke passes in run `1779715657503`, covering the docs-path page, API Reference,
  extras, bundle, screenshot, AI packet, and share zip evidence.
- Runtime demo relation/action diagnostics pass in run `1779715728977`, covering expanded state,
  trigger controls relation, alertdialog labelled_by/described_by relations, modal barrier,
  least-destructive initial focus, cancel/action invoke actions, focus restore, layout sidecar,
  screenshot, bundle, AI packet, and share zip.
- Runtime destructive inline-link diagnostics pass in run `1779715860554`, covering selectable
  description text, inline link role/value/action, click activation, before/after screenshots,
  layout sidecar, bundle, AI packet, and share zip.
- `alert_dialog_agent_packet_p0_v1.json` is included in the matrix generator default packet inputs
  with zero repair, hardening, and gate queue counts.
- The regenerated matrix reports 24 `regression_locked` components and 30 `inventory_only`
  components.

## M17 - State-Depth Model

Status: complete on 2026-05-25.

`SCPM-050` is closed by extending the harness matrix beyond binary automation axes.

Completed criteria:

- The generator now emits `state_depth`, `required_state_depth`, and `missing_state_depth` per
  component.
- The state-depth model covers disabled, hover, focus-visible, pressed, open, keyboard, mobile, RTL,
  text metrics, and paint/token evidence.
- Requirements are component-specific, so missing states are planning signals only when they apply
  to that component surface.
- `MATRIX.md` now includes `Depth` and `Missing depth` columns plus a state-depth legend.
- The regenerated matrix keeps the same 59 components and still reports 24 `regression_locked`
  components, while exposing depth counts such as 10 keyboard-covered components, 8 mobile-covered
  components, and 6 text-metric-covered components.

## M18 - Continue Inventory Sweep

Status: complete on 2026-05-25.

`date-picker.docs-family.desktop-mobile` is promoted from `inventory_only` to
`regression_locked` after connecting the existing Date Picker audit, recipe tests, web-golden
layout/overlay fixtures, gallery docs-surface tests, upstream goldens, and runtime diagnostics
scripts into a component-matrix packet.

Completed criteria:

- The manifest records the shadcn Date Picker docs/source refs plus static, open, range, presets,
  and mobile web goldens.
- The Date Picker packet records trigger width ownership, default open-on-select behavior, explicit
  `close_on_select` behavior, trigger-level required/invalid semantics, mobile drawer presentation,
  and existing fixture-driven overlay placement/chrome coverage.
- `date_picker_agent_packet_p0_v1.json` is included in the matrix generator default packet inputs
  with zero repair, hardening, and gate queue counts.
- The regenerated matrix reports 25 `regression_locked` components and 29 `inventory_only`
  components.
- State-depth coverage now records Date Picker `OPEN`, `KEY`, `MOB`, and `PAINT` evidence; the
  next component gap remains a text/paint or paint-snapshot gate rather than a missing source,
  layout, semantics, behavior, or responsive proof.

## M19 - Continue Inventory Sweep

Status: complete on 2026-05-25.

`resizable.docs-path.desktop` is promoted from `inventory_only` to `regression_locked` after
connecting the existing Resizable audit, upstream goldens, web-vs-Fret layout fixtures, runtime
splitter/panel-group gates, gallery docs-surface tests, keyboard splitter diagnostics, handle
chrome screenshots, and adaptive panel resize proof.

Completed criteria:

- The state-depth model now includes a `DRAG` axis so splitter resize behavior is visible as a
  first-class coverage signal.
- The manifest records the shadcn Resizable docs/source refs plus demo, demo-with-handle, handle,
  and vertical web goldens.
- The Resizable packet records panel-group layout ownership, web-vs-Fret geometry, docs-surface
  copyability, stable gallery targets, keyboard Shift+Arrow resize behavior, RTL coverage, handle
  chrome, and adaptive panel resize diagnostics.
- `resizable_agent_packet_p0_v1.json` is included in the matrix generator default packet inputs
  with zero repair, hardening, and gate queue counts.
- The regenerated matrix reports 26 `regression_locked` components and 28 `inventory_only`
  components.
- State-depth coverage now records Resizable `DRAG`, `KEY`, `RTL`, and `PAINT` evidence; the next
  component gap remains a text/paint or paint-snapshot gate.

## M20 - Continue Inventory Sweep

Status: complete on 2026-05-25.

`sidebar.docs-path.desktop-mobile` is promoted from `inventory_only` to `harness_hardening` after
connecting the existing Sidebar audit, upstream goldens, web-vs-Fret layout fixtures, runtime
provider/mobile/menu diagnostics, gallery docs-surface tests, and AppSidebar dropdown evidence.

Completed criteria:

- The manifest records the shadcn Sidebar source refs plus tracked `sidebar-01`, `sidebar-13.open`,
  and `sidebar-16` web goldens.
- The Sidebar packet records provider shortcut behavior, controlled open sync, mobile sheet Escape
  focus restore, menu-button chrome/layout, AppSidebar dropdown relation/action state, docs-surface
  copyability, and stable gallery target coverage.
- `sidebar_agent_packet_p0_v1.json` is included in the matrix generator default packet inputs.
- The regenerated matrix reports 26 `regression_locked`, 1 `harness_hardening`, 27
  `inventory_only`, and 5 `not_in_harness` components.
- State-depth coverage now records Sidebar `HOV`, `FOCUS-VIS`, `OPEN`, `KEY`, `MOB`, `RTL`, and
  `PAINT` evidence after the recipe focus-visible ring gate. Targeted peer-size state gates now
  prove `SidebarMenuAction` and `SidebarMenuBadge` inherit same-row `SidebarMenuButton` size in the
  closure-composition path, and targeted active peer foreground gates now prove Action/Badge follow
  same-row active MenuButton foreground while GroupAction custom children inherit sidebar
  foreground. The row intentionally keeps `hardening=2` and `gate=2` for cookie/API-shape and
  residual peer/group/data-* class-state parity gaps.

## M21 - Continue Inventory Sweep

Status: complete on 2026-05-26.

`aspect-ratio.docs-demo.desktop` is promoted from `inventory_only` to `regression_locked` after
connecting the existing Radix/shadcn audit, kit primitive tests, web-vs-Fret geometry gate, Gallery
authoring-surface checks, and runtime screenshot/overlay diagnostics into a component-matrix
packet.

Completed criteria:

- The manifest records the Radix primitive, shadcn wrapper/docs/demo refs, and the
  `aspect-ratio-demo` web golden.
- The AspectRatio packet records the mechanism owner split: `fret-ui-kit` owns ratio geometry and
  the full-size content host, while `fret-ui-shadcn` remains a thin re-export and Gallery/demo code
  owns max width, rounded/background chrome, object-fit media, RTL caption layout, and direct
  multi-child overlay composition.
- Runtime diagnostics pass for docs smoke, demo screenshot, composable-children overlay, and RTL
  screenshot evidence.
- `aspect_ratio_agent_packet_p0_v1.json` is included in the matrix generator default packet inputs
  with zero repair, hardening, and gate queue counts.
- The regenerated matrix reports 27 `regression_locked`, 1 `harness_hardening`, 26
  `inventory_only`, and 5 `not_in_harness` components.
- State-depth coverage now records AspectRatio `RTL`, `TEXT-MET`, and `PAINT` evidence; the
  component has no required state-depth gaps.

## M22 - Continue Inventory Sweep

Status: complete on 2026-05-26.

`avatar.docs-demo.desktop` is promoted from `inventory_only` to `regression_locked` after connecting
the existing Avatar audit, Radix fallback-delay primitive tests, shadcn recipe tests, web-vs-Fret
geometry gates, Gallery authoring-surface checks, dropdown trigger relation/action diagnostics,
badge/group-count evidence, and fallback-only screenshots into a component-matrix packet.

Completed criteria:

- The manifest records shadcn/base Avatar docs/examples, Radix Avatar primitive refs, and tracked
  `avatar-demo`, `empty-avatar`, and `empty-avatar-group` web goldens.
- The Avatar packet records the owner split: `fret-ui-kit` owns delayed fallback/status mechanism,
  while `fret-ui-shadcn` owns root clipping, cover image paint, fallback chrome, badge overflow
  wrapper, group/count sizing, RTL badge/group behavior, and dropdown-trigger composition.
- Runtime diagnostics pass for docs screenshots, badge/group-count, dropdown relation/action state,
  and fallback-only screenshot evidence.
- `avatar_agent_packet_p0_v1.json` is included in the matrix generator default packet inputs with
  zero repair, hardening, and gate queue counts.
- The regenerated matrix reports 28 `regression_locked`, 1 `harness_hardening`, 25
  `inventory_only`, and 5 `not_in_harness` components.
- State-depth coverage now records Avatar `OPEN`, `KEY`, `RTL`, `TEXT-MET`, and `PAINT` evidence;
  the component has no required state-depth gaps.

## M23 - Continue Inventory Sweep

Status: complete on 2026-05-26.

`breadcrumb.docs-path.desktop-mobile` is promoted from `inventory_only` to `regression_locked`
after connecting the existing Breadcrumb audit, recipe semantics/layout tests, web-golden layout
gates, overlay placement fixtures, Gallery docs-surface checks, link command diagnostics, ellipsis
relation/action diagnostics, responsive dropdown/drawer behavior, custom separator text paint, and
RTL screenshots into a component-matrix packet.

Completed criteria:

- The manifest records shadcn Breadcrumb base/radix docs, new-york-v4 source refs, and tracked
  `breadcrumb-demo`, `breadcrumb-link`, `breadcrumb-ellipsis`, `breadcrumb-dropdown`,
  `breadcrumb-responsive.vp375x812`, and `breadcrumb-separator` web goldens.
- The Breadcrumb packet records the owner split: `fret-ui-shadcn` owns nav/list/item/current-page
  semantics, hidden separator/ellipsis affordances, href/action behavior, separator/ellipsis
  geometry, text style, responsive truncation, and overlay/dropdown composition; Gallery owns the
  docs-path teaching surface and advanced raw-primitive escape hatch examples.
- Runtime diagnostics pass for Usage link command semantics, Dropdown semantic-link behavior, Demo
  ellipsis relation/action state, custom separator text paint, responsive dropdown/drawer handoff,
  and RTL screenshot evidence.
- `breadcrumb_agent_packet_p0_v1.json` is included in the matrix generator default packet inputs
  with zero repair, hardening, and gate queue counts.
- The regenerated matrix reports 29 `regression_locked`, 1 `harness_hardening`, 24
  `inventory_only`, and 5 `not_in_harness` components.
- State-depth coverage now records Breadcrumb `DIS`, `HOV`, `OPEN`, `KEY`, `MOB`, `RTL`,
  `TEXT-MET`, and `PAINT` evidence; the component has no required state-depth gaps.

## M24 - Continue Inventory Sweep

Status: complete on 2026-05-26.

`field.docs-path.desktop-responsive` is promoted from `inventory_only` to `regression_locked`
after connecting the existing Field audit, recipe semantics/layout tests, web-golden layout
fixtures, field-local association tests, Gallery docs-surface checks, docs smoke diagnostics,
label/control relation diagnostics, responsive orientation diagnostics, password text paint, and
dark-theme radio screenshots into a component-matrix packet.

Completed criteria:

- The manifest records shadcn Field base/radix docs, base/radix source refs, and tracked
  `field-demo`, `field-input`, `field-textarea`, `field-select`, `field-slider`,
  `field-fieldset`, `field-checkbox`, `field-radio`, `field-switch`, `field-choice-card`,
  `field-group`, and `field-responsive` web goldens.
- The Field packet records the owner split: `fret-ui-shadcn` owns description/error layout,
  intrinsic label/title width, fieldset/group composition, field-local control association, and
  recipe text/chrome; Gallery owns the docs-path teaching surface and the responsive example shell
  width.
- The responsive orientation diagnostic exposed a real Gallery harness issue: the default
  `DocSection` width capped the example below the intended 900px container-query state. The Field
  page now gives the responsive section `.max_w(Px(980.0))`, keeping page width ownership in the
  Gallery layer while leaving recipe defaults unchanged.
- Runtime diagnostics pass for docs smoke, label/control relation state, narrow/wide responsive
  orientation, password text paint, and dark-theme radio chrome.
- `field_agent_packet_p0_v1.json` is included in the matrix generator default packet inputs with
  zero repair, hardening, and gate queue counts.
- The regenerated matrix reports 30 `regression_locked`, 1 `harness_hardening`, 23
  `inventory_only`, and 5 `not_in_harness` components.
- State-depth coverage now records Field `MOB`, `TEXT-MET`, and `PAINT` evidence; the component has
  no required state-depth gaps.

## M25 - Promote Form

Status: complete on 2026-05-26.

`form.docs-path.desktop` is promoted from `inventory_only` to `regression_locked` after connecting
the existing Form audit, recipe control-passthrough and field semantics tests, RHF/TanStack web
goldens, Gallery docs-surface checks, submit-validation and disabled-field diagnostics, and a
component-matrix packet.

Completed criteria:

- The manifest records shadcn Form docs context, RHF/TanStack goldens, and the tracked
  `form-rhf-*` / `form-tanstack-*` snapshot family.
- The Form packet records the owner split: `fret-ui-shadcn` owns FormControl slot passthrough and
  FormField required/invalid decoration, while concrete controls keep `disabled` / `read_only`
  ownership.
- The Gallery docs surface keeps the upstream teaching path plus the submit-validation and
  disabled-field teaching slices, and the docs-smoke script now starts directly on the Form page.
- Runtime diagnostics pass for docs smoke, submit validation, and disabled field action-state on
  the `gallery-dev` catalog.
- `form_agent_packet_p0_v1.json` is included in the matrix generator default packet inputs with
  zero repair, hardening, and gate queue counts.
- The regenerated matrix reports 31 `regression_locked`, 1 `harness_hardening`, 22
  `inventory_only`, and 5 `not_in_harness` components.
- State-depth coverage now records Form `DIS`, `RTL`, `TEXT-MET`, and `PAINT` evidence; the
  component has no required state-depth gaps.

## M26 - Promote Input Group

Status: complete on 2026-05-26.

`input-group.docs-path.desktop` is promoted from `inventory_only` to `regression_locked` after
connecting the existing Input Group audit, upstream input-group goldens, Fret layout gates, Gallery
docs-surface checks, runtime diagnostics, and a component-matrix packet.

Completed criteria:

- The manifest records the shadcn Input Group docs path and tracked `input-group-*` upstream
  goldens.
- The Input Group packet records the owner split: `fret-ui-shadcn` owns root `w-full min-w-0` and
  block addon row fill/min-width, while Gallery owns docs-shell composition and the narrow
  `custom_input(...)` / `custom_textarea(...)` teaching lane.
- The web-golden gate exposed real `input-group-textarea` trailing button drift from shrink-wrapped
  block addon rows; the recipe fix now fills those rows so auto-margin affordances align right.
- Runtime diagnostics pass for docs smoke, text non-overlap, button focus, label focus, addon tab
  focus, RTL screenshot, dropdown relation/action state, and RTL addon order.
- `input_group_agent_packet_p0_v1.json` is included in the matrix generator default packet inputs
  with zero repair, hardening, and gate queue counts.
- The regenerated matrix reports 32 `regression_locked`, 1 `harness_hardening`, 21
  `inventory_only`, and 5 `not_in_harness` components.
- State-depth coverage now records Input Group `DIS`, `FOCUS-VIS`, `OPEN`, `KEY`, `RTL`,
  `TEXT-MET`, and `PAINT` evidence; the component has no required state-depth gaps.

## M27 - Promote Pagination

Status: complete on 2026-05-26.

`pagination.docs-path.desktop` is promoted from `inventory_only` to `regression_locked` after
connecting the existing Pagination audit, upstream `pagination-demo` golden, recipe
semantics/chrome/keyboard tests, web-golden layout gates, Gallery docs-surface checks, runtime
action/selected diagnostics, rows-per-page Select overlay proof, and a component-matrix packet.

Completed criteria:

- The manifest records the shadcn Pagination docs path and tracked `pagination-demo` upstream
  golden.
- The Pagination packet records the owner split: `fret-ui-shadcn` owns root/list/link semantics,
  active selected state, link-like Enter activation, disabled/focus chrome, responsive previous/next
  text, RTL icon order, and hidden ellipsis; Gallery/app code owns routing actions and rows-per-page
  Select composition; a future dedicated Navigation landmark role remains a core follow-up.
- Runtime diagnostics pass for docs smoke, demo action/selected state plus command dispatch, and
  rows-per-page Select overlay screenshot/text paint.
- `pagination_agent_packet_p0_v1.json` is included in the matrix generator default packet inputs
  with zero repair, hardening, and gate queue counts.
- The regenerated matrix reports 33 `regression_locked`, 1 `harness_hardening`, 20
  `inventory_only`, and 5 `not_in_harness` components.
- State-depth coverage now records Pagination `DIS`, `FOCUS-VIS`, `OPEN`, `KEY`, `MOB`, `RTL`,
  `TEXT-MET`, and `PAINT` evidence; the component has no required state-depth gaps.

## M28 - Promote Card

Status: complete on 2026-05-26.

`card.docs-path.desktop` is promoted from `inventory_only` to `regression_locked` after connecting
the existing Card audit, current main-worktree `repo-ref` source refs, `card-demo` and
`card-with-form` upstream goldens, recipe slot/grid/chrome tests, Gallery docs-surface checks,
action-state diagnostics, text-paint follow-ups, RTL render-flow evidence, and a component-matrix
packet.

Completed criteria:

- The manifest records the current shadcn Card docs path, new-york-v4 visual source, base source,
  docs-demo and with-form examples, and tracked `card-demo` / `card-with-form` upstream goldens.
- The Card packet records the owner split: `fret-ui-shadcn` owns intrinsic card chrome, slot
  padding, title/description typography, header grid/action placement, and footer wrap budget;
  Gallery owns page width constraints, example form composition, media examples, and Fret-only
  rich text follow-ups; the runtime grid contract remains the mechanism layer proof for the
  `1fr auto` header slot family.
- Runtime diagnostics cover docs smoke and demo action-state behavior, while existing text-wrap,
  hitbox, image-cover, screenshot, composition, and meeting-notes scripts remain parse-checked
  coverage anchors.
- `card_agent_packet_p0_v1.json` is included in the matrix generator default packet inputs with
  zero repair, hardening, and gate queue counts.
- The regenerated matrix reports 34 `regression_locked`, 1 `harness_hardening`, 19
  `inventory_only`, and 5 `not_in_harness` components.
- State-depth coverage now records Card `HOV`, `FOCUS-VIS`, `KEY`, `RTL`, `TEXT-MET`, and `PAINT`
  evidence; the component has no required state-depth gaps.

## M29 - Promote Checkbox

Status: complete on 2026-05-26.

`checkbox.docs-path.desktop` is promoted from `inventory_only` to `regression_locked` after
connecting the existing Checkbox audit, current main-worktree `repo-ref` source refs, `checkbox-demo`
/ `checkbox-with-text` / `checkbox-disabled` / focus upstream goldens, recipe semantics tests,
web-vs-Fret layout/chrome gates, Gallery docs-surface checks, disabled/required/table diagnostics,
RTL/text/paint follow-ups, and a component-matrix packet.

Completed criteria:

- The manifest records the current shadcn Checkbox docs path, new-york-v4 visual source, registry
  demo/with-text/disabled/field examples, form checkbox examples, and tracked checkbox upstream
  goldens.
- The Checkbox packet records the owner split: `fret-ui-shadcn` owns the 16px leaf control,
  checked/disabled/required/invalid/indeterminate semantics, focus-visible chrome, and no-children
  API decision; Gallery/Field own labels, descriptions, fieldset framing, table composition, RTL
  layout, and larger click targets.
- Runtime diagnostics cover disabled action-state, required disabled group action-state, and table
  mixed-state behavior. The suite is intentionally run without `--reuse-launch` because each script
  starts at a different virtualized section.
- `checkbox_agent_packet_p0_v1.json` is included in the matrix generator default packet inputs with
  zero repair, hardening, and gate queue counts.
- The regenerated matrix reports 35 `regression_locked`, 1 `harness_hardening`, 18
  `inventory_only`, and 5 `not_in_harness` components.
- State-depth coverage now records Checkbox `DIS`, `FOCUS-VIS`, `KEY`, `RTL`, `TEXT-MET`, and
  `PAINT` evidence; the component has no required state-depth gaps.
