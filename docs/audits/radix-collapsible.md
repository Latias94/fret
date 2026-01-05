# Radix Primitives Audit — Collapsible

This audit compares Fret's Radix-aligned collapsible substrate against the upstream Radix
`@radix-ui/react-collapsible` primitive implementation pinned in `repo-ref/primitives`.

## Upstream references (source of truth)

- Implementation: `repo-ref/primitives/packages/react/collapsible/src/collapsible.tsx`
- Tests: `repo-ref/primitives/packages/react/collapsible/src/collapsible.test.tsx`
- Public exports: `repo-ref/primitives/packages/react/collapsible/src/index.ts`

Key upstream concepts:

- `Collapsible` root is a controlled/uncontrolled boolean `open` state with `onOpenChange`.
- `CollapsibleTrigger` toggles open state and exposes `aria-expanded` + `aria-controls`.
- `CollapsibleContent` uses Radix `Presence` to coordinate mount/unmount and measures its full size
  to expose `--radix-collapsible-content-height/width` CSS variables for height animations.

## Fret mapping

Fret does not use React context or CSS variables. Collapsible outcomes are composed via:

- Mechanism layer (runtime): `crates/fret-ui` (`Pressable`, focus, event dispatch).
- Declarative helpers: `ecosystem/fret-ui-kit/src/declarative/action_hooks.rs`
- Radix-named primitive facades: `ecosystem/fret-ui-kit/src/primitives/collapsible.rs`

## Current parity notes

- Pass: Trigger exposes an "expanded" a11y outcome (mapped to `PressableA11y.expanded`).
- Note: Fret does not currently model `aria-controls` wiring (content id references).
- Partial: Dimension-driven collapse animations are modeled as a presence-driven "keep mounted while
  closing" + cached height clip (no CSS variables yet).
  - Shared helper: `ecosystem/fret-ui-kit/src/declarative/collapsible_motion.rs`
  - Implementation: `ecosystem/fret-ui-shadcn/src/collapsible.rs`
  - Backing primitive: `ecosystem/fret-ui-kit/src/primitives/presence.rs`
  - Note: `--radix-collapsible-content-height/width` style variables are not modeled; we keep the
    measured height in per-element state instead.
