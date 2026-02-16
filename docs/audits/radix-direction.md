# Radix Primitives Audit — Direction


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Radix UI Primitives: https://github.com/radix-ui/primitives

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
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
- Pass: For vertical placements (`top`/`bottom`), `align="start"`/`"end"` follow Floating UI/Radix
  logical alignment rules under RTL (start/end flip when `rtl && isVertical`).

## Gaps / intentional differences

- Pass: A lightweight, component-layer provider mechanism exists via
  `primitives::direction::with_direction_provider(...)` + `inherited_direction(...)`.
  - Golden path: `fret-bootstrap` installs this provider for the root view using an optional
    `LayoutDirection` global (defaults to LTR when unset).
- Note: Provider state is currently rooted to the element root id. Overlay roots created via
  `ElementContext::with_root_name(...)` do not automatically inherit this state (unlike React
  context flowing through portals).
  - Recommended: resolve `LayoutDirection` before entering an overlay root and thread it explicitly
    into placement helpers (`PopperContentPlacement::new(direction, ...)`).
  - Regression coverage:
    - Solver: `crates/fret-ui/src/overlay_placement/tests.rs` (`alignment_axis_inverts_under_rtl_for_vertical_alignments`)
    - Recipes: `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`
      (`dropdown_menu_align_start_respects_direction_provider`) and
      `ecosystem/fret-ui-shadcn/src/navigation_menu.rs`
      (`navigation_menu_viewport_align_start_respects_direction_provider`).
