# Component recipe: Toast / Sonner

Goal: transient notifications that don’t break focus/keyboard flows and remain consistent across the app.

## Upstream references

- shadcn docs (Sonner): https://ui.shadcn.com/docs/components/sonner
- shadcn source (v4 New York registry): https://github.com/shadcn-ui/ui/blob/main/apps/v4/registry/new-york-v4/ui/sonner.tsx
- Sonner project docs (non-Radix): https://sonner.emilkowal.ski/
- Local pinned snapshot (optional; not necessarily present on GitHub checkouts):
  - `repo-ref/ui/apps/v4/registry/new-york-v4/ui/sonner.tsx`

## Fret building blocks

- Component surface:
  - `fret-ui-shadcn::Sonner` (toaster host)
  - `fret-ui-shadcn::Toast` (toast surface)
- Key requirement: the app must have **one** toast host near the root so toasts render in a stable overlay layer.

## Checklist (what to verify)

- Focus + input:
  - toasts do not steal focus
  - keyboard navigation remains unchanged while toast appears/disappears
- Dismiss:
  - timer-based auto-dismiss is stable under low FPS / jank
  - pointer dismiss (close button / swipe if supported) works predictably
- Placement + layering:
  - toasts stack and clamp inside the viewport
  - overlays do not fight with modal dialogs (z-order is deterministic)
- Reduced motion:
  - transitions respect the app’s reduced-motion policy (if present)

## `test_id` suggestions

- `toaster-root`
- `toast-<id>`
- `toast-close-<id>`

## See also

- `references/mind-models/mm-overlays-and-focus.md` (overlay layering + dismiss/focus rules)
- `fret-animation-and-scheduling` (timers/RAF and transition helpers)
- `fret-diag-workflow` (capture bundles/screenshots when timing regressions happen)
