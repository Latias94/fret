# shadcn Part Surface Alignment v1 (TODO + Tracker)

This tracker is **workstream-local**. It is not the global roadmap source of truth.

Last audit snapshot: **2026-03-01**.

## How to use this tracker

- “Parts” should match upstream shadcn/ui v4 *bases* exports under
  `repo-ref/ui/apps/v4/registry/bases/radix/ui/<component>.tsx`.
- “Footgun” means the part reads inherited scope (size/side/variant/open/dir) and can silently
  fall back when constructed outside the scope.
- “Gate” is required for every row we mark “Done”.

## Status legend

- `Not started`
- `In progress`
- `Done (with known gaps)`
- `Done`
- `Deferred (planned)`

## Tracker table

| Component | Upstream parts (radix base) | Fret surface today | Gaps / drift | Footgun risk | Proposed refactor / additions | Gate | Priority | Status |
|---|---|---|---|---:|---|---|---:|---|
| `card` | `Card, CardHeader, CardTitle, CardDescription, CardAction, CardContent, CardFooter` (+ `size`) | `Card*` parts + `CardSize` + `card_sized` | `CardAction` styling override was missing; `size=sm` spacing drift | Low | Keep part surface; lock `size=sm` spacing + add safe builders (already landed) | unit tests in `ecosystem/fret-ui-shadcn/src/card.rs` | P0 | Done |
| `direction` | `DirectionProvider, useDirection` | `DirectionProvider, useDirection` (+ `use_direction` alias) | none (alias provided for copy/paste parity) | Low | Keep as thin wrappers; treat this as the shadcn-aligned entrypoint | unit tests in `ecosystem/fret-ui-shadcn/src/direction.rs` | P2 | Done |
| `breadcrumb` | `Breadcrumb, BreadcrumbList, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator, BreadcrumbEllipsis` | `Breadcrumb` recipe builder + re-exported primitives (`BreadcrumbList/Link/Page/Ellipsis`) | Root part name conflicts (`Breadcrumb` recipe vs upstream `Breadcrumb` root part) | Low | Keep recipe as-is; keep primitives available for upstream-shaped composition | unit tests in `ecosystem/fret-ui-shadcn/src/breadcrumb.rs` | P2 | Done (with known gaps) |
| `avatar` | `Avatar, AvatarImage, AvatarFallback` | `Avatar*` (+ `AvatarBadge`, `AvatarGroup*`) + `avatar_sized(...)` | extra parts exist (fine) | **Yes** (size scope) | Keep extra parts; add scoped builder + explicit `size(...)` overrides for size-dependent leaf parts | unit tests in `ecosystem/fret-ui-shadcn/src/avatar.rs` | P0 | Done |
| `dialog` | `Dialog, DialogTrigger, DialogPortal, DialogOverlay, DialogContent, DialogClose, DialogHeader, DialogFooter, DialogTitle, DialogDescription` | `Dialog, DialogContent, DialogClose, DialogHeader, DialogFooter, DialogTitle, DialogDescription` (+ thin parts adapters) | Portal/overlay/trigger are adapters; trigger still requires a caller-authored toggle | Medium | Add thin parts that delegate to existing `Dialog` overlay composition; preserve current closure API | diag script `ui-gallery-dialog-part-surface-smoke` | P0 | Done (with known gaps) |
| `alert-dialog` | `AlertDialog, AlertDialogTrigger, AlertDialogPortal, AlertDialogOverlay, AlertDialogContent, AlertDialogCancel, AlertDialogAction, AlertDialogTitle, AlertDialogDescription, AlertDialogHeader, AlertDialogFooter` | `AlertDialog` closure API + `AlertDialogTrigger/Portal/Overlay` adapters + existing parts (+ `AlertDialogMedia`) | Portal is still implicit in the overlay controller; trigger remains caller-authored (toggle) | Medium | Add thin parts for shadcn-like call sites + demo + gate | diag script `ui-gallery-alert-dialog-part-surface-smoke` | P0 | Done (with known gaps) |
| `sheet` | `Sheet, SheetTrigger, SheetPortal, SheetOverlay, SheetContent, SheetClose, SheetHeader, SheetFooter, SheetTitle, SheetDescription` | `Sheet` closure API + `SheetTrigger/Portal/Overlay` adapters + `SheetContent/Close/Header/Footer/Title/Description` | Portal is still implicit in the overlay controller; trigger remains caller-authored (toggle) | Medium | Keep closure API; add thin parts for shadcn-like call sites + demo + gate | diag script `ui-gallery-sheet-part-surface-smoke` | P0 | Done (with known gaps) |
| `drawer` | `Drawer, DrawerTrigger, DrawerPortal, DrawerOverlay, DrawerContent, DrawerClose, DrawerTitle, DrawerDescription` | `Drawer*` parts incl `DrawerPortal/DrawerOverlay` + `DrawerTitle/DrawerDescription` re-export | Portal/overlay are adapters (overlay controller owns actual portal) | Medium | Keep closure API; keep thin parts for shadcn-like call sites | diag script `ui-gallery-drawer-docs-smoke` | P1 | Done (with known gaps) |
| `dropdown-menu` | `DropdownMenu, Trigger, Portal, Content, Group, Item, CheckboxItem, RadioGroup, RadioItem, Label, Separator, Shortcut, Sub, SubTrigger, SubContent` | `DropdownMenu` + parts adapters (+ `DropdownMenuPortal`) | Portal is a no-op wrapper (overlay already renders in an overlay root) | Medium | Keep current impl; keep thin part adapters for copy/paste parity | diag script `ui-gallery-dropdown-menu-part-surface-smoke` | P0 | Done |
| `menubar` | `Menubar, Menu, Trigger, Portal, Content, Group, Item, CheckboxItem, RadioGroup, RadioItem, Label, Separator, Shortcut, Sub, SubTrigger, SubContent` | `Menubar` + `MenubarMenu*` + parts adapters (+ `MenubarPortal`) | Portal is a no-op wrapper; trigger/content are adapters | Medium | Keep current impl; keep thin part adapters for copy/paste parity | diag script `ui-gallery-menubar-part-surface-smoke` | P0 | Done (with known gaps) |
| `navigation-menu` | many parts + a style helper `navigationMenuTriggerStyle` | `NavigationMenu*` parts + `NavigationMenuTriggerStyle` + `navigation_menu_trigger_style(...)` | helper only encodes base layout (interactive states still recipe-owned) | Low | Keep helper as a typed, mergeable refinement surface; reuse it for trigger/link base layout | unit tests in `ecosystem/fret-ui-shadcn/src/navigation_menu.rs` | P2 | Done (with known gaps) |
| `tabs` | parts + `tabsListVariants` style helper | `Tabs*` parts + `TabsListVariant` + `tabs_list_variants(...)` | line variant uses shared indicator line (approx) | Low | Provide typed variant surface + helper returning mergeable refinements | unit tests in `ecosystem/fret-ui-shadcn/src/tabs.rs` | P2 | Done (with known gaps) |
| `carousel` | `Carousel, CarouselContent, CarouselItem, CarouselPrevious, CarouselNext, useCarousel` | `Carousel` facade + part adapters (`CarouselContent/Item/Previous/Next`, `useCarousel` + `use_carousel` alias) | per-item basis/class surface remains Rust-native | Low-Med | Keep current engine; add part surface adapter + gallery demo + gate | diag script `ui-gallery-carousel-part-surface-smoke` | P1 | Done (with known gaps) |
| `chart` | `ChartContainer, ChartTooltip, ChartTooltipContent, ChartLegend, ChartLegendContent, ChartStyle` | `ChartContainer` + `ChartStyle` + stubs for `ChartTooltip/ChartLegend` + `ChartTooltipContent/ChartLegendContent` recipes | Not wired to the chart engine yet; CSS variable injection is not represented; icon mapping is not modeled | Medium | Keep recipes; add a thin context + layout wrapper so upstream doc shapes can be expressed | unit tests in `ecosystem/fret-ui-shadcn/src/chart.rs` | P2 | Done (with known gaps) |
| `popover` | `Popover, PopoverTrigger, PopoverContent, PopoverAnchor, PopoverHeader, PopoverTitle, PopoverDescription` | `Popover*` parts | `PopoverContent` encapsulates portal/presence; surface includes additional knobs (hover-open, modal mode) | Medium | Keep surface; lock default alignment + hover open/close events and focus behavior with unit tests | unit tests in `ecosystem/fret-ui-shadcn/src/popover.rs` | P1 | Done (with known gaps) |
| `tooltip` | `TooltipProvider, Tooltip, TooltipTrigger, TooltipContent` | `Tooltip*` parts (+ `TooltipAnchor`) | `Tooltip` includes policy knobs not present upstream; content slot defaults are modeled via `ShadcnSurfaceSlot` | Medium | Keep surface; lock provider delay semantics + content inherited defaults via unit tests | unit tests in `ecosystem/fret-ui-shadcn/src/tooltip.rs` | P1 | Done (with known gaps) |
| `hover-card` | `HoverCard, HoverCardTrigger, HoverCardContent` | `HoverCard*` parts (+ `HoverCardAnchor`) | Adds `HoverCardAnchor` and extra policy knobs (delays, safe corridor buffer) | Medium | Keep surface; lock hover intent lease/delays + placement defaults via unit tests | unit tests in `ecosystem/fret-ui-shadcn/src/hover_card.rs` | P2 | Done (with known gaps) |
| `select` | `Select, SelectTrigger, SelectValue, SelectContent, SelectGroup, SelectItem, SelectLabel, SelectSeparator, SelectScrollUpButton, SelectScrollDownButton` | `Select*` parts (config wrappers + entries) | Structural drift: Trigger/Value/Content are config wrappers, not literal nested elements (limits copy/paste parity) | High | Keep current surface for now; **defer** a true part-surface redesign to Milestone 6 (while keeping the existing unit tests as a behavior baseline) | unit tests in `ecosystem/fret-ui-shadcn/src/select.rs` | P0 | Done (with known gaps) |
| `combobox` | many parts (Base UI-rooted) | `Combobox` (Popover + Command recipe) + `ComboboxOption/OptionGroup` data model + v4 parts adapters (`into_element_parts`) | Still missing full Base UI surface (render-prop ergonomics, true in-trigger input; anchor uses element IDs instead of DOM refs) | High | Data model rename landed (`src/combobox_data.rs`) + placement knobs + v4-named parts (`ComboboxInput/Content/Empty/List/Item/Group/...`) and chips adapter (`ComboboxChips::into_element_parts`). Usage notes: `docs/workstreams/shadcn-part-surface-alignment-v1/COMBOBOX_V4_USAGE.md`. Next: tighten gates around copy/paste docs usage and extend chips/clear/anchor parity as needed. | unit tests in `ecosystem/fret-ui-shadcn/src/combobox.rs` and `ecosystem/fret-ui-shadcn/src/combobox_chips.rs` | P0 | Done (with known gaps) |
| `scroll-area` | `ScrollArea, ScrollBar` | `ScrollArea` + `ScrollArea*` primitives + `ScrollBar` alias | Previously only exposed `ScrollAreaScrollbar`; `ScrollBar` alias added for copy/paste parity | Low | Keep both names; treat `ScrollBar` as the docs-aligned spelling | unit tests in `ecosystem/fret-ui-shadcn/src/scroll_area.rs` | P2 | Done |
| `input-group` | `InputGroup, InputGroupAddon, InputGroupButton, InputGroupText, InputGroupInput, InputGroupTextarea` | `InputGroup` recipe + part adapters (`InputGroupAddon/InputGroupInput/InputGroupTextarea`) | Addon click-to-focus is not implemented (no selector engine; no DOM focus emulation) | Medium | Keep recipe; provide `into_element_parts` adapter for copy/paste parity | unit tests in `ecosystem/fret-ui-shadcn/src/input_group.rs` | P2 | Done (with known gaps) |
| `input-otp` | `InputOTP, InputOTPGroup, InputOTPSlot, InputOTPSeparator` | `InputOtp` recipe + parts adapters (`InputOTPGroup/Slot/Separator`) | Slot-level `aria-invalid` is global (`InputOtp::aria_invalid`), not per-slot; per-part refinements are not modeled | Low | Keep monolithic recipe; provide `into_element_parts` adapter for copy/paste parity | unit tests in `ecosystem/fret-ui-shadcn/src/input_otp.rs` | P2 | Done (with known gaps) |
| `empty` | `Empty, EmptyHeader, EmptyTitle, EmptyDescription, EmptyContent, EmptyMedia` | `Empty*` parts | Uses container queries instead of viewport breakpoints (intentional) | Low | Keep surface; lock padding breakpoint behavior via deterministic unit test | `ecosystem/fret-ui-shadcn/tests/empty_responsive_padding.rs` | P2 | Done |
| `context-menu` | `ContextMenu, Trigger, Portal, Content, Item, CheckboxItem, RadioItem, Label, Separator, Shortcut, Group, Sub, SubTrigger, SubContent, RadioGroup` | `ContextMenu` + `ContextMenuTrigger/Portal/Content` adapters + submenu helpers | Portal is a no-op wrapper; submenu parts are helpers over `ContextMenuItem::submenu(...)` | High | Keep closure API; expose part adapters + submenu helper parts; add diag script when UI gallery uses it | unit tests in `ecosystem/fret-ui-shadcn/src/context_menu.rs` | P0 | Done (with known gaps) |

