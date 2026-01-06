# Radix Primitives Audit — Toggle Group

This audit compares Fret's Radix-aligned toggle-group substrate against the upstream Radix
`@radix-ui/react-toggle-group` primitive implementation pinned in `repo-ref/primitives`.

## Upstream references (source of truth)

- Implementation: `repo-ref/primitives/packages/react/toggle-group/src/toggle-group.tsx`
- Tests: `repo-ref/primitives/packages/react/toggle-group/src/toggle-group.test.tsx`
- Public exports: `repo-ref/primitives/packages/react/toggle-group/src/index.ts`

Key upstream concepts:

- `ToggleGroup` root supports `type="single" | "multiple"` and is optionally roving-focus driven.
- `ToggleGroupImpl` wraps a roving focus group by default (`loop=true`).
- `ToggleGroupItem` composes Radix `Toggle` and updates the group value via activation callbacks.
- A11y: in `single` mode items use `role="radio"` + `aria-checked`; in `multiple` mode items use
  `aria-pressed` (via `Toggle`).

## Fret mapping

Fret does not use React context. Instead, toggle-group behavior is composed via:

- Mechanism layer (runtime): `crates/fret-ui` (`RovingFlex`, focus, event dispatch).
- Headless helpers: `ecosystem/fret-ui-kit/src/headless/roving_focus.rs`
- Declarative wiring helpers: `ecosystem/fret-ui-kit/src/declarative/action_hooks.rs`
- Radix-named primitive facades: `ecosystem/fret-ui-kit/src/primitives/toggle_group.rs`

## Current parity notes

- Pass: `single` / `multiple` selection outcomes are supported by shadcn recipes.
- Pass: `orientation` + `loop` outcomes are supported via `RovingFlex` + APG navigation.
- Pass: Item semantics matches Radix outcomes: `single` mode uses `RadioButton` + `checked`, while
  `multiple` mode uses a button-like `selected` flag (pressed).

## Follow-ups (recommended)

- Consider adding a semantics mapping for single mode to use `SemanticsRole::RadioGroup` +
  `SemanticsRole::RadioButton` if strict parity is required.
