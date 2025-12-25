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

- Fret repo HEAD: `aed379e6ec8fbc9bc47bf7468ade892ea58f8355`
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
- Re-exports (thin aliases) for: `input`, `textarea`, `separator`, `checkbox`, `switch`, `tabs`,
  `select`, `tooltip`, `toast`, `popover`, `dialog`, `dropdown_menu`, `command`
- Infrastructure re-exports: `ChromeRefinement`, `LayoutRefinement`, `Space`, `Radius`, `Size`, `StyledExt`

Missing from the shadcn facade (but implemented in `fret-components-ui` today):

- `menubar` (likely maps to `AppMenuBar`)
- `context-menu` (maps to `ContextMenu`)
- `combobox` (maps to `Combobox`)
- `scroll-area`, `progress`, `slider`, `sonner`, `resizable`
- Several infra helpers (`WindowOverlays`, `DialogOverlay`, `CommandPaletteOverlay`) that may need
  shadcn-facing names.

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

| shadcn v4 component | Upstream ref | Fret mapping (today) | Status | Notes |
| --- | --- | --- | --- | --- |
| Accordion | `ui/accordion.tsx` |  | ❌ | Needs disclosure/animation + keyboard rules. |
| Alert | `ui/alert.tsx` |  | ❌ | Likely pure composition + tokens. |
| Alert Dialog | `ui/alert-dialog.tsx` |  | ❌ | Map to dialog overlay with “destructive confirm” policy. |
| Aspect Ratio | `ui/aspect-ratio.tsx` |  | ❌ | Depends on Tailwind layout primitives + image/media element. |
| Avatar | `ui/avatar.tsx` |  | ❌ | Needs image + fallback initials/icon. |
| Badge | `ui/badge.tsx` |  | ❌ | Pure styling + variants. |
| Breadcrumb | `ui/breadcrumb.tsx` |  | ❌ | Needs link-like text + separators. |
| Button | `ui/button.tsx` | `fret_components_shadcn::Button` | ✅ | Wrapper exists; verify hover/active/disabled parity. |
| Button Group | `ui/button-group.tsx` |  | ❌ | Requires consistent border/radius merging rules. |
| Calendar | `ui/calendar.tsx` |  | ❌ | Likely needs date model + navigation + selection. |
| Card | `ui/card.tsx` |  | ❌ | Surface + slots (header/content/footer). |
| Carousel | `ui/carousel.tsx` |  | ❌ | Not a near-term priority for editor UIs. |
| Chart | `ui/chart.tsx` |  | ❌ | Not a near-term priority; depends on a charting substrate. |
| Checkbox | `ui/checkbox.tsx` | `fret_components_shadcn::checkbox::*` | ✅ | Validate focus-visible + keyboard toggle. |
| Collapsible | `ui/collapsible.tsx` |  | ❌ | Similar to accordion but single section. |
| Combobox | `ui/combobox.tsx` | `fret_components_ui::Combobox` | ◐ | Add shadcn facade module + naming parity. |
| Command | `ui/command.tsx` | `fret_components_ui::command_*` | ✅ | API shape differs; focus on behavior parity (cmdk-style). |
| Context Menu | `ui/context-menu.tsx` | `fret_components_ui::ContextMenu` | ◐ | Add shadcn facade module + naming parity. |
| Dialog | `ui/dialog.tsx` | `fret_components_ui::dialog_*` | ✅ | Overlay/service model differs; ensure dismissal + inert background. |
| Drawer | `ui/drawer.tsx` |  | ❌ | Could map to “sheet”/panel overlay later. |
| Dropdown Menu | `ui/dropdown-menu.tsx` | `fret_components_ui::dropdown_menu::*` | ✅ | Ensure keyboard navigation parity. |
| Empty | `ui/empty.tsx` |  | ❌ | Pure composition helper (icon + title + description + actions). |
| Field | `ui/field.tsx` |  | ❌ | Needs label/control/help-text composition patterns. |
| Form | `ui/form.tsx` |  | ❌ | Likely a higher-level “form framework” decision in Fret. |
| Hover Card | `ui/hover-card.tsx` |  | ❌ | Can likely map to tooltip/popover hybrid. |
| Input | `ui/input.tsx` | `fret_components_shadcn::Input` | ✅ | Currently maps to `TextField`; missing “input-group” patterns. |
| Input Group | `ui/input-group.tsx` |  | ❌ | Depends on per-edge padding + icon slots (Tailwind primitives). |
| Input OTP | `ui/input-otp.tsx` |  | ❌ | Needs multi-cell input + paste semantics. |
| Item | `ui/item.tsx` |  | ❌ | Upstream is a generic “item row” primitive; define Fret equivalent. |
| Kbd | `ui/kbd.tsx` |  | ❌ | Likely simple surface + mono text + border/radius. |
| Label | `ui/label.tsx` |  | ❌ | Needs semantics binding to controls (ADR 0033 follow-up). |
| Menubar | `ui/menubar.tsx` | `fret_components_ui::AppMenuBar` | ◐ | Add shadcn facade + ensure desktop-menu UX. |
| Native Select | `ui/native-select.tsx` |  | ❌ | Probably non-goal (Fret is not HTML). |
| Navigation Menu | `ui/navigation-menu.tsx` |  | ❌ | Likely later; depends on menu + focus routing. |
| Pagination | `ui/pagination.tsx` |  | ❌ | Not critical for editor UIs; can be added later. |
| Popover | `ui/popover.tsx` | `fret_components_ui::popover::*` | ✅ | Ensure placement + dismissal parity. |
| Progress | `ui/progress.tsx` | `fret_components_ui::progress::*` | ◐ | Add shadcn facade module. |
| Radio Group | `ui/radio-group.tsx` |  | ❌ | Needs single-choice selection model + keyboard arrows. |
| Resizable | `ui/resizable.tsx` | `fret_components_ui::ResizablePanelGroup` | ◐ | Add shadcn facade module + API naming parity. |
| Scroll Area | `ui/scroll-area.tsx` | `fret_components_ui::scroll_area::*` | ◐ | Add shadcn facade module + style parity. |
| Select | `ui/select.tsx` | `fret_components_shadcn::select::*` | ✅ | Ensure focus + typeahead parity. |
| Separator | `ui/separator.tsx` | `fret_components_shadcn::Separator` | ✅ | Mostly styling. |
| Sheet | `ui/sheet.tsx` |  | ❌ | Candidate to implement via overlay + dock/panel semantics. |
| Sidebar | `ui/sidebar.tsx` |  | ❌ | Big surface; likely later after nav/tree contracts stabilize. |
| Skeleton | `ui/skeleton.tsx` |  | ❌ | Pure composition + animation; optional. |
| Slider | `ui/slider.tsx` | `fret_components_ui::slider::*` | ◐ | Add shadcn facade module + keyboard/drag parity. |
| Sonner | `ui/sonner.tsx` | `fret_components_ui::sonner::*` | ◐ | Add shadcn facade module; treat as primary toast ref. |
| Spinner | `ui/spinner.tsx` |  | ❌ | Optional; pure drawing/animation. |
| Switch | `ui/switch.tsx` | `fret_components_shadcn::switch::*` | ✅ | Validate keyboard toggle + focus ring. |
| Table | `ui/table.tsx` |  | ❌ | Needs virtualized table contract (not DOM table). |
| Tabs | `ui/tabs.tsx` | `fret_components_shadcn::tabs::*` | ✅ | Ensure keyboard navigation parity. |
| Textarea | `ui/textarea.tsx` | `fret_components_shadcn::Textarea` | ✅ | Needs multiline editing parity over time. |
| Toggle | `ui/toggle.tsx` |  | ❌ | Two-state button; can reuse button primitives + pressed state. |
| Toggle Group | `ui/toggle-group.tsx` |  | ❌ | Multi-toggle with single/multiple selection modes. |
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
- [ ] Decide and document shadcn-facing naming for overlay/service primitives:
  - what maps to `Dialog` vs `AlertDialog` vs `Sheet`
  - what maps to `Command` vs `CommandDialog`-like flows
- [ ] Add a “parity test plan” section per P0 overlay component (manual checklist):
  - focus trapping / inert background, escape dismissal, click-outside, keyboard navigation.

### P1 — implement missing “core app UI” components

- [x] `Alert`, `Badge`, `Card`, `Empty`, `Kbd`, `Label` (mostly styling + semantics)
  - Status: first-pass declarative helpers exist in `crates/fret-components-shadcn/src`:
    `alert.rs`, `badge.rs`, `card.rs`, `empty.rs`, `kbd.rs`, `label.rs`.
- [ ] `Accordion` / `Collapsible` (expansion model + keyboard)
- [ ] `InputGroup` (icon slots + per-edge padding primitives; depends on Tailwind primitive parity)
- [ ] `RadioGroup` (selection model + keyboard arrows)
- [ ] `Toggle` / `ToggleGroup` (pressed state + selection policy)

### P2 — larger surfaces (likely require dedicated contracts)

- [ ] `Table` / `DataTable` (virtualized table, resizing, selection; not DOM semantics)
- [ ] `Sidebar` / `NavigationMenu` (navigation contracts + tree patterns)
- [ ] `Calendar` / `DatePicker` (date model + navigation; optional for editor MVP)
