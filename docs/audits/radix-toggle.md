# Radix Primitives Audit — Toggle


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Radix UI Primitives: https://github.com/radix-ui/primitives

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's Radix-aligned toggle substrate against the upstream Radix
`@radix-ui/react-toggle` primitive implementation pinned in `repo-ref/primitives`.

## Upstream references (source of truth)

- Implementation: `repo-ref/primitives/packages/react/toggle/src/toggle.tsx`
- Tests: `repo-ref/primitives/packages/react/toggle/src/toggle.test.tsx`
- Public exports: `repo-ref/primitives/packages/react/toggle/src/index.ts`

Key upstream concepts:

- `Toggle` is a button-like control with `aria-pressed` state.
- It supports controlled/uncontrolled `pressed` state and `onPressedChange`.
- It exposes a `data-state="on|off"` attribute for styling.

## Fret mapping

Fret does not use React/DOM attributes. Instead, toggle behavior is composed via:

- Mechanism layer (runtime): `crates/fret-ui` (`Pressable`, focus, event dispatch).
- Declarative wiring helpers: `ecosystem/fret-ui-kit/src/declarative/action_hooks.rs`
- Radix-named primitive facades: `ecosystem/fret-ui-kit/src/primitives/toggle.rs`

## Current parity notes

- Pass: Controlled and uncontrolled state are supported (`pressed` / `defaultPressed`) via
  `ecosystem/fret-ui-kit/src/primitives/toggle.rs` (`toggle_use_model`) (backed by the shared
  controllable-state substrate).
- Pass: Activation toggling exists in shadcn recipes.
- Note: Fret currently maps `aria-pressed` to a button-like semantics outcome (`selected=true`).
