# Component recipe: Tabs

Goal: Radix-like tabs with predictable keyboard navigation, stable selection state, and correct focus rings.

## Upstream references

- shadcn docs: https://ui.shadcn.com/docs/components/tabs
- shadcn source (v4 New York registry): https://github.com/shadcn-ui/ui/blob/main/apps/v4/registry/new-york-v4/ui/tabs.tsx
- Radix docs: https://www.radix-ui.com/primitives/docs/components/tabs
- Radix source: https://github.com/radix-ui/primitives/tree/main/packages/react/tabs/src
- Local pinned snapshot (optional; not necessarily present on GitHub checkouts):
  - `repo-ref/ui/apps/v4/registry/new-york-v4/ui/tabs.tsx`
  - `repo-ref/primitives/packages/react/tabs/src/*`

## Fret building blocks

- Component surface: `fret-ui-shadcn::Tabs` (+ `TabsList`, `TabsTrigger`, `TabsContent`).
- Model:
  - usually a `Model<Option<Arc<str>>>` or equivalent “selected tab value”
- When tabs are used as an editor surface (like “docked tabs”), see docking recipes instead.

## Checklist (what to verify)

- Keyboard navigation:
  - left/right (or up/down for vertical) changes the active trigger (roving focus)
  - Home/End moves to first/last trigger
  - Enter/Space activates if selection is “manual” (policy-dependent; align with Radix)
- Focus + rings:
  - focus-visible styling applies only for keyboard modality
  - active trigger shows the correct state chrome without requiring focus
- Content:
  - content mounts/unmounts policy is explicit (performance vs state retention)
  - content does not cause layout jumps when switching
- Semantics:
  - correct roles and relationships (tablist/tab/tabpanel)
  - disabled tabs are non-interactive and announced

## `test_id` suggestions

- `tabs-root`
- `tabs-trigger-<value>`
- `tabs-content-<value>`

## See also

- `fret-action-hooks` (roving/typeahead policies when you need custom behavior)
- `references/mind-models/mm-a11y-and-testid.md`
- `fret-shadcn-source-alignment`
