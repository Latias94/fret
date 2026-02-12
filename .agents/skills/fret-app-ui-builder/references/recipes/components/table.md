# Component recipe: Table

Goal: shadcn-style table primitives (header/body/rows/cells) with consistent density/tokens, suitable for small to medium datasets.

## Upstream references

- shadcn docs: https://ui.shadcn.com/docs/components/table
- shadcn source (v4 New York registry): https://github.com/shadcn-ui/ui/blob/main/apps/v4/registry/new-york-v4/ui/table.tsx

## Fret building blocks

- Component surface:
  - `fret-ui-shadcn::Table` (+ `TableHeader`, `TableBody`, `TableRow`, `TableHead`, `TableCell`, etc.)
- Important difference vs upstream:
  - upstream wraps `<table>` in a horizontally scrollable container;
  - Fret’s core scroll primitive may not support all horizontal scroll patterns yet, so **wrap with `ScrollArea` when needed**.

## Checklist (what to verify)

- Density:
  - row height and padding match the theme tokens (no ad-hoc px values)
  - header vs body typography is consistent
- Alignment:
  - header and body column widths stay aligned for the chosen layout strategy
- Interaction (if rows are clickable/selectable):
  - focus-visible styles for keyboard navigation
  - disabled/selected row styling is deterministic
- Overflow:
  - long text uses truncation/ellipsis consistently (per-cell policy)

## `test_id` suggestions

- `table-root`
- `table-row-<key>`
- `table-cell-<row>-<col>`

## See also

- `references/recipes/apps/app-data-table.md` (virtualized, editor-grade data table)
- `references/recipes/components/scroll-area.md` (wrapping for clipping/scroll)
- Virtualization contracts: `docs/adr/0042-virtualization-and-large-lists.md`, `docs/adr/0047-virtual-list-data-source-and-stable-item-keys.md`
