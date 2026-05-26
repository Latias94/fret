---
title: Shadcn Component Parity Matrix v1 TODO
status: active
date: 2026-05-26
---

# TODO

- [x] SCPM-010: Open a narrow workstream for shadcn component harness coverage, separate from the
      closed component fact harness v1 lane.
- [x] SCPM-020: Add a matrix generator that reads the canonical progress doc, coverage manifest,
      current suite report, and extra component packet artifacts.
- [x] SCPM-030: Generate the first machine-readable and human-readable component harness matrix.
- [x] SCPM-040: Pick the next P0 `inventory_only` or `coverage_targeted` component and promote it
      to a full harness seed with source refs, upstream snapshot, Fret `test_id`s, diagnostics
      script, and packet checks.
- [x] SCPM-050: Add a stricter depth model for states that are not visible in the current binary
      axes: disabled, hover, focus-visible, pressed, open, keyboard, mobile, RTL, text metrics, and
      paint/token output.
- [x] SCPM-060: Pick the next P0 row after Drawer and repeat the repair-seed loop.
- [x] SCPM-070: Pick the next P0 row after Calendar and repeat the repair-seed loop.
- [x] SCPM-080: Pick the next P0 row after Select and repeat the repair-seed loop.
- [x] SCPM-090: Pick the next P0 row after Combobox and repeat the repair-seed loop.
- [x] SCPM-100: Pick the next P0 row after Popover and repeat the repair-seed loop.
- [x] SCPM-110: Pick the next P0 row after Dropdown Menu and repeat the repair-seed loop.
- [x] SCPM-120: Pick the next P0 row after Input and repeat the repair-seed loop.
- [x] SCPM-130: Close the remaining Button Group harness hardening row with behavior diagnostics.
- [x] SCPM-140: Promote Progress from inventory-only to a regression-locked docs-demo harness seed.
- [x] SCPM-150: Promote Badge from inventory-only to a regression-locked docs-demo harness seed.
- [x] SCPM-160: Promote Button from inventory-only to a regression-locked docs-demo harness seed.
- [x] SCPM-170: Promote Accordion from inventory-only to a regression-locked docs-demo harness seed.
- [x] SCPM-180: Promote Alert from inventory-only to a regression-locked docs-demo harness seed.
- [x] SCPM-190: Promote Alert Dialog from inventory-only to a regression-locked docs-demo harness
      seed.
- [x] SCPM-200: Promote Date Picker from inventory-only to a regression-locked docs-family harness
      seed covering trigger width ownership, popover/calendar open behavior, trigger semantics,
      mobile drawer presentation, and existing web-golden overlay fixtures.
- [x] SCPM-210: Promote Resizable from inventory-only to a regression-locked docs-path harness seed
      covering splitter drag/keyboard behavior, panel-group layout ownership, RTL, handle chrome,
      and gallery docs-surface proof.
- [x] SCPM-220: Promote Sidebar from inventory-only to a harness-hardening docs-path seed covering
      provider shortcut behavior, controlled open sync, mobile sheet focus restore, menu-button
      chrome/layout, AppSidebar dropdown semantics, gallery docs-surface proof, and explicit
      hardening queues for cookie/API-shape and full class-state parity.
- [x] SCPM-230: Harden Sidebar focus-visible state-depth evidence with a recipe gate covering
      SidebarMenuButton, SidebarGroupAction, and SidebarMenuAction focus-ring wiring while keeping
      the remaining cookie/API-shape and full class-state queues explicit.
- [x] SCPM-240: Harden Sidebar href polymorphism by exposing link semantics/value and OpenUrl
      target/rel behavior for both default and as_child menu-button/menu-sub-button paths while
      keeping the remaining cookie/API-shape and full class-state queues explicit.
- [x] SCPM-250: Harden a targeted Sidebar peer class-state slice by letting SidebarMenuAction
      inherit same-row SidebarMenuButton size from the SidebarMenuItem closure context while keeping
      explicit action-size overrides and the remaining cookie/API-shape plus residual class-state
      queues explicit.
