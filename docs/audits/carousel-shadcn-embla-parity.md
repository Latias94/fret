# Carousel parity: shadcn/ui v4 vs Embla vs Fret

This note records the current alignment status for the `carousel` component and highlights gaps
that should be addressed at the correct layer (mechanism vs policy/recipes).

## Sources of truth (local snapshots)

- shadcn/ui v4 docs: `repo-ref/ui/apps/v4/content/docs/components/carousel.mdx`
- shadcn/ui v4 component: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/carousel.tsx`
- shadcn/ui v4 examples:
  - `repo-ref/ui/apps/v4/registry/new-york-v4/examples/carousel-demo.tsx`
  - `repo-ref/ui/apps/v4/registry/new-york-v4/examples/carousel-size.tsx`
  - `repo-ref/ui/apps/v4/registry/new-york-v4/examples/carousel-spacing.tsx`
  - `repo-ref/ui/apps/v4/registry/new-york-v4/examples/carousel-orientation.tsx`
  - `repo-ref/ui/apps/v4/registry/new-york-v4/examples/carousel-plugin.tsx`
  - `repo-ref/ui/apps/v4/registry/new-york-v4/examples/carousel-api.tsx`
- Embla options reference (defaults): `repo-ref/embla-carousel/packages/embla-carousel/src/components/Options.ts`

## Fret implementation (in-tree)

- Component: `ecosystem/fret-ui-shadcn/src/carousel.rs`
- UI gallery page: `apps/fret-ui-gallery/src/ui/pages/carousel.rs`
- Web-vs-Fret layout harness: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/carousel.rs`

## What we match today

- **Layout composition (docs)**: negative track start margin + per-item start padding matches the
  shadcn spacing recipe (`-ml-*` on content + `pl-*` on items).
- **Orientation**: vertical tracks stack items and rotate controls; keyboard mapping stays left/right
  even in vertical mode (matching shadcn/ui behavior).
- **Input correctness**: carousel dragging no longer swallows pointer-down events intended for
  interactive descendants (pressables/buttons).
- **Drag threshold**: we gate the start of a drag with a pixel threshold, similar to Embla’s
  `dragThreshold` (default is `10` in Embla).

## Known gaps vs upstream

### API surface

- Upstream supports `opts`, `plugins`, and `setApi` (Embla API instance). Fret currently exposes a
  deterministic “snap + buttons + swipe” surface only.
- No event hook surface (e.g. `select`, `reInit`) because there is no `setApi` equivalent yet.
- No carousel-internal “selected index” contract exposed to callers (required for slide counters).

### Behavior/physics

- Embla provides momentum, snapping physics, and options like `loop`, `align`, `containScroll`,
  `dragFree`, etc. Fret currently uses a fixed-tick settle animation and a single snap-per-item model.

### Accessibility semantics

- Upstream uses `role="region"` + `aria-roledescription="carousel"` and item semantics like
  `aria-roledescription="slide"`. Fret currently stamps group roles for diagnostics and automation;
  richer semantics should be considered if/when we expand the accessibility contract.

## Layering guidance (where fixes should live)

- `crates/fret-ui`: keep mechanism-only (pointer capture, hit-testing, semantics plumbing).
- `ecosystem/fret-ui-kit`: reusable headless policies (gesture arbitration patterns, roving focus, etc.).
- `ecosystem/fret-ui-shadcn`: shadcn-aligned recipes and component-specific interaction policies
  (carousel drag threshold, prev/next placement, docs-aligned sizing/spacing).

## Regression gates

- Layout parity: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/carousel.rs`
- Interaction: `ecosystem/fret-ui-shadcn/tests/carousel_pointer_passthrough.rs`

