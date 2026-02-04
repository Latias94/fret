# Component recipe: NavigationMenu

Goal: a Radix-like navigation menu with stable viewport/indicator behavior and robust focus + dismiss semantics.

## Upstream references

- shadcn docs: https://ui.shadcn.com/docs/components/navigation-menu
- shadcn source (v4 New York registry): https://github.com/shadcn-ui/ui/blob/main/apps/v4/registry/new-york-v4/ui/navigation-menu.tsx
- Radix docs: https://www.radix-ui.com/primitives/docs/components/navigation-menu
- Radix source: https://github.com/radix-ui/primitives/tree/main/packages/react/navigation-menu/src
- Local pinned snapshot (optional; not necessarily present on GitHub checkouts):
  - `repo-ref/ui/apps/v4/registry/new-york-v4/ui/navigation-menu.tsx`
  - `repo-ref/primitives/packages/react/navigation-menu/src/*`

## Fret building blocks

- Component surface: `fret-ui-shadcn::NavigationMenu` (+ `NavigationMenuList`, `NavigationMenuItem`, `NavigationMenuTrigger`, `NavigationMenuContent`, `NavigationMenuViewport`, `NavigationMenuIndicator`).
- Typical composition: a top-level nav bar where active content is rendered into a shared viewport.

## Checklist (what to verify)

- Trigger behavior:
  - click and/or hover opens (follow upstream policy; be consistent)
  - switching between triggers updates content without “stale focus”
- Dismiss:
  - Escape closes
  - outside press closes
  - focus leaving closes (policy-dependent; align with Radix)
- Focus + keyboard nav:
  - roving focus within triggers and within content, as appropriate
  - close restores focus to the correct trigger
- Viewport/indicator:
  - viewport positions correctly under the active trigger
  - indicator moves predictably and doesn’t desync from content
- Placement:
  - clamped in constrained viewports

## `test_id` suggestions

- `nav-menu-root`
- `nav-menu-trigger-<name>`
- `nav-menu-content-<name>`
- `nav-menu-viewport`
- `nav-menu-indicator`

## Regression gates (recommended)

- Scripted repro:
  - switch triggers, assert viewport/indicator updates
  - keyboard nav across triggers, assert focus and content correctness
- Add a small test for viewport anchoring math if that’s the fragile area.

## See also

- `references/mind-models/mm-overlays-and-focus.md`
- `fret-shadcn-source-alignment`
- `fret-diag-workflow`
