# Component recipe: ContextMenu

Goal: a right-click menu (contextmenu-triggered) that matches Radix expectations and stays stable near viewport edges.

## Upstream references

- shadcn docs: https://ui.shadcn.com/docs/components/context-menu
- shadcn source (v4 New York registry): https://github.com/shadcn-ui/ui/blob/main/apps/v4/registry/new-york-v4/ui/context-menu.tsx
- Radix docs: https://www.radix-ui.com/primitives/docs/components/context-menu
- Radix source: https://github.com/radix-ui/primitives/tree/main/packages/react/context-menu/src

## Fret building blocks

- Component surface: `fret-ui-shadcn::ContextMenu` + `ContextMenuEntry` variants.
- Model:
  - `Model<bool>` for open/closed
- Example usage: `apps/fret-ui-gallery/src/docs.rs` (Menus)

## Checklist

- Right click opens at pointer position.
- Edge behavior is clamped (no offscreen content).
- Escape/outside press dismisses.
- Focus and keyboard navigation behave like a menu.

## Regression gates

- `tools/diag-scripts/ui-gallery-contextmenu-edge-bounds.json` (screenshot-backed edge clamp)

## See also

- `fret-shadcn-source-alignment` (when matching upstream behavior)
- `fret-diag-workflow` (scripted repro + packaging)
