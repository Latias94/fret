# Radix Primitives Audit - Separator


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
- Pass: Non-decorative separators map to `SemanticsRole::Separator`, and vertical separators also
  carry `SemanticsOrientation::Vertical`.
- Pass: Decorative separators are hidden from the semantics tree, matching the Radix `role="none"`
  outcome.
- Note: Base UI's headless separator is always semantic; Fret keeps that substrate in
  `fret-ui-kit::primitives::separator::Separator`, while the shadcn recipe layer adds the
  Radix-style `.decorative(...)` override.
