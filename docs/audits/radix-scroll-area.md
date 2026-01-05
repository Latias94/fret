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
- Partial: `type` behavior is best-effort:
  - `always` is supported.
  - `hover` is supported via a hover-region gate in shadcn recipes.
  - `auto` and `scroll` are not modeled yet (no overflow measurement / scroll-hide delay state).
- Missing: Separate X/Y scrollbars, thumb sizing parity, and corner rendering.

## Follow-ups (recommended)

- Add a headless scroll-area state machine to support `scrollHideDelay` and "show while scrolling".
- Add horizontal scrollbar + corner support to match Radix's full surface.

