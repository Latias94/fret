---
title: shadcn/ui v4 Component Parity TODO (fret-components-shadcn alignment)
---

# shadcn/ui v4 Component Parity TODO (fret-components-shadcn alignment)

This document tracks actionable TODOs for aligning Fret’s shadcn-facing crate:

- `crates/fret-components-shadcn` (naming + taxonomy surface)

with the pinned upstream reference:

- `repo-ref/ui` (shadcn/ui v4 docs + registry implementation)

The intent is “knowledge transfer parity”: users familiar with shadcn/ui should be able to identify
the same component concepts and compose similar UIs, even if the underlying runtime is not DOM.

## Snapshot (for traceability)

- Fret repo base: `19df1158a3f5c9e41c0759e8d1b42f4e91de18da`
- shadcn/ui HEAD: `ccafdaf7c6f6747a24f54e84436b42ec42f01779`

## Scope and non-goals

- Scope:
  - Ensure `fret-components-shadcn` exposes a shadcn-like taxonomy for the subset of components we
    already have (even if implemented in `fret-components-ui`).
  - Track missing shadcn components and decide where they should live (`fret-components-ui` infra vs
    `fret-components-shadcn` policy/wrappers).
- Non-goals:
  - Exact API parity with Radix subcomponents (`DialogTrigger`, `DialogContent`, etc.).
    Fret’s overlay model is service/layer driven (see ADR 0011), so parity is about behavior and
    composition outcomes, not 1:1 React component structure.
  - TanStack Table parity for `DataTable` (Fret will use its own virtualization/table contracts).

## Reference sources in `repo-ref/ui`

