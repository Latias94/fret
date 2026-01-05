# Radix Primitives Audit — Accordion

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
- Pass: `collapsible` behavior in single mode matches Radix (toggle-off only when enabled).
- Note: Fret uses `RovingFlex` for keyboard navigation rather than a collection of triggers.

## Follow-ups (recommended)

- If strict parity is needed, consider adding a "composable surface" mirroring Radix (`Item` /
  `Header` / `Trigger` / `Content`) while keeping the skin in shadcn layer.

