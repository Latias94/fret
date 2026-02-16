# Radix Primitives Audit — Scroll Area


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Radix UI Primitives: https://github.com/radix-ui/primitives

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
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
- Pass: Viewport content has a minimum width equal to the viewport width, matching Radix's
  `ScrollAreaViewport` content wrapper (`minWidth: '100%'`). This prevents `w-full` / percent-style
  descendants from collapsing when the scroll axis is probed with `MaxContent` during intrinsic
  measurement. In Fret, this is enforced by clamping the scroll content bounds to at least the
  viewport bounds during layout (`crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs`).
- Pass: Scroll axes are gated by mounted scrollbars (Radix `overflowX/overflowY` only enable when
  the corresponding `ScrollAreaScrollbar` is present). In Fret's shadcn/Radix-shaped surface, the
  underlying `Scroll` axis is derived from the requested scrollbar orientations, so horizontal
  scrolling is disabled by default unless a horizontal scrollbar is mounted.

## Follow-ups (recommended)

- None at the moment.