- [x] SCPM-260: Harden the matching SidebarMenuBadge peer-size slice by letting badges inherit
      same-row SidebarMenuButton size from the SidebarMenuItem closure context while keeping
      explicit badge-size overrides and the remaining cookie/API-shape plus residual class-state
      queues explicit.
- [x] SCPM-270: Harden a targeted Sidebar active peer foreground slice by letting
      SidebarMenuAction and SidebarMenuBadge inherit same-row SidebarMenuButton active foreground
      state, while locking SidebarGroupAction custom child foreground inheritance and keeping the
      remaining cookie/API-shape plus residual class-state queues explicit.
- [x] SCPM-280: Promote AspectRatio from inventory-only to a regression-locked docs-demo harness
      seed covering Radix/shadcn source refs, kit primitive mechanism tests, web-golden layout
      parity, Gallery docs-surface authoring checks, runtime screenshots, RTL, and direct
      multi-child overlay diagnostics.
- [x] SCPM-290: Promote Avatar from inventory-only to a regression-locked docs-demo harness seed
      covering Radix fallback-delay mechanism tests, shadcn recipe tests, web-golden layout parity,
      Gallery docs-surface authoring checks, dropdown trigger relation/action diagnostics, RTL,
      badge/group-count, and fallback text/paint screenshots.
- [x] SCPM-300: Promote Breadcrumb from inventory-only to a regression-locked docs-path harness
      seed covering nav/list/item/current-page semantics, hidden separator/ellipsis affordances,
      web-golden layout parity, overlay placement, Gallery docs-surface authoring checks, link
      command semantics, ellipsis relation/action diagnostics, responsive dropdown/drawer handoff,
      custom separator text paint, and RTL screenshots.
- [x] SCPM-310: Promote Field from inventory-only to a regression-locked docs-path harness seed
      covering description/error ownership, intrinsic label/title width, Input/Textarea/Select
      field-local association, responsive container orientation, password text paint, and
      dark-theme radio chrome.
- [x] SCPM-320: Promote Form from inventory-only to a regression-locked docs-path harness seed
      covering FormControl slot ownership, FormField required/invalid decoration, RHF/TanStack
      layout goldens, submit validation semantics, disabled control ownership, and Gallery
      docs-surface smoke coverage.
- [x] SCPM-330: Promote Input Group from inventory-only to a regression-locked docs-path harness
      seed covering root width ownership, block addon row fill, typed parts/compact shorthand,
      text non-overlap, button/label focus, keyboard tab order, dropdown relation/action state,
      and RTL logical slot order.
- [x] SCPM-340: Promote Pagination from inventory-only to a regression-locked docs-path harness
      seed covering root/list/link semantics, active selected state, Enter-only keyboard
      activation, responsive previous/next text, RTL icon order, ellipsis hidden semantics, app
      action dispatch, and rows-per-page Select overlay composition.
- [x] SCPM-350: Promote Card from inventory-only to a regression-locked docs-path harness seed
      covering root chrome, caller-owned width, source-aligned header grid/action slot placement,
      footer width budget, rich title/description children lanes, action-state diagnostics, RTL
      form controls, and text/paint follow-ups.
- [x] SCPM-360: Promote Checkbox from inventory-only to a regression-locked docs-path harness seed
      covering leaf-control chrome, required/invalid ownership, disabled action-state, label
      forwarding, indeterminate mixed state, focus-visible ring, RTL field composition, and
      text/paint follow-ups.
- [x] SCPM-370: Promote Collapsible from inventory-only to a regression-locked docs-path harness
      seed covering current shadcn docs/source refs, repository-list disclosure composition,
      controlled/uncontrolled open behavior, keyboard toggle, disabled trigger suppression,
      trigger/content semantics, RTL follow-up layout, and text/paint follow-ups.
