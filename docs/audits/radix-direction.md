# Radix Primitives Audit — Direction

This audit compares Fret's direction substrate against the upstream Radix
`@radix-ui/react-direction` helper pinned in `repo-ref/primitives`.

## Upstream references (source of truth)

- Implementation: `repo-ref/primitives/packages/react/direction/src/direction.tsx`
- Public exports: `repo-ref/primitives/packages/react/direction/src/index.ts`

Key upstream concepts:

- `useDirection(localDir?)` resolves a local override against an optional inherited direction,
  defaulting to `'ltr'`.
- `DirectionProvider` provides the inherited direction via React context.

## Fret mapping

Fret does not use React context. Direction outcomes are represented as:

- `LayoutDirection` (LTR/RTL): `fret-ui` overlay placement types
- Radix-named facade: `ecosystem/fret-ui-kit/src/primitives/direction.rs`

## Current parity notes

- Pass: `use_direction(local, inherited)` matches Radix resolution semantics.
- Pass: Direction is represented as a shared enum (`LayoutDirection`) that is already consumed by
  popper/select placement math.

## Gaps / intentional differences

- Pass: A lightweight, component-layer provider mechanism exists via
  `primitives::direction::with_direction_provider(...)` + `inherited_direction(...)`.
