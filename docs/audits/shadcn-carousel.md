# shadcn/ui v4 Audit — Carousel

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui
- Embla Carousel: https://www.embla-carousel.com/

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Carousel` against the upstream shadcn/ui v4 docs and
Embla-inspired authoring outcomes.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/radix/carousel.mdx`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/carousel.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/carousel.rs`
- Copyable usage snippet: `apps/fret-ui-gallery/src/ui/snippets/carousel/usage.rs`
- Diagnostics suites: `tools/diag-scripts/suites/ui-gallery-carousel-docs-parity/suite.json`,
  `tools/diag-scripts/suites/ui-gallery-carousel-embla-engine/suite.json`

## Audit checklist

### Authoring surface

- Pass: `Carousel::new(items)` / `Carousel::items(...)` already cover the compact builder path.
- Pass: `Carousel::into_element_parts(...)` plus `CarouselContent`, `CarouselItem`, `CarouselPrevious`, and `CarouselNext` expose the upstream-shaped parts surface for copyable examples.
- Pass: `opts(...)`, `orientation(...)`, spacing helpers, responsive item breakpoints, and plugin hooks cover the important shadcn + Embla recipe outcomes.
- Pass: because Fret already exposes both the compact builder and the parts authoring surface, it does not need an additional generic `compose()` builder here.

### Layout & behavior parity

- Pass: responsive item sizing is representable via `CarouselItem::viewport_layout_breakpoint(...)`, covering shadcn `md:basis-*` / `lg:basis-*` outcomes.
- Pass: spacing parity is modeled through `track_start_neg_margin` + `item_padding_start`, matching the upstream `-ml-*` + `pl-*` recipe.
- Pass: orientation, loop, options, events, API snapshot/handle, and autoplay/wheel plugins are already covered by the existing gallery surface.

### Gallery / docs parity

- Pass: the gallery now mirrors the upstream docs structure first (`Demo` / `About` / `Usage` / `Examples` / `Options` / `API` / `Events` / `Plugins` / `RTL` / `API Reference`), while keeping engine/diagnostics follow-ups explicit (loop downgrade, focus watch, duration, expandable).

### Defer rationale

- Pass: this surface has already been audited enough to show no obvious shadcn-facing mechanism or public-surface gap.
- Pass: status remains `Defer` because carousel is not currently editor-critical for Fret's near-term core/ecosystem priorities.
- Pass: follow-up work should only resume when a concrete engine, input, or layout regression appears.

## Validation

- `cargo check -p fret-ui-gallery --message-format short`
