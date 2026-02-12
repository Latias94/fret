# Component recipe: Popover

Goal: a Radix-like popover (non-modal overlay) with predictable dismiss/focus and robust viewport clamping.

## Upstream references

- shadcn docs: https://ui.shadcn.com/docs/components/popover
- shadcn source (v4 New York registry): https://github.com/shadcn-ui/ui/blob/main/apps/v4/registry/new-york-v4/ui/popover.tsx
- Radix docs: https://www.radix-ui.com/primitives/docs/components/popover
- Radix source: https://github.com/radix-ui/primitives/tree/main/packages/react/popover/src

## Fret building blocks

- Component surface: `fret-ui-shadcn::Popover` + `PopoverTrigger` / `PopoverContent` (+ header/title/description helpers).
- Model:
  - `Model<bool>` for open/closed

## Checklist (what to verify)

- Dismiss:
  - Escape closes
  - outside press closes (policy-dependent; be explicit)
  - re-clicking trigger closes
- Focus:
  - open does not trap focus (unless intentionally composed as modal)
  - close restores focus to trigger
- Placement:
  - clamped in constrained viewports
  - scrollable content max-height in tiny windows
- Semantics:
  - content is discoverable to a11y (role and labeling consistent with shadcn)

## `test_id` suggestions

- `popover-trigger`
- `popover-content`

## Regression gates (recommended)

- Scripted repro:
  - open/close via click + Escape + outside press
  - clamp near window edges (screenshot optional)
- Add a small test for focus restore or clamp if that’s the fragile area.

## See also

- `references/mind-models/mm-overlays-and-focus.md`
- `fret-shadcn-source-alignment`
- `fret-diag-workflow`
