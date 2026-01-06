# Radix Primitives Audit бк Presence

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

- Headless state machine: `ecosystem/fret-ui-kit/src/headless/presence.rs` (`FadePresence`)
- Runtime driver: `ecosystem/fret-ui-kit/src/declarative/presence.rs` (frame clock + redraw scheduling)
- Facade surface: `ecosystem/fret-ui-kit/src/primitives/presence.rs`

## Current parity notes

- Pass: Presence exposes the core outcome: `present` vs unmounted, with a stable transition window.
- Pass: While animating, the driver holds a continuous frames lease and requests redraws.
- Intentional difference: Fret currently implements a deterministic fade-only presence model
  (`opacity` progress), not CSS animation introspection.

## Follow-ups (recommended)

- Add additional presence drivers (e.g. scale/slide) if shadcn motion recipes need more than `opacity`.
- Consider a generic "transition timeline" headless helper if multiple components converge on the
  same easing/duration policies.

