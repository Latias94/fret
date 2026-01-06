# Radix Primitives Audit — Scroll Area

This audit compares Fret's Radix-aligned scroll-area substrate against the upstream Radix
`@radix-ui/react-scroll-area` primitive implementation pinned in `repo-ref/primitives`.

## Upstream references (source of truth)

- Implementation: `repo-ref/primitives/packages/react/scroll-area/src/scroll-area.tsx`
- Public exports: `repo-ref/primitives/packages/react/scroll-area/src/index.ts`

Key upstream concepts:

- `ScrollArea` root provides shared context and supports `type="auto|always|scroll|hover"` and
  `scrollHideDelay`.
- A dedicated `Viewport` element owns the native scroll container and hides native scrollbars.
- `Scrollbar` supports X/Y, thumb sizing, drag-to-scroll, wheel interception, and "hover/scroll"
  visibility modes.
- `Corner` renders a corner affordance when both scrollbars are present.

## Fret mapping

Fret does not use the DOM or native scroll containers. Scrolling is an explicit runtime element:

- Runtime mechanism: `crates/fret-ui` (`Scroll` + `Scrollbar`).
- Declarative composition helper: `ecosystem/fret-ui-kit/src/declarative/scroll.rs`.
- Radix-named primitive facade: `ecosystem/fret-ui-kit/src/primitives/scroll_area.rs`.

## Current parity notes

- Pass: Root-level `type` enum exists (`ScrollAreaType`).
- Pass: `type` behavior is modeled as outcomes:
  - `auto` shows scrollbars only when overflowing.
  - `hover` shows scrollbars while hovered and hides after `scrollHideDelay`.
  - `scroll` shows scrollbars while scrolling and hides after `scrollHideDelay` (after a short
    scroll-end debounce).
  - `always` shows scrollbars when overflowing.
- Pass: Supports separate X/Y scrollbars and a corner element when both overflow.
- Pass: Thumb sizing matches Radix (includes scrollbar padding and minimum `18px` thumb).

## Follow-ups (recommended)

- None at the moment.
