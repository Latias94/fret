# Radix Primitives Audit — Aspect Ratio


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Radix UI Primitives: https://github.com/radix-ui/primitives

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's Radix-aligned aspect ratio substrate against the upstream Radix
`@radix-ui/react-aspect-ratio` primitive implementation pinned in `repo-ref/primitives`.

## Upstream references (source of truth)

- Implementation: `repo-ref/primitives/packages/react/aspect-ratio/src/aspect-ratio.tsx`
- Public exports: `repo-ref/primitives/packages/react/aspect-ratio/src/index.ts`

Key upstream concepts:

- `AspectRatio` enforces a preferred ratio for its child by using CSS sizing mechanics.
- The primitive is intentionally small and visual-agnostic.

## Fret mapping

- Aspect ratio is modeled in layout (`LayoutStyle.aspect_ratio`) and resolved by the layout engine
  (ADR 0057).
- Radix-named facade: `ecosystem/fret-ui-kit/src/primitives/aspect_ratio.rs`.

## Current parity notes

- Pass: `AspectRatio` stamps `layout.aspect_ratio = Some(ratio)` and defaults to clipped overflow,
  matching the upstream outcome.

