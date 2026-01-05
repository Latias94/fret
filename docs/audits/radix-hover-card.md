# Radix Primitives Audit — Hover Card

This audit compares Fret's Radix-aligned hover-card substrate against the upstream Radix
`@radix-ui/react-hover-card` primitive implementation pinned in `repo-ref/primitives`.

## Upstream references (source of truth)

- Implementation: `repo-ref/primitives/packages/react/hover-card/src/hover-card.tsx`
- Public exports: `repo-ref/primitives/packages/react/hover-card/src/index.ts`

Key upstream concepts:

- A trigger opens content on hover/focus with configurable delays.
- Content is anchored via Popper and rendered in a portal.
- The open surface stays mounted while moving pointer from trigger to content (intent).

## Fret mapping

- Hover intent state machine: `ecosystem/fret-ui-kit/src/headless/hover_intent.rs` (reused by shadcn).
- Placement helpers: `ecosystem/fret-ui-kit/src/primitives/popper.rs` (+ `popper_content.rs`).
- Hover overlay policy: per-window overlays via `OverlayController` (`ecosystem/fret-ui-kit/src/window_overlays/*`).
- Radix-named facade: `ecosystem/fret-ui-kit/src/primitives/hover_card.rs` (root naming + request wiring).

## Current parity notes

- Pass: Overlay root naming and request wiring are standardized in `primitives::hover_card`.
- Pass: shadcn `HoverCard` composes hover intent + hover overlays to match Radix open/close outcomes.
- Partial: Motion/arrow rendering are handled by the shadcn recipe layer today.

