# shadcn/ui v4 Audit - Drawer (new-york)

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Drawer` surface against the upstream shadcn/ui v4
registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/base/drawer.mdx`
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
- Pass: `Drawer::direction(...)` now aliases the placement surface to match the upstream
  `direction` prop, while `side(...)` remains as a compatibility escape hatch.
- Pass: `Drawer::disable_pointer_dismissal(...)` is now available as the Base UI-style alias for
  `overlay_closable(false)`, so the root dismissal policy can be taught without inventing another
  mechanism seam.
- Pass: `Drawer::modal(false)` / `Drawer::modal_mode(DrawerModalMode::NonModal)` now expose the
  Base UI-style non-modal follow-up on the recipe root.
- Pass: `Drawer::modal_trap_focus(true)` / `Drawer::modal_mode(DrawerModalMode::TrapFocus)` now
  expose the Base UI-style trap-focus follow-up on the recipe root without widening the mechanism
  contract.
- Pass: `DrawerTrigger` exists as a thin passthrough wrapper for taxonomy parity.
- Pass: `DrawerPortal` is exposed for taxonomy parity (portal mounting is owned by the overlay
  manager in Fret).
- Pass: `DrawerOverlay` is exposed as a shadcn-named configuration surface (delegates to `Sheet`
  overlay defaults).
- Pass: `DrawerClose` is available and delegates to `DialogClose` (modal-overlay backed close).
- Pass: `DrawerClose::from_scope()` is available as recipe-layer sugar for content-local close
  buttons while preserving `DrawerClose::new(open)` as the explicit constructor.
- Pass: `DrawerClose::from_scope().build(cx, child)` supports composable child-close authoring,
  which is the Fret-side approximation of upstream `DrawerClose asChild`.
- Pass: `Drawer::children([DrawerPart::trigger(...), DrawerPart::content(...)])` is available as
  the closest recipe-level equivalent to upstream nested children composition while still lowering
  into the existing trigger/content slots.
- Pass: `Drawer::compose()` provides a recipe-level builder for part assembly without pushing
  shadcn-specific composition concerns into the lower-level mechanism contract.
- Pass: The default first-party copyable root path is
  `Drawer::new_controllable(cx, None, false).children([DrawerPart::trigger(...), DrawerPart::content_with(...)])`,
  while managed-open ownership remains explicit on `Drawer::new(open)` / `new_controllable(...)`.
- Pass: `DrawerContent` / `DrawerHeader` / `DrawerFooter` provide Drawer-specific layout while
  reusing shared dialog substrate building blocks (`Title` / `Description`).
- Note: Root authoring still lowers through recipe-layer deferred parts rather than true JSX-style
  nesting, but the default curated surface now matches the upstream mental model more closely via
  `children([...])`.

### Placement & sizing

- Pass: Bottom/top drawers apply `mt-24` / `mb-24`-style edge gaps and cap height to `max-h-[80vh]`
  when using auto-height content.
- Pass: Left/right drawers use `w-3/4` with an `sm:max-w-sm`-style cap (75% viewport width, capped
  at 384px).
- Pass: Bottom drawers include the small handle affordance region above the content.
- Pass: Vaul-style snap points are modeled for bottom drawers via `Drawer::snap_points(...)`.
- Pass: The first-party `Snap Points` recipe now stays on that same typed `compose()` root lane
  rather than falling back to the older closure-root `into_element(...)` path just to demonstrate
  the Vaul/Fret policy extension.

### Dismissal behavior

- Pass: Escape dismiss is handled by the shared dismissible root (Radix-aligned outcome).
- Pass: Overlay click-to-dismiss is implemented by rendering a full-window barrier behind the
  content (default on).
- Pass: Base UI-style `disablePointerDismissal` intent is now represented directly by
  `Drawer::disable_pointer_dismissal(...)`, which forwards to the existing sheet dismissal policy.
- Pass: Default overlay color matches the upstream `bg-black/50` intent (via the shared `Sheet`
  overlay defaults).
- Pass: Dismissals can be intercepted (Radix `DismissableLayer` "preventDefault" outcome) via
  `Drawer::on_dismiss_request(...)` (delegates to `Sheet`).
- Pass: Open lifecycle callbacks are available via `Drawer::on_open_change(...)` and
  `Drawer::on_open_change_complete(...)` (delegates to `Sheet`).
- Pass: Bottom drawers support Vaul-style drag-to-dismiss from the handle affordance region.
- Pass: Base UI-style `TrapFocus` follow-up now traps Tab focus inside the drawer while keeping
  outside pointer interaction enabled (`Drawer::modal_trap_focus(true)`).

### Focus behavior

- Pass: Modal barrier scoping prevents underlay focus traversal (ADR 0068).
- Pass: Focus restore on close is deterministic to the trigger (modal close unmount path).
- Pass: Base UI-style non-modal / trap-focus follow-up now defaults initial focus to the drawer
  popup root instead of the first focusable descendant.
- Pass: Base UI-style trap-focus follow-up keeps Tab traversal within the drawer subtree while
  remaining click-through to the underlay.

## Known gaps / intentional differences

- Base UI `modal={false | 'trap-focus'}` is now available, but the non-modal/trap-focus follow-up
  currently omits the modal scrim. In Fret, the current auto-overlay styling path is still
  modal-oriented, so `DrawerOverlay` / `overlay_color(...)` only affect the modal barrier path.
- Base UI / Vaul controlled snap-point surfaces are still narrower in Fret. Today we expose
  recipe-owned `snap_points(...)` plus `default_snap_point(index)`, but not
  `snapPoint` / `onSnapPointChange` / `snapToSequentialPoints`.
- Base UI nested-drawer coordination is not modeled yet. Upstream tracks nested swipe progress,
  frontmost height, and nested-open state for features like background indentation and gesture
  arbitration; Fret currently treats each drawer as an independent modal overlay.
- Vaul drag physics (rubber-banding, velocity, snap decisions) are not modeled yet; Fret currently
  uses a simpler threshold/inertia-based settle-and-dismiss policy.

## Validation

- `cargo check -p fret-ui-shadcn`
- `cargo nextest run -p fret-ui-shadcn drawer::tests`
- `cargo nextest run -p fret-ui-shadcn drawer_open_change_handlers_forward_to_sheet`

## Authoring note: `children([...])` and `compose()`

`Drawer::children([...])` is now the default recipe-layer bridge for authors who want the closest
equivalent to upstream nested parts without widening the mechanism contract.

- Scope: ergonomics only; it lowers into `Drawer::into_element_parts(...)`.
- Default teaching path: first-party examples now prefer
  `Drawer::new_controllable(cx, None, false).children([DrawerPart::trigger(...), DrawerPart::content_with(...)])`.
- `Drawer::compose()` remains the builder-first alternative when explicit trigger/content chaining
  reads better at the call site.
- Follow-up policy lane: Vaul-oriented `snap_points(...)` / `default_snap_point(...)` remain
  explicit recipe policy on that same root surface rather than a separate authoring seam.
- Layering: it does **not** change the underlying overlay/focus/dismiss mechanism.
- Limitation: this is still not a full React-style nested children API; Fret stores deferred parts
  and assembles them at the final call site.
