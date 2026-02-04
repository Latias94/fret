# Component recipe: ScrollArea

Goal: Radix-like scroll area with consistent scrollbar styling, correct wheel/trackpad behavior, and robust clamping.

## Upstream references

- shadcn docs: https://ui.shadcn.com/docs/components/scroll-area
- shadcn source (v4 New York registry): https://github.com/shadcn-ui/ui/blob/main/apps/v4/registry/new-york-v4/ui/scroll-area.tsx
- Radix docs: https://www.radix-ui.com/primitives/docs/components/scroll-area
- Radix source: https://github.com/radix-ui/primitives/tree/main/packages/react/scroll-area/src
- Local pinned snapshot (optional; not necessarily present on GitHub checkouts):
  - `repo-ref/ui/apps/v4/registry/new-york-v4/ui/scroll-area.tsx`
  - `repo-ref/primitives/packages/react/scroll-area/src/*`

## Fret building blocks

- Component surface: `fret-ui-shadcn::ScrollArea` (+ scrollbar primitives as exposed by the crate).
- Runtime primitive: `fret-ui::ScrollHandle` (revisioned state for viewport/content/offset).

## Checklist (what to verify)

- Input:
  - wheel/trackpad scrolls the correct surface in nested scroll scenarios
  - shift+wheel maps to horizontal scroll (policy-dependent)
- Clamping:
  - offset clamps to bounds on resize and content changes
  - no “rubber band” overscroll unless explicitly implemented
- Scrollbars:
  - thumb size reflects viewport/content ratio
  - hover/drag behavior is predictable
- Semantics:
  - scrollable region is discoverable (a11y labeling if needed)

## `test_id` suggestions

- `scroll-area-root`
- `scrollbar-vertical`
- `scrollbar-horizontal`

## See also

- `fret-scroll-and-virtualization` (when you need large lists/tables inside a scroll surface)
- `references/mind-models/mm-layout-and-sizing.md`
