# Radix Primitives Audit бк Presence


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Radix UI Primitives: https://github.com/radix-ui/primitives

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's Radix-aligned presence substrate against the upstream Radix
`@radix-ui/react-presence` primitive implementation pinned in `repo-ref/primitives`.

## Upstream references (source of truth)

- Implementation: `repo-ref/primitives/packages/react/presence/src/presence.tsx`
- State machine helper: `repo-ref/primitives/packages/react/presence/src/use-state-machine.tsx`
- Public exports: `repo-ref/primitives/packages/react/presence/src/index.ts`

Key upstream concepts:

- Presence keeps content mounted while exit animations run, unmounting only after the animation ends.
- It detects animations by reading computed `animation-name` and listening to DOM animation events.
- The API is "outcome driven": call sites care about `isPresent` rather than DOM details.

## Fret mapping

Fret does not have DOM animation events. Instead, Presence is modeled as:

- Headless transition timeline: `ecosystem/fret-ui-kit/src/headless/transition.rs` (`TransitionTimeline`)
  - deterministic `present` vs unmounted behavior
  - normalized `progress` value (`0..1`) with configurable easing
- Runtime driver: `ecosystem/fret-ui-kit/src/declarative/transition.rs` (frame clock + redraw scheduling)
- Presence helpers (fade / scale+fade mapping): `ecosystem/fret-ui-kit/src/declarative/presence.rs`
- Facade surfaces:
  - `ecosystem/fret-ui-kit/src/primitives/presence.rs` (Radix-named presence helpers)
  - `ecosystem/fret-ui-kit/src/primitives/transition.rs` (generic transition facade)

## Current parity notes

- Pass: Presence exposes the core outcome: `present` vs unmounted, with a stable transition window.
- Pass: While animating, the driver holds a continuous frames lease and requests redraws.
- Intentional difference: Fret does not inspect CSS `animation-name` nor listen to animation events.
  Instead it exposes a deterministic transition progress value and lets component recipes map that
  progress into opacity/scale/transforms (see `docs/audits/shadcn-motion.md`).

## Follow-ups (recommended)

- Consider adding additional mapping helpers (e.g. slide, clip) if multiple components converge on
  the same progress->transform policies.
