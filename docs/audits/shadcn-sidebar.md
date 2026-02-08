# shadcn/ui v4 Audit - Sidebar (Base UI aligned)

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
- Fret sidebar exports: `11`
- Missing in Fret: `13`

This means current sidebar parity is still a **partial surface alignment** despite broad golden-key
coverage for `sidebar-*` pages.

### Progress note (2026-02-08)

- Implemented: `SidebarProvider` + `use_sidebar` export surface.
- Implemented: provider-driven collapsed state propagation for `Sidebar`, `SidebarContent`,
  `SidebarGroupLabel`, and `SidebarMenuButton`.
- Implemented: collapsed `SidebarMenuButton` hint path switched from `HoverCard` to `Tooltip`
  semantics (`TooltipContent`), with provider-level `delay_duration_frames(0)`.
- Added targeted tests for provider-driven collapse width and collapsed-tooltip semantics in
  `ecosystem/fret-ui-shadcn/src/sidebar.rs`.

## Component-by-component audit (24/24)

Status legend:

- `Aligned`: exists and behavior largely matches upstream intent.
- `Partial`: exists but behavior/composition diverges in meaningful ways.
- `Missing`: not implemented/exported yet.

| Component | Upstream role | Base UI/Radix contract touchpoint | Fret status | Primary gap | Owner layer |
| --- | --- | --- | --- | --- | --- |
| `SidebarProvider` | Owns `open/openMobile/state`, keyboard toggle, tooltip provider | Tooltip delay-group + controlled/uncontrolled open model | Partial | Core provider state and tooltip delay-group are implemented; keyboard shortcut and richer callback surface are still missing | `fret-ui-shadcn` |
| `useSidebar` | Access provider state/actions | Context read contract | Partial | Hook exists (`use_sidebar`), but parity gaps remain versus upstream API shape (`setOpen/setOpenMobile`-style helpers) | `fret-ui-shadcn` |
| `Sidebar` | Desktop shell + mobile sheet branch; side/variant/collapsible data-state channel | Sheet/Dialog for mobile | Partial | No mobile branch; no `side/variant/collapsible` behavior matrix | `fret-ui-shadcn` |
| `SidebarTrigger` | Toggle sidebar state | Provider action + button semantics | Missing | No trigger component | `fret-ui-shadcn` |
| `SidebarRail` | Thin rail toggle affordance | Provider action + pointer affordance | Missing | No rail component or cursor-state logic | `fret-ui-shadcn` |
| `SidebarInset` | Peer/inset content container | None (layout recipe) | Missing | Missing shell composition primitive | `fret-ui-shadcn` |
| `SidebarInput` | Sidebar-local input style wrapper | None (input styling wrapper) | Missing | Missing component surface | `fret-ui-shadcn` |
| `SidebarSeparator` | Sidebar-local separator wrapper | None (separator styling wrapper) | Missing | Missing component surface | `fret-ui-shadcn` |
| `SidebarHeader` | `flex-col gap-2 p-2` header region | None (layout recipe) | Partial | Missing explicit `gap-2` default in header slot contract | `fret-ui-shadcn` |
| `SidebarFooter` | `flex-col gap-2 p-2` footer region | None (layout recipe) | Partial | Missing explicit `gap-2` default in footer slot contract | `fret-ui-shadcn` |
| `SidebarContent` | `min-h-0 flex-1 overflow-auto`, icon-collapsed overflow hidden | None (layout recipe) | Partial | Provider-driven collapse is now wired, but broader mobile/variant matrix parity is still missing | `fret-ui-shadcn` |
| `SidebarGroup` | Group container (`relative`, `min-w-0`, `p-2`) | None (layout recipe) | Partial | Missing relative/min-w state contract parity | `fret-ui-shadcn` |
| `SidebarGroupLabel` | Collapsed animation (`-mt-8`, `opacity-0`), focus ring styling | None (layout recipe) | Partial | Provider-driven collapse motion is wired, but class-level transform/state channel parity is still incomplete | `fret-ui-shadcn` |
| `SidebarGroupAction` | Group-level action button (absolute position, focus ring) | Button semantics | Missing | Missing component surface | `fret-ui-shadcn` |
| `SidebarGroupContent` | Group body wrapper | None (layout recipe) | Missing | Missing component surface | `fret-ui-shadcn` |
| `SidebarMenu` | Menu list container (`ul`-like semantics) | None (list semantics) | Partial | No explicit list semantics channel by default | `fret-ui-shadcn` |
| `SidebarMenuItem` | Menu item container (`li`-like semantics) | None (list item semantics) | Partial | No explicit list-item semantics channel by default | `fret-ui-shadcn` |
| `SidebarMenuButton` | Core action row; active/size variants; collapsed tooltip | Tooltip trigger/content contract | Partial | Collapsed tooltip now uses `Tooltip` semantics; remaining gaps are variant/asChild/link-polymorphism parity | `fret-ui-shadcn` |
| `SidebarMenuAction` | Per-row action button | Button semantics | Missing | Missing component surface | `fret-ui-shadcn` |
| `SidebarMenuBadge` | Per-row badge slot | None (layout/styling wrapper) | Missing | Missing component surface | `fret-ui-shadcn` |
| `SidebarMenuSkeleton` | Loading skeleton row | None (layout/styling wrapper) | Missing | Missing component surface | `fret-ui-shadcn` |
| `SidebarMenuSub` | Nested menu list wrapper | None (list semantics) | Missing | Missing component surface | `fret-ui-shadcn` |
| `SidebarMenuSubItem` | Nested menu item wrapper | None (list item semantics) | Missing | Missing component surface | `fret-ui-shadcn` |
| `SidebarMenuSubButton` | Nested row button/link wrapper | Button/link semantics | Missing | Missing component surface | `fret-ui-shadcn` |

