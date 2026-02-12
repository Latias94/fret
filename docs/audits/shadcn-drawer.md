# shadcn/ui v4 Audit - Drawer (new-york)

This audit compares Fret's shadcn-aligned `Drawer` surface against the upstream shadcn/ui v4
registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/drawer.tsx`
- Underlying primitive concept: `vaul` `Drawer` (a drawer-shaped modal surface built on top of a
  dialog-like primitive)

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/drawer.rs`
- Current mapping: Drawer is implemented on top of `Sheet` for overlay/presence/dismissal, with
  Drawer-specific content/header/footer layout to match shadcn/ui + Vaul styling intent.
- Overlay policy/infra:
  - Modal overlay roots: `ecosystem/fret-ui-kit/src/window_overlays/*`
  - Radix-aligned dialog substrate: `ecosystem/fret-ui-kit/src/primitives/dialog.rs`
  - shadcn sheet recipe: `ecosystem/fret-ui-shadcn/src/sheet.rs`

## What upstream exports (new-york)

Upstream exports a thin wrapper around `vaul`:

- `Drawer` (root)
- `DrawerTrigger`
- `DrawerPortal`
- `DrawerOverlay`
- `DrawerContent` (direction-aware sizing + optional handle)
- `DrawerClose`
- `DrawerHeader`
- `DrawerFooter`
- `DrawerTitle`
- `DrawerDescription`

## Audit checklist

### API & composition

- Pass: Fret provides `Drawer` as a recipe driven by a `Model<bool>` open state.
- Pass: Trigger/content composition matches the shadcn mental model.
- Pass: `DrawerTrigger` exists as a thin passthrough wrapper for taxonomy parity.
- Pass: `DrawerPortal` is exposed for taxonomy parity (portal mounting is owned by the overlay
  manager in Fret).
- Pass: `DrawerOverlay` is exposed as a shadcn-named configuration surface (delegates to `Sheet`
  overlay defaults).
- Pass: `DrawerClose` is available and delegates to `DialogClose` (modal-overlay backed close).
- Pass: `DrawerContent`/`Header`/`Footer` provide Drawer-specific layout while reusing shared dialog
  substrate building blocks (`Title`/`Description`).

### Placement & sizing

- Pass: Bottom/top drawers apply `mt-24`/`mb-24`-style edge gaps and cap height to `max-h-[80vh]`
  when using auto-height content.
- Pass: Left/right drawers use `w-3/4` with an `sm:max-w-sm`-style cap (75% viewport width, capped
  at 384px).
- Pass: Bottom drawers include the small "handle" affordance region above the content.

### Dismissal behavior

- Pass: Escape dismiss is handled by the shared dismissible root (Radix-aligned outcome).
- Pass: Overlay click-to-dismiss is implemented by rendering a full-window barrier behind the
  content (default on).
- Pass: Default overlay color matches the upstream `bg-black/50` intent (via the shared `Sheet`
  overlay defaults).
- Pass: Dismissals can be intercepted (Radix `DismissableLayer` "preventDefault" outcome) via
  `Drawer::on_dismiss_request(...)` (delegates to `Sheet`).
- Pass: Open lifecycle callbacks are available via `Drawer::on_open_change(...)` and
  `Drawer::on_open_change_complete(...)` (delegates to `Sheet`).
- Pass: Bottom drawers support Vaul-style drag-to-dismiss from a small handle affordance region.

### Focus behavior

- Pass: Modal barrier scoping prevents underlay focus traversal (ADR 0068).
- Pass: Focus restore on close is deterministic to the trigger (modal close unmount path).

## Known gaps / intentional differences

- Pass: Vaul-style snap points are modeled for bottom drawers via `Drawer::snap_points(...)`.
- Vaul drag physics (rubber-banding, velocity, snap decisions) are not modeled yet (Fret currently
  uses a simple threshold-based close).

## Validation

- `cargo check -p fret-ui-shadcn`
- `cargo nextest run -p fret-ui-shadcn drawer::tests`
- `cargo nextest run -p fret-ui-shadcn drawer_open_change_handlers_forward_to_sheet`