## Notes / recurring hazards

### Provider footgun candidates (likely)

- `avatar`: size-dependent badge/count placement.
- overlay families that install “side” scopes: `sheet`, `drawer`.

Preferred mitigation: scoped builders first, explicit overrides second.

## Upstream exported style helpers (optional)

Upstream `bases/radix` exports a small number of Tailwind/CVA helpers. In Fret we only port these
when they are useful **authoring surfaces** (mergeable refinements) rather than implementation
details.

| Helper (upstream) | Where it comes from | Fret mapping | Status |
|---|---|---|---|
| `tabsListVariants({ variant })` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/tabs.tsx` | `TabsListVariant` + `tabs_list_variants(...)` | Done (with known gaps) |
| `navigationMenuTriggerStyle()` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/navigation-menu.tsx` | `NavigationMenuTriggerStyle` + `navigation_menu_trigger_style(...)` | Done (with known gaps) |
| `buttonVariants({ variant, size })` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/button.tsx` | Prefer `Button` + typed enums (`ButtonVariant`, `ButtonSize`) | Not planned (for now) |
| `badgeVariants({ variant })` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/badge.tsx` | Prefer `Badge` + typed enums (`BadgeVariant`) | Not planned (for now) |
| `toggleVariants({ variant, size })` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/toggle.tsx` | Prefer `Toggle` + typed enums (`ToggleVariant`, `ToggleSize`) | Not planned (for now) |
| `buttonGroupVariants({ orientation })` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/button-group.tsx` | Prefer `ButtonGroup` + `ButtonGroupOrientation` recipe surface | Not planned (for now) |
| `fieldVariants({ orientation })` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/field.tsx` | Prefer `Field` + `FieldOrientation` recipe surface | Not planned (for now) |
| `itemVariants({ variant, size, className })` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/item.tsx` | Prefer `Item` + `ItemStyle` / typed enums | Not planned (for now) |
| `itemMediaVariants({ variant })` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/item.tsx` | Prefer `ItemMediaVariant` enum on `ItemMedia` | Not planned (for now) |
