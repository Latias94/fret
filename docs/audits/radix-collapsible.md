# Radix Primitives Audit — Collapsible


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Radix UI Primitives: https://github.com/radix-ui/primitives

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
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
  - Height animation substrate: `primitives::collapsible::{last_measured_height_for, collapsible_height_wrapper_refinement, ...}`

## Current parity notes

- Pass: Root supports controlled/uncontrolled `open` state (`open` + `defaultOpen`) via
  `CollapsibleRoot` (recommended) or `collapsible_use_open_model(...)` (thin helper), backed by the
  shared controllable-state substrate.
- Pass: Trigger exposes an "expanded" a11y outcome (mapped to `PressableA11y.expanded`).
- Pass: Trigger can model Radix `aria-controls` via the `controls_element` relationship (mapped to
  `SemanticsNode.controls`) when given a stable content element id.
  - Helper: `ecosystem/fret-ui-kit/src/primitives/collapsible.rs` (`apply_collapsible_trigger_controls(...)`)
  - Reference usage: `ecosystem/fret-ui-shadcn/src/collapsible.rs`
- Pass: Dimension-driven collapse animations are modeled via a cached height clip driven by a
  deterministic transition timeline (no CSS variables).
  - Shared helper (primitives surface): `ecosystem/fret-ui-kit/src/primitives/collapsible.rs`
  - Backing implementation: `ecosystem/fret-ui-kit/src/declarative/collapsible_motion.rs`
  - Implementation: `ecosystem/fret-ui-shadcn/src/collapsible.rs`
  - Backing primitive: `ecosystem/fret-ui-kit/src/primitives/presence.rs`
  - First-open behavior: when no cached measurement exists, Fret mounts an off-flow measurement
    wrapper (opacity 0 + absolute positioning) to populate the cached height, then starts the
    open transition on the next frame.
  - Note: `--radix-collapsible-content-height/width` style variables are not modeled; we keep the
    measured size in per-element state instead.
