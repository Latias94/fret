# Radix Primitives Audit бк Radio Group


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Radix UI Primitives: https://github.com/radix-ui/primitives

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's Radix-aligned radio-group substrate against the upstream Radix
`@radix-ui/react-radio-group` primitive implementation pinned in `repo-ref/primitives`.

## Upstream references (source of truth)

- Implementation: `repo-ref/primitives/packages/react/radio-group/src/radio-group.tsx`
- Radio primitive: `repo-ref/primitives/packages/react/radio-group/src/radio.tsx`
- Public exports: `repo-ref/primitives/packages/react/radio-group/src/index.ts`

Key upstream concepts:

- RadioGroup root is a roving focus group (`loop=true` by default) with `role="radiogroup"`.
- Items are radios (`role="radio"`) and selection is committed when focus moves via arrow keys.
- According to WAI-ARIA, radios do not activate on Enter; Radix consumes `Enter` in `onKeyDown`.

## Fret mapping

- Mechanism layer (runtime): `crates/fret-ui` (`RovingFlex`, focus, event dispatch).
- Headless helpers: `ecosystem/fret-ui-kit/src/headless/roving_focus.rs`.
- A11y stamping helpers: `ecosystem/fret-ui-kit/src/primitives/radio_group.rs`.
- shadcn recipe: `ecosystem/fret-ui-shadcn/src/radio_group.rs`.

## Current parity notes

- Pass: Roving focus navigation is wired via `RovingFlex` + `cx.roving_nav_apg()`.
- Pass: Arrow navigation commits selection via `cx.roving_select_option_arc_str(...)`.
- Pass: Enter key presses are consumed; Space activates the focused item (Radix/WAI-ARIA parity).
- Pass: Controlled/uncontrolled selection (`value` / `defaultValue`) is supported via
  `ecosystem/fret-ui-kit/src/primitives/controllable_state.rs`.

## Follow-ups (recommended)

- If strict parity is required, consider modeling form submission (`name`, `required`) semantics
  for radio groups. Fret currently focuses on interaction outcomes rather than DOM forms.
