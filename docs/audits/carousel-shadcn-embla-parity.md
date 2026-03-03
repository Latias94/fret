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
- Headless snap model: `ecosystem/fret-ui-headless/src/carousel.rs` (`snap_model_1d`)
- Snap contract (workstream): `docs/workstreams/carousel-embla-fearless-refactor-v1/snap-model-contract.md`
- UI gallery page: `apps/fret-ui-gallery/src/ui/pages/carousel.rs`
- Web-vs-Fret layout harness: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/carousel.rs`

## What we match today

- **Layout composition (docs)**: negative track start margin + per-item start padding matches the
  shadcn spacing recipe (`-ml-*` on content + `pl-*` on items).
- **Snap semantics (Embla-aligned, headless)**: deterministic snap model with Embla-like vocabulary:
  `align` (start/center/end), `containScroll` (none/keepSnaps/trimSnaps), `slidesToScroll`
  (fixed/auto), and `pixelTolerance` edge handling.
- **Measured slide geometry (recipe)**: the shadcn recipe derives `CarouselSlide1D` inputs from each
  rendered slide's measured bounds (with a first-frame uniform fallback) before calling
  `snap_model_1d`.
  - Evidence: `ecosystem/fret-ui-shadcn/src/carousel.rs` (snap model generation)
- **Recipe-level `opts` (policy-only)**: shadcn-style `CarouselOptions` maps the docs examples:
  `carousel-size` / `carousel-orientation` use `align: start`, while other examples rely on defaults.
- **Orientation**: vertical tracks stack items and rotate controls; keyboard mapping stays left/right
  even in vertical mode (matching shadcn/ui behavior).
- **Input correctness**: carousel dragging no longer swallows pointer-down events intended for
  interactive descendants (pressables/buttons).
- **Drag threshold**: we gate the start of a drag with a pixel threshold, similar to Embla’s
  `dragThreshold` (default is `10` in Embla).

## Known gaps vs upstream

### API surface

- Upstream supports `opts`, `plugins`, and `setApi` (Embla API instance). Fret currently exposes a
  deterministic “snap + buttons + swipe” surface only; `opts` is supported only for snap model
  semantics (not for the full Embla options set).
- No callback subscription surface (e.g. Embla `api.on('select')` / `api.on('reInit')`) because there
  is no `setApi`-style handle yet.
- MVP event observability exists via monotonic generation counters published in
  `CarouselApiSnapshot` (`select_generation` / `reinit_generation`).
- No Embla-style imperative API surface. Fret exposes a small, deterministic snapshot surface
  (`CarouselApiSnapshot`) for slide counters and basic state.

### Behavior/physics

- Embla provides momentum + snapping physics (velocity, friction, edge constraints) and a seamless
  loop engine (scroll + slide loopers). Fret intentionally stays deterministic and mechanism-light:
  it uses a fixed-tick settle animation (no momentum) and a snap model derived from measured slide
  geometry.
- Fret does implement a *subset* of Embla options at the recipe/headless level (best-effort parity,
  not a 1:1 port): `align`, `containScroll`, `slidesToScroll`, `duration`, `skipSnaps`, `dragFree`,
  and a non-seamless `loop` selection wrap.

## Embla options parity matrix (best-effort, subset)

Source of truth: `repo-ref/embla-carousel/packages/embla-carousel/src/components/Options.ts`.

Legend:

- **Aligned**: same observable outcome for our supported surfaces.
- **Partial**: similar outcome, but missing physics / edge cases / engine behavior.
- **Not implemented**: no corresponding surface yet (or intentionally out of scope for v1).

| Embla option | Default | Fret surface | Status | Notes |
| --- | --- | --- | --- | --- |
| `align` | `"center"` | `CarouselOptions.align` + `snap_model_1d` | **Aligned** | Snap positions match Embla fixtures for LTR. |
| `containScroll` | `"trimSnaps"` | `CarouselOptions.contain_scroll` + `snap_model_1d` | **Aligned** | `None/KeepSnaps/TrimSnaps` vocabulary. |
| `slidesToScroll` | `1` | `CarouselOptions.slides_to_scroll` + `snap_model_1d` | **Aligned** | Supports fixed and auto grouping. |
| `dragThreshold` | `10` | `CarouselDragConfig.drag_threshold_px` | **Aligned** | Threshold-arms then steals capture. |
| `duration` (ms) | `25` | `CarouselOptions.duration` | **Partial** | Mapped to deterministic settle frames; Embla varies duration based on force. |
| `skipSnaps` | `false` | `CarouselOptions.skip_snaps` | **Partial** | Implemented at release by choosing the closest snap (no momentum). |
| `dragFree` | `false` | `CarouselOptions.drag_free` | **Partial** | Settles to projected offset; no inertia scroll body. |
| `loop` | `false` | `CarouselOptions.loop_enabled` | **Partial** | Wraps prev/next/keys and release neighbor selection; **not** Embla's seamless loop engine. |
| `axis` | `"x"` | `CarouselOrientation` | **Partial** | Horizontal/vertical supported; not a generic axis + direction model. |
| `direction` | `"ltr"` | `CarouselOptions.direction` | **Partial** | Mirrors horizontal drag/key/control placement in RTL; does **not** reflow slide layout like CSS `direction` yet. |
| `startSnap` | `0` | `CarouselOptions.start_snap` | **Partial** | Applied once snaps are measurable (recipe derives snaps from geometry). |
| `draggable` | `true` | `CarouselOptions.draggable` | **Aligned** | Disables pointer dragging; buttons/keys remain active. |
| `resize` | `true` | (none) | **Not implemented** | Re-init semantics are implicit via layout passes; no explicit option. |
| `slideChanges` | `true` | (none) | **Not implemented** | No DOM mutation observer equivalent (not applicable). |
| `focus` | `true` | (none) | **Not implemented** | No SlideFocus parity contract yet. |
| `breakpoints` | `{}` | (none) | **Not implemented** | Use Fret container/viewport queries instead. |
| `inViewThreshold` | `0` | (none) | **Not implemented** | No SlidesInView parity surface yet. |
| `inViewMargin` | `"0px"` | (none) | **Not implemented** | No SlidesInView parity surface yet. |
| `ssr` | `[]` | (none) | **Not implemented** | Not applicable for our renderer. |

### Accessibility semantics

- Upstream uses `role="region"` + `aria-roledescription="carousel"` and item semantics like
  `aria-roledescription="slide"`. Fret models this via `SemanticsRole::Region` +
  `role_description="carousel"` on the root, and `SemanticsRole::Group` + `role_description="slide"`
  on items (labels remain present for diagnostics and automation).

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
- Headless snap contract: `ecosystem/fret-ui-headless/src/carousel.rs` tests (nextest)
- Interaction: `ecosystem/fret-ui-shadcn/tests/carousel_pointer_passthrough.rs`
- Diagnostics (native screenshots): `tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-*-screenshot.json`
