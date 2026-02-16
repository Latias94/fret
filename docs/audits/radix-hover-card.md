# Radix Primitives Audit — Hover Card


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Radix UI Primitives: https://github.com/radix-ui/primitives

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
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
- Pass: Motion math is available in `declarative::overlay_motion`, and diamond arrow rendering is
  available via `primitives::popper_arrow`; recipes primarily wire tokens and layout.
- Pass: Controlled/uncontrolled open modeling is available via `primitives::hover_card::HoverCardRoot`
  (`open` / `defaultOpen`) and is exposed by the shadcn recipe via `HoverCard::{open, default_open}`.
- Pass: Close delay is suppressed while the pointer is down on the hover card content (Radix
  `isPointerDownOnContentRef` outcome).
- Pass: Text selection containment (`hasSelectionRef`) is modeled by tracking the active non-empty
  text selection at the window runtime level and suppressing close while a selection exists in the
  hover card overlay root.
- Note: The Radix Vega web golden keeps hover-card content mounted on hover-out while flipping
  `data-state` from `open` to `closed`. Fret mirrors this outcome via presence/motion (content can
  remain mounted during close), and the state gate asserts `open=false` after hover-out while the
  content remains present.

## Conformance gates

- `ecosystem/fret-ui-shadcn/tests/radix_web_overlay_geometry.rs` validates hover-card placement
  (popper gap + cross-axis delta) against the Radix Vega web golden
  (`goldens/radix-web/v4/radix-vega/hover-card-example.hover-card.hover.light.json`).
- `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs` validates hover open/close
  outcomes against the Radix Vega web golden timeline.
- Run: `cargo nextest run -p fret-ui-shadcn --test radix_web_overlay_geometry`