- Component docs (canonical list): `repo-ref/ui/apps/v4/content/docs/components/`
- Registry UI implementations: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/`

Note: the registry index references `toast` files that are missing in this pinned checkout:
`repo-ref/ui/apps/v4/registry/new-york-v4/ui/_registry.ts:604` points to `ui/toast.tsx`,
`ui/toaster.tsx`, and `hooks/use-toast.ts`, but those files are not present under
`repo-ref/ui/apps/v4/registry/new-york-v4/` at this commit. Prefer using `sonner` as the upstream
reference for toasts until the pin is updated.

## Current Fret shadcn surface (what exists today)

`crates/fret-components-shadcn/src/lib.rs` currently exposes:

- `button::Button` (custom wrapper implementing shadcn variants/sizes)
- First-pass styling helpers: `alert`, `badge`, `card`, `empty`, `kbd`, `label`
- Re-exports (thin aliases) for: `input`, `textarea`, `separator`, `checkbox`, `switch`, `tabs`,
  `select`, `tooltip`, `toast`, `popover`, `dialog`, `dropdown_menu`, `command`, plus facade modules
  for `menubar`, `context_menu`, `combobox`, `scroll_area`, `progress`, `slider`, `sonner`,
  `resizable`
- Infrastructure re-exports: `ChromeRefinement`, `LayoutRefinement`, `Space`, `Radius`, `Size`, `StyledExt`

Note: the overlay installation helper is exposed as `fret_components_shadcn::WindowOverlays` (thin
alias of `fret_components_ui::WindowOverlays`) and is the canonical way to wire standard overlays
into a `UiTree`.

## Parity levels (how we judge “done”)

For each shadcn component, track progress in these layers:

1) **Taxonomy parity**: a discoverable module/type exists in `fret-components-shadcn`.
2) **Recipe parity**: can build the same *composition result* as shadcn docs show.
3) **Interaction parity**: keyboard/focus/dismissal behaviors match shadcn expectations.
4) **Token parity**: uses shadcn/gpui semantic palette keys via `Theme::color_by_key` aliases (ADR 0050).

## Component parity matrix (v4)

Status legend:

- ✅ = present in `fret-components-shadcn` (at least taxonomy parity)
- ◐ = implemented in `fret-components-ui` but not surfaced as shadcn taxonomy yet
- ❌ = missing (no implementation in-tree yet)
- — = non-goal (tracked for transparency)

| shadcn v4 component | Upstream ref | Fret mapping (today) | Status | Notes |
| --- | --- | --- | --- | --- |
| Accordion | `ui/accordion.tsx` | `fret_components_shadcn::Accordion*` | ✅ | First pass: selection model drives open/close; no animation or built-in chevron. |
| Alert | `ui/alert.tsx` | `fret_components_shadcn::Alert` | ✅ | First-pass composition helper; verify token parity. |
| Alert Dialog | `ui/alert-dialog.tsx` | `fret_components_shadcn::AlertDialogRequest` | ✅ | Policy wrapper over `DialogOverlay`: cancel closes without a command by default. |
| Aspect Ratio | `ui/aspect-ratio.tsx` | `fret_components_shadcn::AspectRatio` | ✅ | Declarative helper over ADR 0057 aspect ratio semantics. |
| Avatar | `ui/avatar.tsx` | `fret_components_shadcn::{Avatar, AvatarImage, AvatarFallback}` | ✅ | Declarative composition: clipped rounded root + image/fallback layers. |
| Badge | `ui/badge.tsx` | `fret_components_shadcn::Badge` | ✅ | First-pass composition helper; verify variants. |
| Breadcrumb | `ui/breadcrumb.tsx` | `fret_components_shadcn::{Breadcrumb, BreadcrumbItem}` | ✅ | Builder renders shadcn-style link/page + separators + ellipsis. |
| Button | `ui/button.tsx` | `fret_components_shadcn::Button` | ✅ | Wrapper exists; verify hover/active/disabled parity. |
| Button Group | `ui/button-group.tsx` | `fret_components_shadcn::{ButtonGroup, ButtonGroupItem}` | ✅ | Segmented buttons (radius merge + border collapse) with keyboard navigation. |
| Calendar | `ui/calendar.tsx` | `fret_components_shadcn::Calendar` | ✅ | Retained widget: month view + prev/next + single-date selection (no range/multi-month yet). |
| Date Picker | `ui/date-picker.tsx` | `fret_components_shadcn::DatePicker` | ✅ | Trigger opens `PopoverSurfaceOverlay` anchored to the button; content node is app-installed (typically `Calendar::on_select(\"popover_surface.close\")`). |
| Card | `ui/card.tsx` | `fret_components_shadcn::Card*` | ✅ | First-pass slots exist; verify spacing + typography tokens. |
| Carousel | `ui/carousel.tsx` |  | — | Non-goal (v1). |
| Chart | `ui/chart.tsx` |  | — | Non-goal (v1); depends on a charting substrate. |
| Checkbox | `ui/checkbox.tsx` | `fret_components_shadcn::checkbox::*` | ✅ | Validate focus-visible + keyboard toggle. |
| Collapsible | `ui/collapsible.tsx` | `fret_components_shadcn::Collapsible*` | ✅ | First pass: `Model<bool>` drives open/close; trigger handles click + Enter/Space. |
| Combobox | `ui/combobox.tsx` | `fret_components_shadcn::combobox::*` | ✅ | Currently a thin re-export. |
| Command | `ui/command.tsx` | `fret_components_ui::command_*` | ✅ | API shape differs; focus on behavior parity (cmdk-style). |
| Context Menu | `ui/context-menu.tsx` | `fret_components_shadcn::context_menu::*` | ✅ | Currently a thin re-export. |
| Dialog | `ui/dialog.tsx` | `fret_components_ui::dialog_*` | ✅ | Overlay/service model differs; ensure dismissal + inert background. |
| Drawer | `ui/drawer.tsx` | `fret_components_shadcn::open_drawer` | ✅ | Compatibility alias: mapped to a bottom sheet (no drag gestures yet). |
| Dropdown Menu | `ui/dropdown-menu.tsx` | `fret_components_ui::dropdown_menu::*` | ✅ | Ensure keyboard navigation parity. |
| Empty | `ui/empty.tsx` | `fret_components_shadcn::Empty` | ✅ | First-pass composition helper; add icon/action slots later. |
| Field | `ui/field.tsx` | `fret_components_shadcn::Field*` | ✅ | First pass: declarative composition helpers (no HTML `for`/aria wiring yet). |
| Form | `ui/form.tsx` |  | — | Non-goal (v1); likely a higher-level “form framework” decision in Fret. |
| Hover Card | `ui/hover-card.tsx` | `fret_components_shadcn::{HoverCard, HoverCardTrigger, HoverCardContent}` | ✅ | Declarative hover primitive (no open/close animations yet). |
| Input | `ui/input.tsx` | `fret_components_shadcn::Input` | ✅ | Currently maps to `TextField`; use `InputGroup` for icon-slot patterns. |
| Input Group | `ui/input-group.tsx` | `fret_components_shadcn::InputGroup` | ✅ | Leading/trailing icon slots + sizing tokens; first-pass parity. |
| Input OTP | `ui/input-otp.tsx` | `fret_components_shadcn::InputOTP` | ✅ | Retained widget: multi-slot input + paste fill + per-slot focus ring. |
| Item | `ui/item.tsx` | `fret_components_shadcn::Item*` | ✅ | First pass: generic “item row” with media/content/actions slots. |
| Kbd | `ui/kbd.tsx` | `fret_components_shadcn::Kbd` | ✅ | First-pass composition helper. |
| Label | `ui/label.tsx` | `fret_components_shadcn::Label` | ✅ | First-pass helper; semantics binding is a follow-up. |
| Menubar | `ui/menubar.tsx` | `fret_components_shadcn::menubar::*` | ✅ | Currently a thin re-export of `AppMenuBar`. |
| Native Select | `ui/native-select.tsx` |  | — | Non-goal (v1) (Fret is not HTML). |
| Navigation Menu | `ui/navigation-menu.tsx` | `fret_components_shadcn::{NavigationMenu, NavigationMenuItem, NavigationMenuLink}` | ✅ | Prototype: uses `Popover` overlay for list-like content (rich panels TBD). |
| Pagination | `ui/pagination.tsx` | `fret_components_shadcn::Pagination*` | ✅ | Taxonomy + first-pass recipe parity; uses Pressable + focus-visible ring. |
| Popover | `ui/popover.tsx` | `fret_components_ui::popover::*` | ✅ | Ensure placement + dismissal parity. |
| Progress | `ui/progress.tsx` | `fret_components_shadcn::progress::*` | ✅ | Currently `Progress` is a thin alias of `ProgressBar`. |
| Radio Group | `ui/radio-group.tsx` | `fret_components_shadcn::RadioGroup` | ✅ | First pass: single-choice selection model + keyboard arrows + focus-visible ring. |
| Resizable | `ui/resizable.tsx` | `fret_components_shadcn::resizable::*` | ✅ | Currently a thin re-export. |
| Scroll Area | `ui/scroll-area.tsx` | `fret_components_shadcn::scroll_area::*` | ✅ | Currently a thin re-export. |
| Select | `ui/select.tsx` | `fret_components_shadcn::select::*` | ✅ | Ensure focus + typeahead parity. |
| Separator | `ui/separator.tsx` | `fret_components_shadcn::Separator` | ✅ | Mostly styling. |
| Sheet | `ui/sheet.tsx` | `fret_components_shadcn::sheet::*` | ✅ | Modal overlay shell + app-provided content under `WindowOverlays`. |
| Sidebar | `ui/sidebar.tsx` | `fret_components_shadcn::{Sidebar, SidebarHeader, SidebarContent, SidebarFooter, SidebarGroup, SidebarGroupLabel, SidebarMenu, SidebarMenuItem, SidebarMenuButton}` | ✅ | Declarative V1: collapsible icon mode + grouped menu buttons (mobile offcanvas TBD). |
| Skeleton | `ui/skeleton.tsx` | `fret_components_shadcn::Skeleton` | ✅ | `bg-accent + rounded-md + animate-pulse` (frame-driven). |
| Slider | `ui/slider.tsx` | `fret_components_shadcn::slider::*` | ✅ | Currently a thin re-export. |
| Sonner | `ui/sonner.tsx` | `fret_components_shadcn::sonner::*` | ✅ | Thin re-export; treat as primary toast ref. |
| Spinner | `ui/spinner.tsx` | `fret_components_shadcn::Spinner` | ✅ | Dot-ring (frame-driven) until transforms land. |
| Switch | `ui/switch.tsx` | `fret_components_shadcn::switch::*` | ✅ | Validate keyboard toggle + focus ring. |
| Table | `ui/table.tsx` | `fret_components_shadcn::{Table, TableHeader, TableBody, TableFooter, TableRow, TableHead, TableCell, TableCaption}` | ✅ | Declarative composition facade; no horizontal scroll primitive yet (wrap in `ScrollArea` if needed). |
| Tabs | `ui/tabs.tsx` | `fret_components_shadcn::tabs::*` | ✅ | Ensure keyboard navigation parity. |
| Textarea | `ui/textarea.tsx` | `fret_components_shadcn::Textarea` | ✅ | Needs multiline editing parity over time. |
| Toggle | `ui/toggle.tsx` | `fret_components_shadcn::Toggle` | ✅ | Two-state button; variants/sizes should be validated. |
| Toggle Group | `ui/toggle-group.tsx` | `fret_components_shadcn::ToggleGroup` | ✅ | First pass: single/multiple selection models, spacing=0 border/radius merging, arrows + Enter/Space activation. |
| Tooltip | `ui/tooltip.tsx` | `fret_components_ui::tooltip::*` | ✅ | Ensure hover delays + focus behavior parity. |

## TODOs (prioritized)

### P0 — make the shadcn facade “complete for what we already ship”

- [x] Add facade modules for existing `fret-components-ui` surfaces:
  - `menubar` → `AppMenuBar`
  - `context_menu` → `ContextMenu`
  - `combobox` → `Combobox`
  - `scroll_area` → `ScrollArea`
  - `progress` → `ProgressBar`/`Progress` (whatever naming we choose)
  - `slider` → `Slider`
  - `sonner` → `Sonner`/`Toaster` surface
  - `resizable` → `ResizablePanelGroup`
- [x] Decide and document shadcn-facing naming for overlay/service primitives:
  - what maps to `Dialog` vs `AlertDialog` vs `Sheet`
  - what maps to `Command` vs `CommandDialog`-like flows
- [x] Add a “parity test plan” section per P0 overlay component (manual checklist):
  - focus trapping / inert background, escape dismissal, click-outside, keyboard navigation.

#### P0 Manual Acceptance Checklist (overlay + focus semantics)

Treat these as "behavioral contracts" for shadcn alignment. Prefer turning each bullet into:

- a small unit test when feasible (model-only logic), or
- a deterministic demo scenario in `fret-demo --bin ui_kit`, with a repeatable repro script.

**Dialog / AlertDialog (modal, focus restore)**

- Open moves focus into the dialog (first focusable action by default).
- `Escape` closes (uses `cancel_command` for `Dialog`, none for `AlertDialog` by default).
- Clicking outside closes (same semantics as `Escape`).
- `Enter` activates the default action (if enabled).
- While open, pointer/keyboard does not interact with the underlying UI (modal barrier).
- Close restores focus to the previously focused widget (including nested open/close sequences).

**Sheet / Drawer (modal-ish panel, focus restore)**

- Open moves focus into the sheet when `request_focus=true`.
- `Escape` closes when `close_on_escape=true`.
- Clicking outside closes when `close_on_click_outside=true`.
- Underlay is inert while open (modal barrier semantics).
- Close restores focus to the previously focused widget.

**Popover / DropdownMenu / ContextMenu (non-modal)**

- Open does not block the rest of the UI except where the popover is hit-testable.
- Clicking outside closes (menu-like dismissal).
- `Escape` closes and restores focus to trigger/previous focus (depending on component policy).
- Keyboard navigation within the menu list is consistent (Up/Down, Home/End, typeahead if present).

**Command (cmdk-like)**

- Open focuses the command input (or first focusable descendant).
- `Escape` closes and restores focus.
- Arrow navigation is stable and does not depend on hover.
- Filtering does not break selection/highlighting.

**Tooltip / HoverCard (non-modal, pointer semantics)**

- Tooltip shows on hover (with delay) and hides on move-out.
- Tooltip does not steal focus and does not block pointer to underlying widgets unless hit-testable.

#### Overlay Naming Contract (v1)

This is the shadcn-facing naming we converge on, even if the implementation is service/layer based
instead of Radix-style subcomponents.

- `Dialog`: `fret_components_shadcn::dialog::*` + `fret_components_shadcn::WindowOverlays`
  - Open path: `fret_components_shadcn::dialog::open_dialog(...)` sets a `DialogRequest` and
    dispatches `dialog.open`.
- `AlertDialog`: not a separate implementation yet.
  - Short-term mapping: reuse `Dialog` overlay but treat “destructive confirm” as policy on top of
    `DialogRequest` (e.g. action labeling, default/cancel action choice).
- `Popover`: `fret_components_shadcn::popover::*` (service-driven, anchored placement).
- `DropdownMenu`: `fret_components_shadcn::dropdown_menu::*` (menu model + focus/keyboard policy).
- `ContextMenu`: `fret_components_shadcn::context_menu::*` (right-click/open-at-point menu policy).
- `Tooltip`: `fret_components_shadcn::tooltip::*` (hover/focus-driven overlay).
- `Toaster` / `Sonner`: `fret_components_shadcn::sonner::*` + `fret_components_shadcn::toast::*`
  - Note: prefer `Sonner` as the upstream reference (see pinned repo note above).
- `CommandDialog`: modeled as a modal overlay + composable content.
  - Implementation: `fret_components_shadcn::command::CommandPaletteOverlay` + composable list
    content (`fret_components_shadcn::command::command_palette_list`).
  - Open/close path: dispatch `command_palette.open` / `command_palette.close` (managed by
    `WindowOverlays` focus restoration).
- `Sheet` / `Drawer`: not implemented yet.

#### Overlay Parity Test Plan (manual)

These checklists are the acceptance criteria for “interaction parity”.

- Dialog (`DialogOverlay` + `DialogService`)
  - Escape closes.
  - Click outside closes (if allowed by request/policy).
  - Focus is trapped inside while open; focus restores on close.
  - Default action activates on Enter; cancel on Escape.
  - Background is visually dimmed and inert to pointer/keyboard.
- CommandDialog (`CommandPaletteOverlay`)
  - Escape closes; click outside closes; focus restores on close.
  - Typing updates results; Up/Down navigates; Enter activates selection.
  - Scroll keeps selection visible; list does not allocate O(N) widgets.
- Popover (`PopoverService` + `Popover`)
  - Anchor positioning is stable across relayout; placement respects screen bounds.
  - Click outside dismisses; Escape dismisses; focus behavior matches shadcn expectation.
- DropdownMenu / ContextMenu
  - Keyboard navigation (Up/Down/Home/End) works; Enter activates; Escape closes.
  - Submenu opens/closes via Right/Left (or platform-appropriate mapping).
  - Pointer hover updates the active item without requiring extra clicks.
- Tooltip
  - Shows on hover (with delay); hides on move-out; also works via focus.
  - Does not steal focus; does not block pointer to underlying widgets.
- Sonner/Toasts
  - Toaster stack layout is stable; pointer hits only inside toast bounds.
  - Timers dismiss; action button commands dispatch; Escape behavior is consistent.

### P1 — implement missing “core app UI” components

- [x] `Alert`, `Badge`, `Card`, `Empty`, `Kbd`, `Label` (mostly styling + semantics)
  - Status: first-pass declarative helpers exist in `crates/fret-components-shadcn/src`:
    `alert.rs`, `badge.rs`, `card.rs`, `empty.rs`, `kbd.rs`, `label.rs`.
- [x] `Accordion` / `Collapsible` (expansion model + keyboard)
- [x] `InputGroup` (icon slots + per-edge padding primitives; depends on Tailwind primitive parity)
- [x] `RadioGroup` (selection model + keyboard arrows)
- [x] `ToggleGroup` (selection policy, spacing/border merging, keyboard)

### P2 — larger surfaces (likely require dedicated contracts)

- [x] `Table` (basic facade + styling)
- [x] `DataTable` (virtualized table V1: fixed header + fixed row height; not DOM semantics)
- [x] `Sidebar` / `NavigationMenu` (V1; navigation contracts + tree patterns still evolving)
- [x] `Calendar` (date model + navigation)
- [x] `DatePicker` (popover integration; input parsing is a follow-up)
