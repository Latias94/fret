# shadcn/ui v4 Audit - Context Menu

This audit compares Fret's shadcn-aligned `ContextMenu` against the upstream shadcn/ui v4 docs and
examples in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/context-menu.mdx`
- Reference implementation (Radix base): `repo-ref/ui/apps/v4/registry/bases/radix/ui/context-menu.tsx`
- Reference example: `repo-ref/ui/apps/v4/registry/bases/radix/examples/context-menu-example.tsx`

Key upstream behaviors/surfaces:

- Rich content surface similar to dropdown menus: groups, labels, separators, shortcuts, disabled
  items, checkbox items, radio items, submenus, portals.
- Open policy: opened via a trigger region (right click / long press), not a normal button click.
- Dismissal: outside press + Escape.
- Keyboard: roving navigation + typeahead; trigger can be invoked via keyboard as well.

## Fret implementation

- Component code: `crates/fret-components-shadcn/src/context_menu.rs`
- Overlay policy substrate: `ecosystem/fret-ui-kit/src/overlay_controller.rs`
- Dismissible popover policy: `ecosystem/fret-ui-kit/src/window_overlays/mod.rs`

## Audit checklist

### Trigger policy

- Pass: Opens on right click.
- Pass: (macOS) ctrl + left click opens.
- Pass: Shift+F10 opens (fallback keyboard path).
- Note: There is no dedicated `ContextMenu` key in `fret_core::KeyCode` yet.

### Placement

- Pass: Anchored to the last pointer-down position recorded in the trigger region.
- Pass: Keyboard-open fallback anchors at the trigger bounds origin when no pointer position is
  available.
- Pass: Placement uses `anchored_panel_bounds_sized` and clamps to an inset viewport rect.

### Dismissal & focus

- Pass: Non-modal dismissible popover via `window_overlays` (outside press + Escape).
- Pass: On open, focus moves to the first focusable descendant in the menu (via overlay policy),
  enabling keyboard navigation.
- Pass: Selecting an item dispatches the command (if any) and closes the menu.

### Keyboard navigation & typeahead

- Pass: Uses `RovingFlex` + APG navigation + prefix typeahead.
- Gap: No explicit “focus transfer back to trigger” policy on close beyond the default popover
  policy (restore only when focus is missing or still inside the closing layer).

### Visual parity (shadcn)

- Partial: Token usage roughly aligns with popover/menu defaults, but the full Radix-style
  taxonomy (checkbox/radio/submenus/shortcuts/insets) is not implemented.

## Missing surfaces (significant)

Not implemented yet in Fret shadcn surface:

- Labels/groups/shortcuts (ContextMenu variants).
- Checkbox/radio items.
- Submenus and safe-hover corridor.
- Icons and inset variants.

## Validation

- Contract test: `context_menu_items_have_collection_position_metadata_excluding_separators`
- Interaction test: `context_menu_opens_on_shift_f10`

