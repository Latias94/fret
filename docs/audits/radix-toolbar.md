# Radix Primitives Audit — Toolbar


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Radix UI Primitives: https://github.com/radix-ui/primitives

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's toolbar substrate against the upstream Radix
`@radix-ui/react-toolbar` primitive implementation pinned in `repo-ref/primitives`.

## Upstream references (source of truth)

- Implementation: `repo-ref/primitives/packages/react/toolbar/src/toolbar.tsx`
- Public exports: `repo-ref/primitives/packages/react/toolbar/src/index.ts`

Key upstream concepts:

- `Toolbar` root is a `RovingFocusGroup.Root` wrapper with:
  - `orientation` (`horizontal` default)
  - `loop` navigation (`true` default)
  - `dir` support for RTL/LTR
- `ToolbarSeparator` maps its orientation to the opposite axis of the toolbar.
- `ToolbarToggleGroup` wraps Radix `ToggleGroup.Root` with `rovingFocus=false` (toolbar owns roving).

## Fret mapping

Fret does not use React context. Toolbar outcomes are composed via:

- Roving focus: `ecosystem/fret-ui-kit/src/primitives/roving_focus_group.rs`
- Separator: `ecosystem/fret-ui-kit/src/primitives/separator.rs`
- Toggle group: `ecosystem/fret-ui-kit/src/primitives/toggle_group.rs`
- Radix-named facade: `ecosystem/fret-ui-kit/src/primitives/toolbar.rs`

## Current parity notes

- Pass: Toolbar roving focus defaults can be expressed via `toolbar_roving_flex_props(...)`.
- Pass: Separator axis mapping is exposed via `toolbar_separator_orientation(...)`.
- Pass: Toolbar-level roving focus composition helper exists via `toolbar_roving_group_apg(...)`.
- Pass: RTL/LTR arrow-key behavior can be expressed by installing a direction-aware roving policy
  via `toolbar_roving_group_apg_with_direction(...)` (delegates to `roving_focus_group`'s Radix
  `dir` mapping).

## Gaps / intentional differences

- Deferred: Full toolbar a11y semantics (`role="toolbar"`), pending the a11y roadmap.
