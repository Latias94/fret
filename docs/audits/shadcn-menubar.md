# shadcn/ui v4 Audit - Menubar

This audit compares Fret's shadcn-aligned `Menubar` against the upstream shadcn/ui v4 docs and
examples in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/menubar.mdx`
- Reference implementation (Radix base): `repo-ref/ui/apps/v4/registry/bases/radix/ui/menubar.tsx`
- Reference example: `repo-ref/ui/apps/v4/registry/bases/radix/examples/menubar-example.tsx`

Key upstream behaviors/surfaces:

- Menubar root + menu triggers with roving focus across the bar (ArrowLeft/ArrowRight).
- Menu content: groups, labels, separators, shortcuts, disabled items, checkbox/radio items,
  submenus.
- Open policy: click-to-open; hover switches menus when one is already open.
- Dismissal: Escape + outside press.

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/menubar.rs`
- Overlay policy substrate: `ecosystem/fret-ui-kit/src/overlay_controller.rs`
- Dismissible popover policy: `ecosystem/fret-ui-kit/src/window_overlays/mod.rs`

## Audit checklist

### Open/close policy

- Pass: Click-to-open per menu trigger.
- Pass: Hover switches the open menu when a menu is already active.
- Pass: Outside press + Escape dismiss via popover policy.
- Note: Fret exposes an explicit `close_on_select` policy per item; upstream Radix typically relies
  on `onSelect(e) { e.preventDefault() }` to keep menus open for toggles.

### Placement & sizing

- Pass: Anchored placement to the trigger bounds (`Side::Bottom`, `Align::Start`).
- Partial: Estimated sizing only; width/height parity with upstream examples is not exact.

### Keyboard navigation

- Pass: Menu content supports roving + typeahead.
- Pass: Menubar trigger row supports ArrowLeft/ArrowRight roving (wrap) between triggers.

### Visual parity

- Partial: Token usage aligns with shadcn-ish defaults (border/background/radius), but some
  higher-fidelity slots are still text-based or consumer-provided (see missing surfaces).
- Pass: `inset` is supported for items/labels (left padding parity with upstream `data-inset`).
- Pass: Submenu triggers render a right chevron (text fallback) to match upstream affordance.

## Missing surfaces (significant)

Still missing (relative to upstream shadcn/ui v4):

- Chevron-right submenu icon parity (currently rendered as a text fallback).
- Consistent leading icon slot sizing/alignment across all item variants (currently consumer-provided).
- Destructive item variant styling.

## Validation

- Interaction test: `menubar_hover_switches_open_menu`
- Interaction test: `menubar_triggers_roving_moves_focus_with_arrow_keys`
- Contract test: `menubar_items_have_collection_position_metadata_excluding_separators`
