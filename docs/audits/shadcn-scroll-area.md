# shadcn/ui v4 Audit — Scroll Area


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `ScrollArea` surface against the upstream shadcn/ui v4
docs and the `new-york-v4` registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page (Radix): `repo-ref/ui/apps/v4/content/docs/components/radix/scroll-area.mdx`
- Docs page (Base UI): `repo-ref/ui/apps/v4/content/docs/components/base/scroll-area.mdx`
- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/scroll-area.tsx`
- Registry demo: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/scroll-area-demo.tsx`
- Registry horizontal demo: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/scroll-area-horizontal-demo.tsx`
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
- Pass: The typed parts surface already covers the upstream `ScrollArea` + `ScrollBar` teaching
  story without adding an untyped arbitrary-children API. The UI Gallery docs lane should prefer
  `ScrollArea::new(...)` for the wrapper story and use `ScrollAreaRoot::new(...).scrollbar(...)`
  when the example needs explicit extra scrollbar composition.
- Note: Because both the compact builder and the Radix-shaped parts surface already exist, Fret
  does not need an additional generic `compose()` builder for this component right now.
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
- Pass: Viewport paints a focus-visible ring (`focus-visible:ring-[3px]`) via a focusable wrapper
  semantics node inside a focus-ring container (`decl_style::focus_ring`). This keeps the viewport
  input-transparent so touch-pan scrolling still targets the `Scroll` mechanism.

### Docs / teaching surface

- Pass: The UI Gallery page can now mirror the upstream docs flow first (`Demo`, `Usage`,
  `Horizontal`, `RTL`, `API Reference`) before introducing Fret-only follow-ups.
- Pass: The copyable docs lane teaches `ScrollArea::new(...)` instead of promoting the
  Fret-specific `scroll_area(...)` helper as the primary shadcn-aligned surface.
- Pass: The `Horizontal` docs example can stay copyable while exposing the `ScrollBar`
  vocabulary via the explicit typed parts lane.
- Note: Base UI's additional `Content` / `Thumb` headless parts remain informative references, but
  they do not need promoted shadcn-lane public wrappers in Fret today because the runtime owns the
  viewport content wrapper and thumb implementation details.

## Validation

- `cargo test -p fret-ui-shadcn --lib scroll_area`
