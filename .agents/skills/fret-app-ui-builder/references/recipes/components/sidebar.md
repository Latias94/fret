# Component recipe: Sidebar

Goal: a shadcn-style sidebar layout suitable for editor apps (navigation + panels) with stable sizing and keyboard traversal.

## Upstream references

- shadcn docs: https://ui.shadcn.com/docs/components/sidebar
- shadcn source (v4 New York registry): https://github.com/shadcn-ui/ui/blob/main/apps/v4/registry/new-york-v4/ui/sidebar.tsx

## Fret building blocks

- Component surface: `fret-ui-shadcn::Sidebar` (and related helper types as exposed by the crate).
- Composition:
  - sidebar often combines: navigation, search, collapsible groups, and scroll areas.

## Checklist (what to verify)

- Layout:
  - collapsed/expanded widths are consistent across the app (token-driven)
  - content area does not reflow unexpectedly when toggling
- Keyboard traversal:
  - focus order is predictable (nav → content)
  - focus-visible styles are present and consistent
- Scroll:
  - sidebar content scrolls independently when long
- Persistence (if desired):
  - collapsed state persists per workspace/layout profile

## `test_id` suggestions

- `sidebar-root`
- `sidebar-toggle`
- `sidebar-item-<id>`

## See also

- `references/mind-models/mm-theme-and-tokens.md`
- `references/mind-models/mm-layout-and-sizing.md`
- Command routing and keymaps: `docs/adr/0020-focus-and-command-routing.md`, `docs/adr/0021-keymap-file-format.md`, `docs/adr/0022-when-expressions.md`, `docs/adr/0023-command-metadata-menus-and-palette.md`
