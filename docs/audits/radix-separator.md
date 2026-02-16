# Radix Primitives Audit бк Separator


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Radix UI Primitives: https://github.com/radix-ui/primitives

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's Radix-aligned separator substrate against the upstream Radix
`@radix-ui/react-separator` implementation pinned in `repo-ref/primitives`.

## Upstream references (source of truth)

- Implementation: `repo-ref/primitives/packages/react/separator/src/separator.tsx`
- Public exports: `repo-ref/primitives/packages/react/separator/src/index.ts`

Key upstream concepts:

- `Separator` supports `orientation="horizontal" | "vertical"` (default `horizontal`).
- `decorative` can remove the node from the accessibility tree (`role="none"`).
- Non-decorative separators use `role="separator"` and `aria-orientation` when vertical.

## Fret mapping

- Primitive facade: `ecosystem/fret-ui-kit/src/primitives/separator.rs`
- shadcn re-export: `ecosystem/fret-ui-shadcn/src/separator.rs`

## Current parity notes

- Pass: Provides `orientation` outcomes (horizontal vs vertical sizing).
- Note: Fret's semantics roles do not currently include a dedicated `Separator` role, and
  `aria-orientation` has no direct mapping in `SemanticsProps` today.
- Note: The current implementation is a visual/layout separator; a11y parity can be revisited
  once the semantics bridge grows a `Separator` role and/or orientation metadata.

