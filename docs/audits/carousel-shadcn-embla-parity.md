# Carousel parity: shadcn/ui v4 vs Embla vs Fret


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- `embla-carousel`: https://github.com/search?q=embla-carousel&type=repositories
- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This note records the current alignment status for the `carousel` component and highlights gaps
that should be addressed at the correct layer (mechanism vs policy/recipes).

## Sources of truth (local snapshots)

- shadcn/ui v4 docs: `repo-ref/ui/apps/v4/content/docs/components/radix/carousel.mdx`
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

## Mechanism prerequisites for Embla-like drag (Fret-specific)

Embla’s drag model is explicitly designed to coexist with interactive slide contents:

- A pointer-down should still reach descendant buttons/pressables.
- Once movement exceeds `dragThreshold` (default `10`), the carousel “wins” the gesture and prevents
  click activation (Embla sets `preventClick = true` in `DragHandler.ts`).

For a custom-rendered UI runtime, matching this behavior requires two mechanism capabilities:

1) **Capture switching must cancel the previous capture target.** If a parent steals capture after
   threshold (gesture arena outcome), the child pressable must receive a cancel signal so it can
   clear pressed/drag state.
   - In-tree evidence: `crates/fret-ui/src/tree/dispatch.rs` (capture switch emits `PointerCancel`)
   - Regression test: `crates/fret-ui/src/tree/tests/pointer_move_layers.rs`

2) **A parent gesture region must be able to observe pointer moves even when a descendant captured
   the pointer on down.** This is the only way to implement “armed → threshold → steal capture”
   without weakening global pressable semantics.
   - Mechanism: opt in to capture-phase pointer move dispatch via
     `PointerRegionProps.capture_phase_pointer_moves` (built on `Widget::event_capture` / ADR 0218).

Without (2), the carousel can either (a) capture immediately on down (breaking descendant clicks),
or (b) only allow dragging from non-interactive blank areas (breaking upstream expectations).

## Layering guidance (where fixes should live)

- `crates/fret-ui`: keep mechanism-only (pointer capture, hit-testing, semantics plumbing).
- `ecosystem/fret-ui-kit`: reusable headless policies (gesture arbitration patterns, roving focus, etc.).
- `ecosystem/fret-ui-shadcn`: shadcn-aligned recipes and component-specific interaction policies
  (carousel drag threshold, prev/next placement, docs-aligned sizing/spacing).

## Regression gates

- Layout parity: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/carousel.rs`
- Interaction: `ecosystem/fret-ui-shadcn/tests/carousel_pointer_passthrough.rs`
