# Radix Primitives Audit — Progress


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Radix UI Primitives: https://github.com/radix-ui/primitives

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's Radix-aligned progress substrate against the upstream Radix
`@radix-ui/react-progress` primitive implementation pinned in `repo-ref/primitives`.

## Upstream references (source of truth)

- Implementation: `repo-ref/primitives/packages/react/progress/src/progress.tsx`
- Public exports: `repo-ref/primitives/packages/react/progress/src/index.ts`

Key upstream concepts:

- `Progress` renders a progressbar and clamps `value` within `[0, max]`.
- `value` can be absent for indeterminate progress.

## Fret mapping

- Radix-named facade: `ecosystem/fret-ui-kit/src/primitives/progress.rs`.
- shadcn recipe layer uses `normalize_progress(...)` to compute the fill fraction.

## Current parity notes

- Pass: Progress normalization clamps into a `[0, 1]` fraction.
- Pass: Fret stamps `role=ProgressBar`, numeric range/value, and a default percentage `aria-valuetext`-like value for determinate progress.
- Pass: Optional/indeterminate progress omits numeric/value semantics while keeping the visual indicator at the Radix/shadcn 0% baseline.
- Note: The default shadcn/Radix lane remains leaf-first; Base UI-style label/value child slots are treated as a separate headless surface, not the default recipe contract.
