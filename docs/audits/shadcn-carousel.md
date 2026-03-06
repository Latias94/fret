# shadcn/ui v4 Audit - Carousel

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui
- Embla Carousel: https://www.embla-carousel.com/

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Carousel` against the upstream shadcn/ui v4 docs and
Embla-inspired authoring outcomes.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/carousel.mdx`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/carousel.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/carousel.rs`
- Copyable usage snippet: `apps/fret-ui-gallery/src/ui/snippets/carousel/usage.rs`

## Audit checklist

### Authoring surface

- Pass: `Carousel::new(items)` / `Carousel::items(...)` already cover the compact builder path.
- Pass: `Carousel::into_element_parts(...)` plus `CarouselContent`, `CarouselItem`,
  `CarouselPrevious`, and `CarouselNext` expose the upstream-shaped parts surface for copyable examples.
- Pass: `opts(...)`, `orientation(...)`, spacing helpers, responsive item breakpoints, and plugin hooks
  cover the important shadcn + Embla recipe outcomes.
- Note: Because Fret already exposes both the compact builder and the parts authoring surface, it does
  not need an additional generic `compose()` builder here.

### Layout & behavior parity

- Pass: Responsive item sizing is representable via `CarouselItem::viewport_layout_breakpoint(...)`,
  covering shadcn `md:basis-*` / `lg:basis-*` outcomes.
- Pass: Spacing parity is modeled through `track_start_neg_margin` + `item_padding_start`, matching the
  upstream `-ml-*` + `pl-*` recipe.
- Pass: Orientation, loop, options, events, API snapshot/handle, and autoplay/wheel plugins are already
  covered by the existing gallery surface.

## Conclusion

- Result: This component does not currently indicate a missing mechanism-layer gap in the shadcn-facing surface.
- Result: The main missing piece for docs parity was a concise gallery `Usage` example.
- Result: Composable authoring is already supported; follow-up work should focus on concrete engine/interaction
  regressions only if they appear.

## Validation

- `cargo check -p fret-ui-gallery --message-format short`
