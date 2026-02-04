# Component recipe: Tooltip

Goal: a Radix-like tooltip that behaves correctly for hover and keyboard focus without stealing focus.

## Upstream references

- shadcn docs: https://ui.shadcn.com/docs/components/tooltip
- shadcn source (v4 New York registry): https://github.com/shadcn-ui/ui/blob/main/apps/v4/registry/new-york-v4/ui/tooltip.tsx
- Radix docs: https://www.radix-ui.com/primitives/docs/components/tooltip
- Radix source: https://github.com/radix-ui/primitives/tree/main/packages/react/tooltip/src
- Local pinned snapshot (optional; not necessarily present on GitHub checkouts):
  - `repo-ref/ui/apps/v4/registry/new-york-v4/ui/tooltip.tsx`
  - `repo-ref/primitives/packages/react/tooltip/src/*`

## Fret building blocks

- Component surface: `fret-ui-shadcn::TooltipProvider` + `Tooltip` / `TooltipTrigger` / `TooltipContent`.
- Typical composition: provider near the app root, tooltips local to each interactive control.

## Checklist (what to verify)

- Hover/focus behavior:
  - hover opens after delay; leaving closes (with an optional grace period)
  - focus can open for keyboard users; blur closes
- Dismiss:
  - Escape closes
  - moving pointer away closes without flicker
- Placement:
  - clamped in constrained viewports
  - content max width does not explode layouts
- Semantics:
  - role `tooltip`
  - trigger sets `aria-describedby` (or equivalent semantics binding)

## `test_id` suggestions

- `tooltip-trigger-<name>`
- `tooltip-content-<name>`

## Regression gates (recommended)

- Scripted repro:
  - hover trigger, wait for open, assert `tooltip-content-*` appears
  - focus trigger via keyboard, assert tooltip shows without focus change
- Add a small test for placement clamp if the runtime changes.

## See also

- `references/mind-models/mm-overlays-and-focus.md`
- `fret-shadcn-source-alignment`
- `fret-diag-workflow`
