# shadcn/ui v4 Audit - Dropdown Menu

This audit compares Fret's shadcn-aligned `DropdownMenu` against the upstream shadcn/ui v4 docs and
examples in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/dropdown-menu.mdx`
- Reference implementation (Radix base): `repo-ref/ui/apps/v4/registry/bases/radix/ui/dropdown-menu.tsx`
- Reference examples: `repo-ref/ui/apps/v4/registry/bases/radix/examples/dropdown-menu-example.tsx`

Key upstream behaviors/surfaces:

- Rich content surface: groups, labels, separators, shortcuts, disabled items, destructive variant,
  checkbox items, radio items, submenus, and a portal-based content root.
- Content sizing: `align="start"`, `sideOffset=4`, width tracks trigger width, and max-height
  tracks available height with internal scroll.
- Keyboard: roving navigation + typeahead; focus styling is part of the visual parity.
- Dialog interop: examples mention `modal={false}` for opening dialogs from menus.

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`
- Overlay policy substrate: `ecosystem/fret-ui-kit/src/overlay_controller.rs`
- Dismissible popover policy: `ecosystem/fret-ui-kit/src/window_overlays/mod.rs`
- Roving + typeahead policy helpers: `ecosystem/fret-ui-kit/src/declarative/action_hooks.rs`

## Audit checklist

### Placement & sizing

- Pass: Per-window overlay root (portal-like), rendered via `OverlayController`.
- Pass: Anchored placement via the Radix-shaped popper facade (deterministic flip + clamp).
  - Placement policy: `fret_ui_kit::primitives::popper::PopperContentPlacement`
  - Solver: `crates/fret-ui/src/overlay_placement/solver.rs`
- Pass: `align="start"` respects the direction provider: under RTL, the content aligns to the
  trigger's logical start edge (Radix/Floating parity).
- Pass: Default `side_offset` aligns with upstream (`4`).
- Pass: Viewport size tracks trigger width (with a minimum) and clamps height to available space;
  internal scrolling is via `Scroll` (Y-axis).

### Dismissal & focus

- Pass: Dismissible popover (outside press + Escape) via `window_overlays`.
- Pass: On open, focus moves to the first focusable descendant (driven by overlay policy), enabling
  keyboard navigation inside the menu.
- Pass: `DropdownMenu::modal(bool)` is supported (default `true`).
  - `modal=true`: blocks underlay pointer interaction while open (Radix `disableOutsidePointerEvents`).
  - `modal=false`: outside-press dismissal becomes click-through.
- Note: Fret exposes an explicit `close_on_select` policy per item; upstream Radix typically relies
  on `onSelect(e) { e.preventDefault() }` to keep menus open for toggles.

### Keyboard navigation & typeahead

- Pass: `RovingFlex` + APG-style navigation (`roving_nav_apg`) are wired.
- Pass: Prefix typeahead with timeout is wired.

### Visual parity (shadcn)

- Partial: Menu background/foreground now align with popover tokens (`bg-popover text-popover-foreground`).
- Pass: Hover/pressed/focused highlight uses `accent` tokens (Radix `data-[highlighted]`-style outcome).
- Pass: Destructive item variants keep destructive foreground + use a `destructive/10`-style highlight background.
- Pass: `inset` is supported for items/labels (left padding parity with upstream `data-inset`).
- Pass: Leading icons are aligned within a fixed 16×16 slot; when any row provides a leading icon,
  the menu reserves the slot across the panel for consistent label alignment.
- Pass: Checkbox/radio indicators render the `ids::ui::CHECK` icon in a fixed 16×16 slot.

### Missing surfaces (significant)

Still missing (relative to upstream shadcn/ui v4):

_None tracked at this time._

### Submenus

- Pass: Submenus are supported via `DropdownMenuItem::submenu(Vec<DropdownMenuEntry>)` and open on
  hover/activate (pointer click), and can be opened/closed with ArrowRight/ArrowLeft.
- Pass: Pointer grace intent is implemented using a Radix-style polygon corridor (tracked via
  `fret-ui-kit::primitives::menu::pointer_grace_intent`), plus a short close delay on unsafe leave.
- Note: Submenu rendering no longer depends on pointer movement; keyboard-opened submenus render even
  when the pointer hasn't moved since the menu opened.
- Pass: Keyboard focus transfer is wired for submenus:
  - ArrowRight opens the submenu and transfers focus to the first enabled submenu item.
  - ArrowLeft closes the submenu and restores focus to the trigger item.

Notes on API mapping:

- Fret provides a single `DropdownMenu::into_element(trigger, entries)` entry point, rather than a
  DOM-like `Trigger`/`Content` component split.
- Groups/labels/shortcuts/destructive variant are modeled via:
  `DropdownMenuEntry::{Group,Label}` and `DropdownMenuItem::{trailing,variant}`.

## Validation

- Contract test: `dropdown_menu_items_have_collection_position_metadata_excluding_separators`
  (ensures `pos_in_set`/`set_size` exclude separators).
- Interaction test: `dropdown_menu_submenu_opens_on_hover_and_closes_on_leave`
- Keyboard test: `dropdown_menu_submenu_opens_on_arrow_right_without_pointer_move`
- Direction test: `dropdown_menu_align_start_respects_direction_provider`

## Follow-ups (recommended)

- Add icon/indicator slot conventions for menu rows (leading icon, checkmark/radio indicator, trailing shortcut).
- Decide whether dropdown menus need a “modal” option (or whether non-modal is the canonical Fret
  behavior).
- Consider adding a component-facing focus state for `Pressable` (mechanism-only) so menus can
  style the active item like shadcn (background highlight, not just focus ring).
- If we want Radix-level submenu ergonomics, consider:
  - intent-based safe-hover (`safePolygon` velocity heuristics), and
  - explicit keyboard focus transfer between parent menu and submenu.