## Key divergences and likely root causes

### 1) Tooltip contract (collapsed mode)

- Upstream `SidebarMenuButton` uses `Tooltip` and inherits provider timing (`delayDuration=0` in
  sidebar provider path).
- Fret now mirrors this with `TooltipContent` and provider-level delay-group defaults.

Remaining impact/gap:

- Variant/polymorphism (`asChild`/link-like composition) is still narrower than upstream.

### 2) Provider-owned state model

- Upstream owns `open`, `openMobile`, `state`, `toggleSidebar`, and keyboard shortcut in provider.
- Fret now provides core `open/open_mobile/state/toggle` context flow and propagates collapsed
  state to key sidebar primitives.

Remaining impact/gap:

- Keyboard shortcut integration and full API-shape parity remain TODO.
- Mobile sheet state channel (`openMobile` branch behavior) is not fully surfaced yet.

### 3) Surface completeness gap (13 missing exports)

- Many upstream sidebar composition blocks are not available in Fret, especially action and nested
  menu helpers.

Impact:

- Upstream examples cannot be ported 1:1.
- Existing `sidebar-*` goldens can pass while feature surface remains incomplete.

### 4) Semantics and polymorphism gaps

- Upstream frequently composes via `asChild` and semantic list structure (`ul/li` wrappers).
- Current Fret sidebar uses fixed button-centric composition for menu rows.

Impact:

- Accessibility/semantics parity may lag in real app compositions.
- Porting upstream examples requires extra per-app adaptation.

## Test/gate status and blind spots

- Existing sidebar-targeted gates mostly validate menu-button heights and one dialog portal
  placement case (`sidebar-13`), not full surface behavior.
- This creates a breadth/depth mismatch: key coverage can be high while component parity remains
  partial.

## Implementation plan (recommended)

1. `P0` Done: align collapsed tooltip behavior (`Tooltip` + provider-driven delay group).
2. `P0` Done (partial): introduce `SidebarProvider` + `use_sidebar` core; next add `SidebarTrigger`.
3. `P1` Add structural wrappers (`Inset`, `Input`, `Separator`, `GroupContent`, `GroupAction`).
4. `P1` Add menu auxiliary surfaces (`MenuAction`, `Badge`, `Skeleton`, `MenuSub*`).
5. `P1` Add mobile sheet branch + side/variant/collapsible matrix.
6. `P2` Add targeted tests and `diag` scripts for state-machine flows (not only geometry).

## Archived/superseded notes

This document supersedes the older shorthand sidebar statement in
`docs/audits/shadcn-new-york-v4-alignment.md` that treated sidebar as a primarily
menu-height-focused gate area. That statement remains valid for the narrow scope it described,
but not for full component-surface parity.
