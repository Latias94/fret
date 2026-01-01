# shadcn/ui v4 Audit — Dropdown Menu

This audit compares Fret’s shadcn-aligned `DropdownMenu` against the upstream shadcn/ui v4 docs and
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

- Component code: `crates/fret-components-shadcn/src/dropdown_menu.rs`
- Overlay policy substrate: `crates/fret-components-ui/src/overlay_controller.rs`
- Dismissible popover policy: `crates/fret-components-ui/src/window_overlays/mod.rs`
- Roving + typeahead policy helpers: `crates/fret-components-ui/src/declarative/action_hooks.rs`

## Audit checklist

### Placement & sizing

- Pass: Per-window overlay root (portal-like), rendered via `OverlayController`.
- Pass: Anchored placement via `anchored_panel_bounds_sized` (flip + clamp).
- Pass: Default `side_offset` aligns with upstream (`4`).
- Pass: Viewport size tracks trigger width (with a minimum) and clamps height to available space;
  internal scrolling is via `Scroll` (Y-axis).

### Dismissal & focus

- Pass: Non-modal dismissible popover (outside press + Escape) via `window_overlays`.
- Pass: On open, focus moves to the first focusable descendant (driven by overlay policy), enabling
  keyboard navigation inside the menu.
- Note: “modal=true vs modal=false” is not modeled; current menu is always non-modal.

### Keyboard navigation & typeahead

- Pass: `RovingFlex` + APG-style navigation (`roving_nav_apg`) are wired.
- Pass: Prefix typeahead with timeout is wired.

### Visual parity (shadcn)

- Partial: Menu background/foreground now align with popover tokens (`bg-popover text-popover-foreground`).
- Partial: Hover/pressed highlight uses `accent` tokens; focus-highlight parity is limited because
  `PressableState` does not currently expose a “focused” flag for background styling.

### Missing surfaces (significant)

Not implemented yet in Fret shadcn surface:

- Checkbox/radio surfaces: `DropdownMenuCheckboxItem` / `DropdownMenuRadioGroup` / `DropdownMenuRadioItem`
- Submenus: `DropdownMenuSub` / `DropdownMenuSubTrigger` / `DropdownMenuSubContent`
- Styling knobs: inset/padding variants, icons, and “active item” highlight parity (needs focused state)

Notes on API mapping:

- Fret provides a single `DropdownMenu::into_element(trigger, entries)` entry point, rather than a
  DOM-like `Trigger`/`Content` component split.
- Groups/labels/shortcuts/destructive variant are modeled via:
  `DropdownMenuEntry::{Group,Label}` and `DropdownMenuItem::{trailing,variant}`.

## Validation

- Contract test: `dropdown_menu_items_have_collection_position_metadata_excluding_separators`
  (ensures `pos_in_set`/`set_size` exclude separators).

## Follow-ups (recommended)

- Add missing shadcn surfaces gradually, starting with: `Label`, `Group`, `Shortcut`, destructive
  variant, then checkbox/radio items.
- Decide whether dropdown menus need a “modal” option (or whether non-modal is the canonical Fret
  behavior).
- Consider adding a component-facing focus state for `Pressable` (mechanism-only) so menus can
  style the active item like shadcn (background highlight, not just focus ring).
