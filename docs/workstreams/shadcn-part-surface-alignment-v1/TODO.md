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

## Tracker table

| Component | Upstream parts (radix base) | Fret surface today | Gaps / drift | Footgun risk | Proposed refactor / additions | Gate | Priority | Status |
|---|---|---|---|---:|---|---|---:|---|
| `card` | `Card, CardHeader, CardTitle, CardDescription, CardAction, CardContent, CardFooter` (+ `size`) | `Card*` parts + `CardSize` + `card_sized` | `CardAction` styling override was missing; `size=sm` spacing drift | Low | Keep part surface; lock `size=sm` spacing + add safe builders (already landed) | unit tests in `ecosystem/fret-ui-shadcn/src/card.rs` | P0 | Done |
| `avatar` | `Avatar, AvatarImage, AvatarFallback` | `Avatar*` (+ `AvatarBadge`, `AvatarGroup*`) + `avatar_sized(...)` | extra parts exist (fine) | **Yes** (size scope) | Keep extra parts; add scoped builder + explicit `size(...)` overrides for size-dependent leaf parts | unit tests in `ecosystem/fret-ui-shadcn/src/avatar.rs` | P0 | Done |
| `dialog` | `Dialog, DialogTrigger, DialogPortal, DialogOverlay, DialogContent, DialogClose, DialogHeader, DialogFooter, DialogTitle, DialogDescription` | `Dialog, DialogContent, DialogClose, DialogHeader, DialogFooter, DialogTitle, DialogDescription` (+ thin parts adapters) | Portal/overlay/trigger are adapters; trigger still requires a caller-authored toggle | Medium | Add thin parts that delegate to existing `Dialog` overlay composition; preserve current closure API | diag script `ui-gallery-dialog-part-surface-smoke` | P0 | Done (with known gaps) |
| `alert-dialog` | `AlertDialog, AlertDialogTrigger, AlertDialogPortal, AlertDialogOverlay, AlertDialogContent, AlertDialogCancel, AlertDialogAction, AlertDialogTitle, AlertDialogDescription, AlertDialogHeader, AlertDialogFooter` | `AlertDialog` closure API + `AlertDialogTrigger/Portal/Overlay` adapters + existing parts (+ `AlertDialogMedia`) | Portal is still implicit in the overlay controller; trigger remains caller-authored (toggle) | Medium | Add thin parts for shadcn-like call sites + demo + gate | diag script `ui-gallery-alert-dialog-part-surface-smoke` | P0 | Done (with known gaps) |
| `sheet` | `Sheet, SheetTrigger, SheetPortal, SheetOverlay, SheetContent, SheetClose, SheetHeader, SheetFooter, SheetTitle, SheetDescription` | `Sheet` closure API + `SheetTrigger/Portal/Overlay` adapters + `SheetContent/Close/Header/Footer/Title/Description` | Portal is still implicit in the overlay controller; trigger remains caller-authored (toggle) | Medium | Keep closure API; add thin parts for shadcn-like call sites + demo + gate | diag script `ui-gallery-sheet-part-surface-smoke` | P0 | Done (with known gaps) |
| `drawer` | `Drawer, DrawerTrigger, DrawerPortal, DrawerOverlay, DrawerContent, DrawerClose, DrawerTitle, DrawerDescription` | `Drawer*` parts + `DrawerTitle/DrawerDescription` re-export | Portal is implicit; trigger remains caller-authored (toggle) | Medium | Keep closure API; keep thin parts for shadcn-like call sites | diag script `ui-gallery-drawer-docs-smoke` | P1 | Done (with known gaps) |
| `dropdown-menu` | `DropdownMenu, Trigger, Portal, Content, Group, Item, CheckboxItem, RadioGroup, RadioItem, Label, Separator, Shortcut, Sub, SubTrigger, SubContent` | `DropdownMenu` + item/label/group/shortcut/etc (+ thin parts adapters) | n/a | Medium | Keep current impl; expose part adapters + add a demo + gate | diag script `ui-gallery-dropdown-menu-part-surface-smoke` | P0 | Done |
| `menubar` | `Menubar, Menu, Trigger, Portal, Content, Group, Item, CheckboxItem, RadioGroup, RadioItem, Label, Separator, Shortcut, Sub, SubTrigger, SubContent` | `Menubar` + `MenubarMenu*` + item/label/group/shortcut/etc (+ thin parts adapters) | Portal is still implicit; trigger/content are adapters | Medium | Add adapters: `MenubarTrigger/MenubarContent/MenubarSeparator/MenubarSub*` + demo + gate | diag script `ui-gallery-menubar-part-surface-smoke` | P0 | Done (with known gaps) |
| `navigation-menu` | many parts + a style helper `navigationMenuTriggerStyle` | `NavigationMenu*` parts (varies) | “style helper” is upstream-only | Low | Only port helper if it is a useful Fret authoring surface; otherwise ignore | n/a (doc-only) | P2 | Not started |
| `tabs` | parts + `tabsListVariants` style helper | `Tabs*` parts | style helper missing | Low | Same as navigation-menu: optional | n/a (doc-only) | P2 | Not started |
| `carousel` | `Carousel, CarouselContent, CarouselItem, CarouselPrevious, CarouselNext, useCarousel` | `Carousel` facade + part adapters (`CarouselContent/Item/Previous/Next`, `use_carousel`) | per-item basis/class surface remains Rust-native | Low-Med | Keep current engine; add part surface adapter + gallery demo + gate | diag script `ui-gallery-carousel-part-surface-smoke` | P1 | Done (with known gaps) |

## Notes / recurring hazards

### Provider footgun candidates (likely)

- `avatar`: size-dependent badge/count placement.
- overlay families that install “side” scopes: `sheet`, `drawer`.

Preferred mitigation: scoped builders first, explicit overrides second.
