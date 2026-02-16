# Radix Primitives Audit — Accordion


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Radix UI Primitives: https://github.com/radix-ui/primitives

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's Radix-aligned accordion substrate against the upstream Radix
`@radix-ui/react-accordion` primitive implementation pinned in `repo-ref/primitives`.

## Upstream references (source of truth)

- Implementation: `repo-ref/primitives/packages/react/accordion/src/accordion.tsx`
- Tests: `repo-ref/primitives/packages/react/accordion/src/accordion.test.tsx`
- Public exports: `repo-ref/primitives/packages/react/accordion/src/index.ts`

Key upstream concepts:

- `Accordion` root supports `type="single" | "multiple"`.
- In `single` mode, `collapsible` controls whether the open item can be closed.
- Keyboard navigation uses a trigger collection and Home/End/Arrow keys.
- Each item composes `Collapsible` and exposes `open` state.

## Fret mapping

Fret does not use React context. Instead, accordion behavior is composed via:

- Mechanism layer (runtime): `crates/fret-ui` (`RovingFlex`, focus, event dispatch).
- Headless helpers: `ecosystem/fret-ui-kit/src/headless/roving_focus.rs`
- Declarative wiring helpers: `ecosystem/fret-ui-kit/src/declarative/action_hooks.rs`
- Radix-named primitive facades: `ecosystem/fret-ui-kit/src/primitives/accordion.rs`

## Current parity notes

- Pass: `single` / `multiple` selection outcomes are supported by shadcn recipes.
- Pass: Controlled/uncontrolled selection (`value` / `defaultValue`) is supported via
  `accordion_use_single_model(...)` / `accordion_use_multiple_model(...)` (thin helpers), used by
  `AccordionRoot::{single_controllable,multiple_controllable}`.
- Pass: `collapsible` behavior in single mode matches Radix (toggle-off only when enabled).
- Pass: Trigger can model Radix `aria-controls` via the `controls_element` relationship when given
  a stable content element id.
- Note: Fret uses `RovingFlex` for keyboard navigation rather than a collection of triggers.

## Follow-ups (recommended)

- Pass: A composable, Radix-shaped surface exists in `fret-ui-kit` for non-shadcn users:
  `AccordionRoot` / `AccordionList` / `AccordionTrigger` / `AccordionContent` in
  `ecosystem/fret-ui-kit/src/primitives/accordion.rs`.
