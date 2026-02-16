# Radix Primitives Audit — Arrow


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Radix UI Primitives: https://github.com/radix-ui/primitives

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's arrow substrate against the upstream Radix
`@radix-ui/react-arrow` package pinned in `repo-ref/primitives`.

## Upstream references (source of truth)

- Implementation: `repo-ref/primitives/packages/react/arrow/src/arrow.tsx`
- Public exports: `repo-ref/primitives/packages/react/arrow/src/index.ts`

Key upstream concepts:

- `Arrow` is a reusable element that is composed into other primitives (popover/tooltip/etc.).
- Placement/rotation is driven by the popper layer via CSS variables and `side` semantics.

## Fret mapping

Fret does not use DOM/CSS variables. Arrow outcomes are composed via:

- Placement + arrow geometry: `fret-ui` overlay placement (`AnchoredPanelLayout`)
- Popper placement facade: `ecosystem/fret-ui-kit/src/primitives/popper.rs`
- Renderer-agnostic arrow element: `ecosystem/fret-ui-kit/src/primitives/popper_arrow.rs`
- Radix-named facade: `ecosystem/fret-ui-kit/src/primitives/arrow.rs`

## Current parity notes

- Pass: Arrow positioning derives from popper layout and supports all `Side` values.
- Pass: Arrow element is reusable across primitives via the `arrow` facade.

## Gaps / intentional differences

- Intentional: Fret renders a "diamond" arrow shape by default; Radix renders an SVG path.
- Deferred: A11y/styling details (the arrow is purely decorative in most recipes).