- [x] SCPM-380: Promote Command from inventory-only to a regression-locked docs-path harness seed
      covering current shadcn docs/source refs, cmdk active-descendant/filtering behavior,
      CommandDialog overlay chrome and placement/list metrics, disabled keyboard suppression,
      shortcuts, RTL, and text/paint follow-ups.
- [x] SCPM-390: Promote Empty from inventory-only to a regression-locked docs-path harness seed
      covering current shadcn docs/source refs, empty-* upstream goldens, recipe chrome/text
      metrics, responsive padding, Gallery action/link semantics, RTL follow-up, and text/paint
      diagnostics.
- [x] SCPM-400: Promote Item from inventory-only to a regression-locked docs-path harness seed
      covering current shadcn docs/source refs, item-* upstream goldens, recipe slot/text/link
      semantics, web layout gates, Gallery docs ordering, link action diagnostics, RTL follow-up,
      and text/paint evidence.
- [x] SCPM-410: Promote Kbd from inventory-only to a regression-locked docs-path harness seed
      covering current shadcn docs/source refs, kbd-* upstream goldens, fixed keycap chrome,
      grouped shortcut spacing, tooltip-slot paint inversion, RTL order, Gallery docs ordering,
      and text/paint evidence.
- [x] SCPM-420: Promote Label from inventory-only to a regression-locked docs-path harness seed
      covering current shadcn docs/source refs, label upstream goldens, primitive association and
      disabled opacity, click forwarding, inline children, Gallery docs ordering, RTL follow-up,
      and text/paint evidence.
- [x] SCPM-430: Promote Radio Group from inventory-only to a regression-locked docs-path harness
      seed covering current shadcn docs/source refs, radio-group upstream goldens, primitive
      roving selection, recipe item chrome, invalid/disabled/focus-visible states, Gallery docs
      ordering, dropdown-menu radio composition, RTL follow-up, and text/paint evidence.
- [x] SCPM-440: Promote Scroll Area from inventory-only to a regression-locked docs-path harness
      seed covering current shadcn docs/source refs, vertical/horizontal scroll-area upstream
      goldens, runtime and primitive scroll behavior, recipe focus/visibility/drag/overflow
      gates, Gallery docs ordering, RTL follow-up, and text/paint evidence.
- [x] SCPM-450: Promote Separator from inventory-only to a regression-locked docs-path harness
      seed covering current shadcn docs/source refs, separator-demo upstream golden, primitive and
      recipe semantics, horizontal/vertical rule geometry and chrome, Gallery docs ordering,
      decorative-hidden diagnostics, RTL follow-up, and text/paint evidence.
- [x] SCPM-460: Promote Skeleton from inventory-only to a regression-locked docs-path harness
      seed covering current shadcn docs/source refs, skeleton-demo/card upstream goldens, recipe
      leaf chrome and pulse behavior, reduced-motion safety, Gallery docs ordering, base/radix
      example expansion, RTL follow-up, and text/paint evidence.
- [x] SCPM-470: Promote Spinner from inventory-only to a regression-locked docs-path harness
      seed covering current shadcn docs/source refs, spinner-* upstream goldens, status semantics,
      leaf chrome and spin behavior, disabled host controls, reduced-motion safety, Gallery docs
      ordering, Color/Item examples, RTL/Extras follow-ups, and text/paint evidence.
- [x] SCPM-480: Promote Textarea from inventory-only to a regression-locked docs-path harness
      seed covering current shadcn docs/source refs, textarea-* upstream goldens, leaf control
      chrome and semantics, resize drag clamping, label association, Gallery docs ordering, base/
      radix Field follow-up, diagnostics JSON, RTL, and text/paint evidence.
- [x] SCPM-490: Promote Toggle from inventory-only to a regression-locked docs-path harness seed
      covering current shadcn docs/source refs, toggle-* upstream goldens, recipe chrome and
      pressed/disabled/hover/focus-visible/keyboard behavior, Gallery docs ordering, split Small/
      Large examples, diagnostics JSON, RTL/Children/Label follow-ups, and text/paint evidence.
