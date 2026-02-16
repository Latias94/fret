# shadcn/ui v4 Audit — Scroll Area


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `ScrollArea` surface against the upstream shadcn/ui v4
docs and the `new-york-v4` registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/scroll-area.mdx`
- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/scroll-area.tsx`
- Registry demo: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/scroll-area-demo.tsx`
- Underlying primitive: Radix `@radix-ui/react-scroll-area`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/scroll_area.rs`
- Radix-aligned primitives: `ecosystem/fret-ui-kit/src/primitives/scroll_area.rs`
- Runtime substrate: `crates/fret-ui` (`Scroll` + `Scrollbar`)

## Audit checklist

### Composition surface

- Pass: Exposes a composable `ScrollAreaRoot` / `ScrollAreaViewport` / `ScrollAreaScrollbar` /
  `ScrollAreaCorner` surface (Radix-shaped), while keeping the compact `ScrollArea::new(children)`
  builder for convenience.
- Pass: Supports passing a `ScrollHandle` when consumers need programmatic scrolling.
- Pass: Mirrors the Radix `Viewport` content minimum width behavior: the scroll content bounds are
  clamped to at least the viewport bounds so `w-full` descendants do not collapse under
  `probe_unbounded` layouts (see `docs/audits/radix-scroll-area.md`).
- Pass: Matches the shadcn v4 wrapper default: `ScrollArea::new(children)` mounts a vertical
  scrollbar only (no horizontal scrolling unless explicitly enabled via `axis(ScrollAxis::X|Both)`
  or by mounting a horizontal scrollbar on `ScrollAreaRoot`).

### Scrollbar visibility (Radix `type`)

- Pass: Supports Radix `type="auto|always|scroll|hover"` outcomes via the primitives facade.
- Pass: Models delayed hide via `scrollHideDelay` (Fret exposes this as `scroll_hide_delay_ticks`).
- Pass: Supports horizontal overflow with an X scrollbar and renders a corner element when both
  scrollbars are present.

### Visual parity (new-york)

- Pass: Scrollbar thumb styling matches the registry wrapper defaults (`bg-border` + `rounded-full`)
  via the runtime's rounded thumb paint and shadcn token mapping.

## Validation

- `cargo test -p fret-ui-shadcn --lib collapsible` (for interaction scaffolding patterns)
- `cargo test -p fret-ui-shadcn --lib scroll_area`
