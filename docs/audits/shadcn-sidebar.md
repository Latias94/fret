# shadcn/ui v4 Audit - Sidebar (Base UI aligned)


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- MUI Base UI: https://github.com/mui/base-ui
- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's `Sidebar` surface against upstream shadcn/ui v4 (`new-york-v4`) and
the underlying Base UI/Radix behavior contracts used by that surface.

Date: 2026-02-08

## Upstream references (source of truth)

- Sidebar recipe: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/sidebar.tsx`
- Sidebar demos/examples:
  - `repo-ref/ui/apps/v4/registry/new-york-v4/internal/sidebar-demo.tsx`
  - `repo-ref/ui/apps/v4/registry/new-york-v4/internal/sidebar-controlled.tsx`
  - `repo-ref/ui/apps/v4/registry/new-york-v4/internal/sidebar-menu-sub.tsx`
  - `repo-ref/ui/apps/v4/registry/new-york-v4/internal/sidebar-menu-collapsible.tsx`
- Upstream primitive wrappers used by sidebar:
  - Tooltip: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/tooltip.tsx`
  - Sheet (mobile sidebar path): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/sheet.tsx`

## Base UI contracts used by Sidebar

- Tooltip provider delay-group behavior (adjacent tooltip opens instantly after previous close):
  `repo-ref/base-ui/packages/react/src/tooltip/provider/TooltipProvider.tsx`
- Tooltip trigger hover/focus/open-delay behavior:
  `repo-ref/base-ui/packages/react/src/tooltip/trigger/TooltipTrigger.tsx`
- Dialog root controlled/uncontrolled open behavior (used by Sheet/mobile path):
  `repo-ref/base-ui/packages/react/src/dialog/root/DialogRoot.tsx`

## Fret implementation (current)

- Sidebar recipe: `ecosystem/fret-ui-shadcn/src/sidebar.rs`
- Exports: `ecosystem/fret-ui-shadcn/src/lib.rs`
- Related primitives already available in Fret:
  - Tooltip: `ecosystem/fret-ui-shadcn/src/tooltip.rs`
  - Sheet: `ecosystem/fret-ui-shadcn/src/sheet.rs`
  - Collapsible primitives: `ecosystem/fret-ui-kit/src/primitives/collapsible.rs`

## Snapshot summary

- Upstream sidebar exports: `24`
- Fret sidebar exports: `24`
- Missing in Fret: `0`

This means current sidebar parity is still a **partial surface alignment** despite broad golden-key
coverage for `sidebar-*` pages.

### Progress note (2026-02-08)

- Implemented: `SidebarProvider` + `use_sidebar` export surface.
- Implemented: `SidebarTrigger`, `SidebarInset`, `SidebarInput`, `SidebarSeparator` as first-pass
  parity surfaces.
- Implemented: `SidebarGroupAction`, `SidebarGroupContent`, `SidebarMenuAction`, `SidebarMenuBadge`
  as first-pass parity surfaces.
- Implemented: `SidebarRail`, `SidebarMenuSkeleton`, `SidebarMenuSub`, `SidebarMenuSubItem`,
  `SidebarMenuSubButton` as first-pass parity surfaces.
- Implemented: provider-driven collapsed state propagation for `Sidebar`, `SidebarContent`,
  `SidebarGroupLabel`, and `SidebarMenuButton`.
- Implemented: collapsed `SidebarMenuButton` hint path switched from `HoverCard` to `Tooltip`
  semantics (`TooltipContent`), with provider-level `delay_duration_frames(0)`.
- Added targeted tests for provider-driven collapse width and collapsed-tooltip semantics in
  `ecosystem/fret-ui-shadcn/src/sidebar.rs`.
- Added targeted tests for menu/list semantics (`SidebarMenu` + `SidebarMenuItem`), menu-action
  size-dependent top offsets, and collapsed affordance visibility (`GroupAction`/`MenuAction`/
  `MenuBadge`) in `ecosystem/fret-ui-shadcn/src/sidebar.rs`.
- Added targeted tests for `SidebarRail` provider toggle behavior, collapsed `SidebarMenuSkeleton`
  visibility, and nested `SidebarMenuSub*` list semantics/row-height invariants in
  `ecosystem/fret-ui-shadcn/src/sidebar.rs`.
- Added `SidebarSide` / `SidebarCollapsible` first-pass surface and internal context propagation,
  then wired `SidebarRail` side/offcanvas position matrix to that context.
- Added menu-item hover context propagation and desktop `show_on_hover` gating for
  `SidebarMenuAction` to better align with upstream peer/group hover intent.
- Added targeted tests for rail side/offcanvas matrix and `show_on_hover` reveal behavior.
- Added first-pass mobile `Sheet`/`Dialog` branch in `Sidebar` (`is_mobile=true` path) wired to
  `open_mobile`, with side-to-sheet-side mapping and a mobile width fallback (`18rem`).
- Corrected collapsed-state semantics for mobile provider paths (`open=false` no longer implies
  collapsed behavior when `is_mobile=true`).
- Added targeted tests for mobile sheet semantics/content rendering and mobile collapsed-state
  gating behavior.
- Added first-pass provider API alignment methods on `SidebarContext`:
  `set_open(...)` / `set_open_mobile(...)`.
- Added first-pass provider shortcut path: `Ctrl/Cmd+B` keymap binding to `sidebar.toggle`, plus
  provider-level `command_on_command` handling with focus-in-subtree availability gating.
- Added targeted tests for provider setter APIs, provider command handling, and keymap shortcut
  registration.
- Added first-pass `SidebarVariant` alignment (`sidebar` / `floating` / `inset`) for desktop width
  matrix and surface context propagation.
- Added first-pass `SidebarInset` peer-surface behavior for `variant=inset`, including collapsed
  left-margin step and floating-style radius/shadow treatment.
- Added targeted tests for variant width deltas and inset collapsed margin step in
  `ecosystem/fret-ui-shadcn/src/sidebar.rs`.
- Aligned `SidebarHeader` / `SidebarFooter` first-pass stacking contract to `flex-col gap-2 p-2`.
- Aligned `SidebarContent` first-pass core layout contract to `min-h-0 flex-1` with collapsed-only
  overflow hiding behavior (padding no longer coupled to collapse state).
- Aligned `SidebarGroup` first-pass wrapper contract to include `w-full + min-w-0 + relative`.
- Added targeted sidebar tests for header/footer gap contract, group relative-wrapper anchoring, and
  content collapsed overflow/padding behavior.
- Aligned `SidebarMenuButton` first-pass API contract for upstream-style polymorphism surface:
  `variant(default/outline)`, `href`, `on_navigate`, and `as_child` builder methods are now wired
  (with `as_child` paths retaining button semantics).
- Aligned `SidebarMenuAction` first-pass `as_child` path via pressable composition so custom
  children can be mounted without losing command/activate semantics.
- Added targeted tests for menu-button `href` activation semantics, outline variant baseline
  geometry invariant, and menu-action `as_child` command dispatch in
  `ecosystem/fret-ui-shadcn/src/sidebar.rs`.
- Aligned first-pass slot-like composition for `SidebarMenuButton` and `SidebarMenuSubButton`
  `as_child` paths by allowing explicit custom children while retaining pressable/button semantics.
- Added targeted tests for `SidebarMenuButton`/`SidebarMenuSubButton` `as_child` custom-child
  rendering behavior in `ecosystem/fret-ui-shadcn/src/sidebar.rs`.
- Aligned first-pass mobile hit-area behavior for `SidebarGroupAction`/`SidebarMenuAction` by
  expanding effective pressable bounds in mobile provider paths (`after:-inset-2` intent parity),
  and unified default + `as_child` rendering onto the same pressable semantics path to avoid
  icon-button intrinsic-size clamping.
- Added targeted tests asserting mobile-vs-desktop hit-area size deltas for
  `SidebarGroupAction`/`SidebarMenuAction` in `ecosystem/fret-ui-shadcn/src/sidebar.rs`.
- Added first-pass link-role parity for sidebar menu href paths by introducing
  `SemanticsRole::Link` (core + accesskit mapping) and switching default
  `SidebarMenuButton`/`SidebarMenuSubButton` href semantics to link role while keeping
  `as_child` href paths button-centric.
- Added targeted tests for href-link semantics and bare-href `Effect::OpenUrl` fallback behavior in
  `ecosystem/fret-ui-shadcn/src/sidebar.rs`.
- Added first-pass anchor API coverage on `SidebarMenuButton`/`SidebarMenuSubButton`
  (`target`/`rel`) and attached non-`as_child` `href` values into semantics snapshots for
  link-path automation/a11y inspection.
- Added targeted tests asserting `href` semantics-value exposure in link paths and explicit
  non-exposure for `as_child` href paths in `ecosystem/fret-ui-shadcn/src/sidebar.rs`.

### Update note (2026-02-23)

- Updated the UI gallery sidebar preview to demonstrate `SidebarProvider` composition (uncontrolled + controlled open model) and a forced-mobile `open_mobile` sheet path, removing the previous "approximation" note.
  - Preview: `apps/fret-ui-gallery/src/ui/previews/pages/components/composites/sidebar.rs`
- Added a scripted conformance gate that exercises the sidebar provider keyboard shortcut outcomes in the UI gallery (`Ctrl+B` toggles collapse/expand deterministically on desktop).
  - Script: `tools/diag-scripts/ui-gallery-sidebar-provider-shortcut-toggle-focus.json`
  - Suite: `crates/fret-diag/src/diag_suite_scripts.rs`
- Added a scripted conformance gate that exercises `SidebarProvider` controlled open model composition (external toggle + internal trigger stay in sync).
  - Script: `tools/diag-scripts/ui-gallery-sidebar-controlled-open-sync.json`
  - Suite: `crates/fret-diag/src/diag_suite_scripts.rs`
- Added a scripted conformance gate that exercises the mobile/offcanvas sheet path (`open_mobile`): Escape dismisses the sheet and focus restores to `SidebarTrigger`.
  - Script: `tools/diag-scripts/ui-gallery-sidebar-mobile-sheet-escape-focus-restore.json`
  - Suite: `crates/fret-diag/src/diag_suite_scripts.rs`
- Added a scripted conformance gate that exercises mobile `open_mobile` controlled sync: external open-model toggles and `SidebarTrigger`/Escape dismissal stay in sync.
  - Script: `tools/diag-scripts/ui-gallery-sidebar-mobile-controlled-open-sync.json`
  - Suite: `crates/fret-diag/src/diag_suite_scripts.rs`
- Added a scripted conformance gate that exercises the provider keyboard shortcut in forced-mobile mode (`Ctrl/Cmd+B` toggles the mobile sheet and restores focus).
  - Script: `tools/diag-scripts/ui-gallery-sidebar-mobile-shortcut-toggle.json`
  - Suite: `crates/fret-diag/src/diag_suite_scripts.rs`
- Fixed forced-mobile shortcut toggling when focus is inside the sheet: keymap shortcuts now dispatch into the UI tree command path (when handled) and `SidebarMenuButton` participates in handling `sidebar.toggle` so `Ctrl/Cmd+B` can close the sheet reliably.
- Added provider-owned width overrides (`width`, `width_icon`, `width_mobile`) with theme-token fallback remaining in the recipe, matching the upstream `SidebarProvider` width ownership model.
- Added targeted geometry tests for provider-owned desktop/mobile width overrides in `ecosystem/fret-ui-shadcn/src/sidebar.rs`.
- Reordered the UI gallery sidebar page to follow the docs-first flow (`Usage -> SidebarProvider -> Sidebar -> useSidebar`) while keeping existing diagnostic `test_id_prefix` anchors stable.

### Update note (2026-03-28)

- Added first-pass `SidebarGroupLabel::children(...)` + `.as_child(true)` support so the upstream `SidebarGroup` collapsible-label docs lane can be authored directly on the curated Fret surface.
- Added a targeted unit test asserting the `SidebarGroupLabel asChild` path renders a custom child and preserves the child trigger semantics in `ecosystem/fret-ui-shadcn/src/sidebar.rs`.
- Aligned `SidebarTrigger` with the upstream RTL recipe note by flipping the `PanelLeft` icon in RTL instead of treating it as a mechanism-layer concern.
- Aligned `SidebarGroupContent` with the upstream `w-full text-sm` default by scoping inherited `text-sm` typography on the group body wrapper.
- Aligned `SidebarRail` hit-box placement so the offcanvas path keeps the same rail box origin as the default desktop path, matching the upstream `translate-x-0` + `-right-2/-left-2` outcome instead of drifting inward.
- Updated the UI gallery sidebar `Structure` section to keep the official `SidebarGroupLabel asChild + CollapsibleTrigger` composition lane copyable instead of leaving it implicit.
- Added a docs-surface regression test for the sidebar page/snippet teaching surface in `apps/fret-ui-gallery/tests/sidebar_docs_surface.rs`.

## Component-by-component audit (24/24)

Status legend:

- `Aligned`: exists and behavior largely matches upstream intent.
- `Partial`: exists but behavior/composition diverges in meaningful ways.
- `Missing`: not implemented/exported yet.

| Component | Upstream role | Base UI/Radix contract touchpoint | Fret status | Primary gap | Owner layer |
| --- | --- | --- | --- | --- | --- |
| `SidebarProvider` | Owns `open/openMobile/state`, keyboard toggle, tooltip provider | Tooltip delay-group + controlled/uncontrolled open model | Partial | Core state, tooltip delay-group, provider-owned width overrides (`width` / `width_icon` / `width_mobile`), first-pass `Ctrl/Cmd+B` shortcut handling, provider callbacks (`on_open_change` / `on_open_mobile_change`), and function-style setter ergonomics (`set_open_with` / `set_open_mobile_with`) are implemented; cookie persistence and full React API-shape parity remain | `fret-ui-shadcn` |
| `use_sidebar` | Access provider state/actions | Context read contract | Partial | Hook exists and now exposes `set_open/set_open_mobile` and function-style setters (`set_open_with` / `set_open_mobile_with`) on context; parity gaps remain around cookie-backed persistence and full React API-shape parity | `fret-ui-shadcn` |
| `Sidebar` | Desktop shell + mobile sheet branch; side/variant/collapsible data-state channel | Sheet/Dialog for mobile | Partial | `side/collapsible` + mobile `Sheet` are in place, the desktop/mobile width matrix now reads provider-owned resolved widths first, and theme tokens remain recipe fallback; richer data-slot channels and cookie persistence parity remain | `fret-ui-shadcn` |
| `SidebarTrigger` | Toggle sidebar state | Provider action + button semantics | Partial | Toggle behavior and RTL icon flip are wired; upstream data-slot conventions and full keyboard shortcut/API-shape parity remain to align | `fret-ui-shadcn` |
| `SidebarRail` | Thin rail toggle affordance | Provider action + pointer affordance | Partial | Rail toggle + side/offcanvas hit-box placement matrix are wired, and hover now requests `CursorIcon::ColResize` as a first-pass cursor affordance; pseudo-element hit-area parity, richer directional cursor semantics, and full class-state styling parity remain | `fret-ui-shadcn` |
| `SidebarInset` | Peer/inset content container | None (layout recipe) | Partial | First-pass `variant=inset` peer-surface matrix is now wired (margin/radius/shadow + collapsed margin step); responsive breakpoint choreography and full class-state parity remain | `fret-ui-shadcn` |
| `SidebarInput` | Sidebar-local input style wrapper | None (input styling wrapper) | Partial | 32px height + background wrapper exists; full class-level state variants are still narrower than upstream | `fret-ui-shadcn` |
| `SidebarSeparator` | Sidebar-local separator wrapper | None (separator styling wrapper) | Partial | Sidebar-border + horizontal wrapper exists; upstream data-slot/class and variant nuances remain | `fret-ui-shadcn` |
| `SidebarHeader` | `flex-col gap-2 p-2` header region | None (layout recipe) | Partial | Default `gap-2` + `p-2` stack contract is now wired; data-slot/channel parity and `asChild` ergonomics remain | `fret-ui-shadcn` |
| `SidebarFooter` | `flex-col gap-2 p-2` footer region | None (layout recipe) | Partial | Default `gap-2` + `p-2` stack contract is now wired; data-slot/channel parity and `asChild` ergonomics remain | `fret-ui-shadcn` |
| `SidebarContent` | `min-h-0 flex-1 overflow-auto`, icon-collapsed overflow hidden | None (layout recipe) | Partial | Core `min-h-0 flex-1` + collapsed overflow contract is now wired; responsive/variant choreography and full class-state channel parity remain | `fret-ui-shadcn` |
| `SidebarGroup` | Group container (`relative`, `min-w-0`, `p-2`) | None (layout recipe) | Partial | Wrapper `relative + min-w-0 + w-full + p-2` contract is now wired; full upstream slot/state class matrix remains | `fret-ui-shadcn` |
| `SidebarGroupLabel` | Collapsed animation (`-mt-8`, `opacity-0`), focus ring styling | None (layout recipe) | Partial | Provider-driven collapse motion is wired, and first-pass `children(...)` + `as_child(true)` composition now covers the upstream collapsible-group label lane; remaining gaps are full slot/class-state parity and more literal slot-root styling parity | `fret-ui-shadcn` |
| `SidebarGroupAction` | Group-level action button (absolute position, focus ring) | Button semantics | Partial | Absolute action surface + mobile hit-area expansion (`after:-inset-2` intent) are wired via unified pressable semantics (default and `as_child`); remaining gaps are full pseudo-element/state-class parity and richer asChild composition semantics | `fret-ui-shadcn` |
| `SidebarGroupContent` | Group body wrapper | None (layout recipe) | Partial | First-pass `w-full + text-sm` wrapper exists; upstream data-slot/class matrix parity still missing | `fret-ui-shadcn` |
| `SidebarMenu` | Menu list container (`ul`-like semantics) | None (list semantics) | Partial | List semantics (`SemanticsRole::List`) now present; upstream data-slot/class matrix parity still missing | `fret-ui-shadcn` |
| `SidebarMenuItem` | Menu item container (`li`-like semantics) | None (list item semantics) | Partial | Relative list-item semantics + hover context are present; full group/peer class-state and `asChild` parity still missing | `fret-ui-shadcn` |
| `SidebarMenuButton` | Core action row; active/size variants; collapsed tooltip | Tooltip trigger/content contract | Partial | Collapsed tooltip + `variant(default/outline)` + `href/on_navigate` + `as_child` API surface are now wired, `as_child` supports custom-child composition, default `href` path now uses `SemanticsRole::Link`, non-`as_child` `href` now populates semantics value, and `href` falls back to `Effect::OpenUrl` when no `on_navigate` is provided; remaining gaps are true anchor polymorphism (native link attributes/render target) and full class-state parity (`peer/group/data-*`) | `fret-ui-shadcn` |
| `SidebarMenuAction` | Per-row action button | Button semantics | Partial | Size/top/collapsed surface + desktop hover-gated visibility exist, and both default/`as_child` paths now use unified pressable semantics that preserve command/activate behavior while enabling mobile hit-area expansion; remaining gaps are pseudo-element/state-class parity and full peer/group state matrix parity | `fret-ui-shadcn` |
| `SidebarMenuBadge` | Per-row badge slot | None (layout/styling wrapper) | Partial | First-pass absolute badge surface (size-dependent top offsets, collapsed hide) exists; pointer-events/class-state matrix and tabular-number styling parity are incomplete | `fret-ui-shadcn` |
| `SidebarMenuSkeleton` | Loading skeleton row | None (layout/styling wrapper) | Partial | First-pass skeleton row surface exists; upstream random width strategy and icon/text slot data markers are simplified | `fret-ui-shadcn` |
| `SidebarMenuSub` | Nested menu list wrapper | None (list semantics) | Partial | Nested sub-menu wrapper + list semantics exist; exact class transforms/spacing matrix still simplified | `fret-ui-shadcn` |
| `SidebarMenuSubItem` | Nested menu item wrapper | None (list item semantics) | Partial | Nested sub-item wrapper + list-item semantics exist; peer/group class-state parity remains incomplete | `fret-ui-shadcn` |
| `SidebarMenuSubButton` | Nested row button/link wrapper | Button/link semantics | Partial | Nested sub-button surface exists with active/size/collapsed behavior, first-pass `as_child` custom-child composition is wired, default `href` path now uses `SemanticsRole::Link`, non-`as_child` `href` now populates semantics value, and `href` falls back to `Effect::OpenUrl` when no `on_navigate` is provided; remaining gaps are native anchor semantics parity (attributes/render target) and full class matrix parity | `fret-ui-shadcn` |

## Key divergences and likely root causes

### 1) Tooltip contract (collapsed mode)

- Upstream `SidebarMenuButton` uses `Tooltip` and inherits provider timing (`delayDuration=0` in
  sidebar provider path).
- Fret now mirrors this with `TooltipContent` and provider-level delay-group defaults.

Remaining impact/gap:

- Variant/polymorphism is improved (first-pass `variant/href/as_child` surface wired), and default
  `href` paths now expose link semantics; true anchor rendering/attribute parity and full
  slot/class parity are still narrower than upstream.

### 2) Provider-owned state model

- Upstream owns `open`, `openMobile`, `state`, `toggleSidebar`, and keyboard shortcut in provider.
- Fret now provides core `open/open_mobile/state/toggle` context flow and propagates collapsed
  state to key sidebar primitives.

Remaining impact/gap:

- Keyboard shortcut is now present as a first pass (`Ctrl/Cmd+B -> sidebar.toggle`), and
  provider-level change callbacks (`on_open_change` / `on_open_mobile_change`) plus function-style
  setter ergonomics (`set_open_with` / `set_open_mobile_with`) are now available.
  Cookie persistence and full React API-shape parity are still TODO.
- Mobile `openMobile` sheet path is now surfaced in a first pass; remaining gaps are richer
  callback shape (`setOpen`/`setOpenMobile` parity) and cookie persistence semantics.

### 3) Behavioral parity gap (0 missing exports)

- Core sidebar export surface is now complete, but several behaviors are still simplified compared
  with upstream class-state contracts.

Impact:

- Upstream examples can be ported with lower structural friction, but behavior-level adaptation is
  still required in advanced cases.
- Existing `sidebar-*` goldens can pass while interaction/state parity remains partial.

### 4) Semantics and polymorphism gaps

- Upstream frequently composes via `asChild` and semantic list structure (`ul/li` wrappers).
- Fret now supports first-pass `as_child` composition on key menu rows, routes bare `href`
  activations to `Effect::OpenUrl`, default `href` paths now expose link semantics, and non-
  `as_child` href values are surfaced via semantics value.
- Remaining gap is dedicated anchor primitives/attributes for full upstream polymorphism parity.

Impact:

- Accessibility/semantics parity may lag in real app compositions.
- Porting upstream examples requires extra per-app adaptation.

## Test/gate status and blind spots

- Existing sidebar-targeted gates validate menu-button heights and portal placement cases (`sidebar-13`), plus core provider choreography via scripted UI gallery gates (shortcut toggle, controlled open sync, and mobile sheet Escape focus restore).
- This creates a breadth/depth mismatch: key coverage can be high while component parity remains
  partial.

## Implementation plan (recommended)

1. `P0` Done: align collapsed tooltip behavior (`Tooltip` + provider-driven delay group).
2. `P0` Done (partial): introduce `SidebarProvider` + `use_sidebar` core; `SidebarTrigger` first-pass landed.
3. `P1` Done (first pass): structural wrappers (`Inset`, `Input`, `Separator`, `GroupContent`, `GroupAction`) landed.
4. `P1` Done (first pass): menu auxiliary surfaces (`MenuAction`, `Badge`, `MenuSkeleton`, `MenuSub*`) landed.
5. `P1` Done (first pass): mobile sheet branch + provider shortcut/setter APIs landed.
6. `P1` Done (first pass): `side/variant/collapsible` matrix and inset peer behavior are now
   wired with targeted invariant tests.
7. `P2` Add targeted tests and `diag` scripts for focus-within/open-state choreography,
   polymorphism paths, and responsive breakpoint parity.

## Archived/superseded notes

This document supersedes the older shorthand sidebar statement in
`docs/audits/shadcn-new-york-v4-alignment.md` that treated sidebar as a primarily
menu-height-focused gate area. That statement remains valid for the narrow scope it described,
but not for full component-surface parity.
