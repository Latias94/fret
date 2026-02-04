# Component recipe: Menubar

Goal: an editor-grade menubar with predictable keyboard navigation, submenu behavior, and stable command integration.

## Upstream references

- shadcn docs: https://ui.shadcn.com/docs/components/menubar
- shadcn source (v4 New York registry): https://github.com/shadcn-ui/ui/blob/main/apps/v4/registry/new-york-v4/ui/menubar.tsx
- Radix docs: https://www.radix-ui.com/primitives/docs/components/menubar
- Radix source: https://github.com/radix-ui/primitives/tree/main/packages/react/menubar/src
- Local pinned snapshot (optional; not necessarily present on GitHub checkouts):
  - `repo-ref/ui/apps/v4/registry/new-york-v4/ui/menubar.tsx`
  - `repo-ref/primitives/packages/react/menubar/src/*`

## Fret building blocks

- Component surface: `fret-ui-shadcn::Menubar` + `MenubarMenuEntries` + `MenubarEntry` variants.
- Models:
  - checkbox/radio menu entries use `Model<bool>` / `Model<Option<Arc<str>>>` where applicable.
- Command integration:
  - prefer routing through `CommandId` + command metadata (menus and palette share the same source of truth).

## Checklist (what to verify)

- Keyboard navigation:
  - left/right moves between top-level menus
  - up/down navigates within the menu
  - Enter activates; Escape closes; Home/End semantics are consistent
- Dismiss:
  - outside press closes
  - Escape closes the current menu/submenu without collapsing unrelated overlays
- Focus:
  - open focuses first actionable item (or last active, depending on policy)
  - close restores focus to the menubar trigger
- Placement:
  - clamped in constrained viewports
  - submenus remain usable near edges
- Semantics:
  - menubar/menu roles; disabled and check/radio state is announced

## `test_id` suggestions

- `menubar-root`
- `menubar-menu-<name>`
- `menubar-item-<name>`
- `menubar-submenu-<name>` / `menubar-subitem-<name>`

## Regression gates (recommended)

- Scripted repro:
  - open via mouse, navigate via arrows, activate an item, assert command dispatch effect
  - submenu bounds near window edges (screenshot optional)
- Add a small test for focus restore or submenu clamp if that’s the fragile area.

## See also

- `references/mind-models/mm-models-actions-and-commands.md`
- `references/mind-models/mm-overlays-and-focus.md`
- `fret-commands-and-keymap`
- `fret-shadcn-source-alignment`
- `fret-diag-workflow`
