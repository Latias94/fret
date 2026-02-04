# Component recipe: HoverCard

Goal: a Radix-like hover card that uses hover intent (doesn’t flicker) and survives “pointer travel” from trigger → content.

## Upstream references

- shadcn docs: https://ui.shadcn.com/docs/components/hover-card
- shadcn source (v4 New York registry): https://github.com/shadcn-ui/ui/blob/main/apps/v4/registry/new-york-v4/ui/hover-card.tsx
- Radix docs: https://www.radix-ui.com/primitives/docs/components/hover-card
- Radix source: https://github.com/radix-ui/primitives/tree/main/packages/react/hover-card/src
- Local pinned snapshot (optional; not necessarily present on GitHub checkouts):
  - `repo-ref/ui/apps/v4/registry/new-york-v4/ui/hover-card.tsx`
  - `repo-ref/primitives/packages/react/hover-card/src/*`

## Fret building blocks

- Component surface: `fret-ui-shadcn::HoverCard` + `HoverCardTrigger` / `HoverCardContent`.
- Policy: hover intent thresholds + leave delays should be consistent across the app (avoid per-call magic numbers).

## Checklist (what to verify)

- Hover intent:
  - does not open instantly (delay)
  - does not close instantly when moving into content (leave grace)
- Dismiss:
  - Escape closes
  - outside press closes (if enabled)
- Placement:
  - clamped near edges
  - content stays readable in tiny viewports
- Semantics:
  - content is properly labeled; trigger indicates relationship

## `test_id` suggestions

- `hover-card-trigger-<name>`
- `hover-card-content-<name>`

## Regression gates (recommended)

- Scripted repro:
  - hover trigger, wait open, move pointer into content, assert it stays open
  - move pointer away, assert it closes after delay
- Add a small test for hover intent timing if policies change.

## See also

- `references/mind-models/mm-overlays-and-focus.md`
- `fret-shadcn-source-alignment`
- `fret-diag-workflow`
