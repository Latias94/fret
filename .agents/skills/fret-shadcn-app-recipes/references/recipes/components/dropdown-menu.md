# Component recipe: DropdownMenu

Goal: a Radix-like dropdown menu (button-triggered) with predictable dismiss/focus behavior and stable automation hooks.

## Upstream references

- shadcn docs: https://ui.shadcn.com/docs/components/dropdown-menu
- shadcn source (v4 New York registry): https://github.com/shadcn-ui/ui/blob/main/apps/v4/registry/new-york-v4/ui/dropdown-menu.tsx
- Radix docs: https://www.radix-ui.com/primitives/docs/components/dropdown-menu
- Radix source: https://github.com/radix-ui/primitives/tree/main/packages/react/dropdown-menu/src
- Local pinned snapshot (optional; not necessarily present on GitHub checkouts):
  - `repo-ref/ui/apps/v4/registry/new-york-v4/ui/dropdown-menu.tsx`
  - `repo-ref/primitives/packages/react/dropdown-menu/src/*`

## Fret building blocks

- Component surface: `fret-ui-shadcn::DropdownMenu` + `DropdownMenuEntry` variants.
- Model:
  - `Model<bool>` for open/closed
- Example usage: `apps/fret-ui-gallery/src/docs.rs` (Menus docs/usage snippet)

## Checklist (what to verify)

- Dismiss:
  - Escape closes
  - outside press closes
  - re-clicking trigger closes
  - submenu dismiss does not collapse unrelated overlays
- Focus:
  - open focuses first actionable item (or preserves last active, depending on policy)
  - close restores focus to trigger
- Placement:
  - clamped in constrained viewports
  - submenu placement is stable near edges
- Semantics:
  - menu roles, disabled items, checkbox/radio items announce state

## `test_id` suggestions

- `dropdown-trigger`
- `dropdown-content`
- `dropdown-item-<name>`
- `dropdown-submenu-<name>` / `dropdown-subitem-<name>`

## Regression gates (recommended)

- Scripted repro:
  - `tools/diag-scripts/ui-gallery-dropdown-open-select.json`
  - `tools/diag-scripts/ui-gallery-dropdown-submenu-bounds.json` (edge placement + screenshot)
- Add a small test for focus-restore or submenu clamp if that’s the fragile area.

## See also

- `fret-shadcn-source-alignment` (when matching upstream behavior)
- `fret-diag-workflow` (scripted repro + packaging)
