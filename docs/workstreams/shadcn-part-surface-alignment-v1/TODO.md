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
| `avatar` | `Avatar, AvatarImage, AvatarFallback` | `Avatar*` (+ `AvatarBadge`, `AvatarGroup*`) | extra parts exist (fine); missing “scoped build” helper for size-dependent parts | **Yes** (size scope) | Add `avatar_sized(...)` and/or `AvatarBadge::size(...)` / `AvatarGroupCount::size(...)` | unit tests for inherited vs explicit size | P0 | Not started |
| `dialog` | `Dialog, DialogTrigger, DialogPortal, DialogOverlay, DialogContent, DialogClose, DialogHeader, DialogFooter, DialogTitle, DialogDescription` | `Dialog, DialogContent, DialogClose, DialogHeader, DialogFooter, DialogTitle, DialogDescription` | missing `DialogTrigger/DialogPortal/DialogOverlay` parts (API shape mismatch) | Medium | Add thin parts that delegate to existing `Dialog` overlay composition; preserve current closure API | diag script (open/close/focus/dismiss) + small unit gate | P0 | Not started |
| `alert-dialog` | `AlertDialog, AlertDialogTrigger, AlertDialogPortal, AlertDialogOverlay, AlertDialogContent, AlertDialogCancel, AlertDialogAction, AlertDialogTitle, AlertDialogDescription, AlertDialogHeader, AlertDialogFooter` | `AlertDialog, AlertDialogTrigger, AlertDialogContent, AlertDialogCancel, AlertDialogAction, AlertDialogTitle, AlertDialogDescription, AlertDialogHeader, AlertDialogFooter` (+ `AlertDialogMedia`) | missing `AlertDialogPortal/AlertDialogOverlay` parts | Medium | Introduce missing parts; ensure action/cancel composition stays parity-friendly | diag script (escape/outside press, cancel/action) | P0 | Not started |
| `sheet` | `Sheet, SheetTrigger, SheetPortal, SheetOverlay, SheetContent, SheetClose, SheetHeader, SheetFooter, SheetTitle, SheetDescription` | `Sheet` closure API + `SheetContent/Close/Header/Footer/Title/Description` | missing `SheetTrigger` and explicit `Portal/Overlay` parts | Medium | Add `SheetTrigger/Portal/Overlay` thin parts; keep closure API as convenience | diag script (open/close, side, close button) | P0 | Not started |
| `drawer` | `Drawer, DrawerTrigger, DrawerPortal, DrawerOverlay, DrawerContent, DrawerClose, DrawerTitle, DrawerDescription` | has `DrawerPortal/Overlay/Content/Trigger/Close/Header/Footer` | missing `DrawerTitle/DrawerDescription` (surface mismatch) | Low | Add `DrawerTitle/DrawerDescription` thin parts (likely reuse dialog text tokens) | unit test (text style + semantics role if applicable) | P1 | Not started |
| `dropdown-menu` | `DropdownMenu, Trigger, Portal, Content, Group, Item, CheckboxItem, RadioGroup, RadioItem, Label, Separator, Shortcut, Sub, SubTrigger, SubContent` | `DropdownMenu` + item/label/group/shortcut/etc (+ thin parts adapters) | n/a | Medium | Keep current impl; expose part adapters + add a demo + gate | diag script `ui-gallery-dropdown-menu-part-surface-smoke` | P0 | Done |
| `menubar` | `Menubar, Menu, Trigger, Portal, Content, Group, Item, CheckboxItem, RadioGroup, RadioItem, Label, Separator, Shortcut, Sub, SubTrigger, SubContent` | `Menubar` + `MenubarMenu*` + item/label/group/shortcut/etc (+ thin parts adapters) | Portal is still implicit; trigger/content are adapters | Medium | Add adapters: `MenubarTrigger/MenubarContent/MenubarSeparator/MenubarSub*` + demo + gate | diag script `ui-gallery-menubar-part-surface-smoke` | P0 | Done (with known gaps) |
| `navigation-menu` | many parts + a style helper `navigationMenuTriggerStyle` | `NavigationMenu*` parts (varies) | “style helper” is upstream-only | Low | Only port helper if it is a useful Fret authoring surface; otherwise ignore | n/a (doc-only) | P2 | Not started |
| `tabs` | parts + `tabsListVariants` style helper | `Tabs*` parts | style helper missing | Low | Same as navigation-menu: optional | n/a (doc-only) | P2 | Not started |
| `carousel` | `Carousel, CarouselContent, CarouselItem, CarouselPrevious, CarouselNext, useCarousel` | single `Carousel` facade + API models | part surface mismatch (API shape) | Low-Med | Add thin wrapper parts around current `Carousel` (or migrate to part-based authoring) | unit test for wrapper composition + one diag script for prev/next | P1 | Not started |

## Notes / recurring hazards

### Provider footgun candidates (likely)

- `avatar`: size-dependent badge/count placement.
- overlay families that install “side” scopes: `sheet`, `drawer`.

Preferred mitigation: scoped builders first, explicit overrides second.
