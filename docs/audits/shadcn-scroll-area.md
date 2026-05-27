# shadcn/ui v4 Audit - Scroll Area


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `ScrollArea` surface against the current upstream
shadcn/ui v4 docs page, the `new-york-v4` registry source/examples, the tracked scroll-area web
goldens, and the existing Scroll Area gates.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/scroll-area.mdx`
- Component implementation: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/scroll-area.tsx`
- Example compositions: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/scroll-area-demo.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/scroll-area-horizontal-demo.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/select-scrollable.tsx`
- Upstream goldens: `goldens/shadcn-web/v4/new-york-v4/scroll-area-demo*.json`, `goldens/shadcn-web/v4/new-york-v4/scroll-area-horizontal-demo*.json`
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

### Gallery / docs parity

- Pass: The UI Gallery page mirrors the current upstream docs path first: `Demo`, `Usage`, and
  `Horizontal`.
- Pass: `RTL`, `API Reference`, `Compact helper`, `Nested scroll routing`, and diagnostics stay
  explicit Fret follow-ups instead of pretending to be current upstream Scroll Area headings.
- Pass: The copyable docs lane teaches `ScrollArea::new(...)` instead of promoting the
  Fret-specific `scroll_area(...)` helper as the primary shadcn-aligned surface.
- Pass: The `Horizontal` docs example can stay copyable while exposing the `ScrollBar`
  vocabulary via the explicit typed parts lane.
- Pass: Gallery source-policy tests keep visible text on shared roles, snippets on the default app
  surface, and diagnostics raw boundaries isolated to audited harness roots.
- Note: Radix's viewport content wrapper and thumb remain mechanism/runtime details in Fret; they
  do not need promoted public shadcn wrappers today.

## Validation

- `cargo nextest run -p fret-ui-shadcn --lib --status-level fail scroll_area`
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout --status-level fail web_vs_fret_layout_scroll_geometry_matches_web_fixtures`
- `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_scroll --status-level fail`
- `cargo nextest run -p fret-ui-shadcn --test radix_web_primitives_state --status-level fail radix_web_scroll_area_scroll_top_delta_matches_fret`
- `cargo nextest run -p fret-ui-gallery --test scroll_area_docs_surface --status-level fail`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app --status-level fail scroll_area`
- `Get-ChildItem -Path tools/diag-scripts/ui-gallery/scroll-area -Filter *.json | ForEach-Object { python -m json.tool $_.FullName | Out-Null }`
