# Radix Primitives Audit — Checkbox


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Radix UI Primitives: https://github.com/radix-ui/primitives

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's Radix-aligned checkbox substrate against the upstream Radix
`@radix-ui/react-checkbox` primitive implementation pinned in `repo-ref/primitives`.

## Upstream references (source of truth)

- Implementation: `repo-ref/primitives/packages/react/checkbox/src/checkbox.tsx`
- Tests: `repo-ref/primitives/packages/react/checkbox/src/checkbox.test.tsx`
- Public exports: `repo-ref/primitives/packages/react/checkbox/src/index.ts`

Key upstream concepts:

- Checkbox is a pressable root with `role="checkbox"`.
- Checked state is tri-state: `false | true | "indeterminate"`.
- A11y uses `aria-checked="mixed"` for indeterminate.

## Fret mapping

Fret models Radix checkbox outcomes as:

- Headless tri-state: `crate::headless::checked_state::CheckedState`
- Radix-named facade: `ecosystem/fret-ui-kit/src/primitives/checkbox.rs`
- Authoring layers (recipes) build on top of the facade, e.g. `ecosystem/fret-ui-shadcn/src/checkbox.rs`.

## Current parity notes

- Pass: Tri-state behavior is modeled via `CheckedState` (checked / unchecked / indeterminate).
- Pass: Semantics maps indeterminate to `checked: None` (equivalent to Radix `aria-checked="mixed"`).
- Pass: Keyboard activation matches Radix outcomes: Space toggles, Enter is consumed (does not toggle).
- Pass: Optional boolean bindings (`Option<bool>`) can be mapped/toggled via primitives helpers
  (`checked_state_from_optional_bool`, `toggle_optional_bool`), keeping recipes thin.
- Pass: Controlled/uncontrolled checked state (`checked` / `defaultChecked`) can be modeled via
  `checkbox_use_checked_model(...)`.
- Note: Fret currently does not model DOM-style `name`/form submission semantics.

## Follow-ups (recommended)

- If strict parity is required, consider introducing a first-class `A11yChecked::Mixed` instead of
  using `Option<bool>` in the semantics snapshot layer.
