# shadcn/ui v4 Audit — Scroll Area

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

- Partial: Fret exposes a compact builder (`ScrollArea::new(children)`) rather than the full
  `Root/Viewport/Scrollbar/Thumb/Corner` primitive split.
- Pass: Supports passing a `ScrollHandle` when consumers need programmatic scrolling.

### Scrollbar visibility (Radix `type`)

- Pass: Supports Radix `type="auto|always|scroll|hover"` outcomes via the primitives facade.
- Pass: Models delayed hide via `scrollHideDelay` (Fret exposes this as `scroll_hide_delay_ticks`).

### Visual parity (new-york)

- Partial: Uses theme `scrollbar.*` tokens for thumb colors; does not currently match the exact
  `bg-border` + `rounded-full` thumb skin from the registry wrapper.

## Validation

- `cargo test -p fret-ui-shadcn --lib collapsible` (for interaction scaffolding patterns)
- `cargo test -p fret-ui-shadcn --lib scroll_area`
