# Carousel (Embla) Fearless Refactor v1 — TODO

Status: In progress (tracked + gated)

Goal: Align Fret's shadcn-style `Carousel` recipe with shadcn/ui v4 expectations while keeping the
core UI runtime mechanism-only. This workstream focuses on headless snap/contain semantics and
docs-aligned examples/diagnostics.

Non-goals (v1):

- Full Embla API surface (`setApi`, plugins, event subscriptions).
- Full Embla plugin surface (plugin registry, events, arbitrary plugin chaining).
- Momentum physics / inertial drag-free scrolling.
- Virtualization or lazy mounting of slides.

Note: v1 *does* include a small, deterministic "API snapshot" surface and a recipe-level autoplay
policy surface to align with shadcn docs examples without importing Embla's imperative API.
It also includes best-effort parity for a small subset of Embla options (e.g. `duration`,
`skipSnaps`, `dragFree`, and a non-seamless `loop` selection wrap) without implementing Embla's full
physics or loop engine.

Upstream references (local snapshots):

- shadcn docs: `repo-ref/ui/apps/v4/content/docs/components/carousel.mdx`
- shadcn component: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/carousel.tsx`
- Embla core: `repo-ref/embla-carousel/packages/embla-carousel/src/components/*`

In-tree surfaces:

- Recipe: `ecosystem/fret-ui-shadcn/src/carousel.rs`
- Headless helpers: `ecosystem/fret-ui-headless/src/carousel.rs`
- UI gallery page: `apps/fret-ui-gallery/src/ui/pages/carousel.rs`
- Web-vs-Fret layout harness: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/carousel.rs`

---

## P0 — Lock contracts (hard-to-change behavior)

- [x] CAR-010 Document the headless snap model contract (inputs, outputs, invariants).
  - Evidence:
    - `docs/workstreams/carousel-embla-fearless-refactor-v1/snap-model-contract.md`
    - `ecosystem/fret-ui-headless/src/carousel.rs` (`snap_model_1d` rustdoc)
- [x] CAR-020 Add unit tests for the snap model:
  - `slidesToScroll`: `auto` + `n`
  - `containScroll`: `false` / `keepSnaps` / `trimSnaps`
  - `align`: `start` / `center` / `end`
  - `contentSize <= viewSize + pixelTolerance` short-circuit
  - Evidence: `ecosystem/fret-ui-headless/src/carousel.rs` tests:
    - `snap_model_short_circuits_when_content_fits_view_with_tolerance`
    - `snap_model_fixed_slides_to_scroll_groups_slides_by_n`
    - `snap_model_auto_slides_to_scroll_groups_by_view_size`
    - `snap_model_contain_scroll_{none,keep_snaps,trim_snaps}_...`

## P1 — Parity (docs-aligned outcomes)

- [x] CAR-110 Ensure UI gallery examples mirror upstream widths and spacing recipes.
  - Evidence: `apps/fret-ui-gallery/src/ui/pages/carousel.rs`
  - Gate: `tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-*-screenshot.json`
- [x] CAR-130 Support shadcn docs "Plugins / autoplay" outcome without exposing Embla plugins.
  - Evidence:
    - `ecosystem/fret-ui-shadcn/src/carousel.rs` (`CarouselAutoplayConfig`, `Carousel::autoplay`)
    - `apps/fret-ui-gallery/src/ui/pages/carousel.rs` ("Plugin (Autoplay)" section)
  - Gate:
    - `tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-plugin-autoplay-pixels-changed.json`
- [x] CAR-120 Keep pointer/gesture arbitration aligned with Embla expectations:
  - descendant click should work
  - drag should steal capture only after threshold
  - Gate: `ecosystem/fret-ui-shadcn/tests/carousel_pointer_passthrough.rs`

## P2 — Integration (snap model used by the recipe)

- [x] CAR-210 Wire `snap_model_1d` into `ecosystem/fret-ui-shadcn/src/carousel.rs`:
  - prev/next uses snap list instead of `index * extent`
  - `canScrollPrev/Next` matches Embla-style semantics (disabled until measurable)
- [x] CAR-215 Feed the snap model with *measured slide geometry* (per-item bounds), not a uniform
  extent approximation.
  - Evidence: `ecosystem/fret-ui-shadcn/src/carousel.rs` (snap input derived from item bounds)
- [x] CAR-220 Add a minimal recipe-level option surface that stays policy-only:
  - `align` + `containScroll` + `slidesToScroll` + `duration` + `skipSnaps` + `dragFree` + `loop`
    (no Embla API exposure)
  - Evidence: `ecosystem/fret-ui-shadcn/src/carousel.rs`, `ecosystem/fret-ui-shadcn/src/lib.rs`
  - Note: upstream examples mix defaults + overrides:
    - `carousel-demo` / `carousel-spacing`: use Embla defaults (no `opts`)
    - `carousel-size` / `carousel-orientation`: use `opts={{ align: "start" }}`
  - Note: a deterministic API snapshot surface exists to support shadcn "API" examples without
    exposing Embla's imperative API (`CarouselApiSnapshot`).

## P3 — Evidence + guardrails

- [x] CAR-310 Add/refresh diagnostics scripts for reproducible regressions.
  - Evidence: `tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-*.json`
    - `tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-demo-screenshot.json`
    - `tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-api-screenshot.json`
    - `tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-basic-screenshot.json`
    - `tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-sizes-screenshot.json`
    - `tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-spacing-screenshot.json`
    - `tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-orientation-vertical-screenshot.json`
    - `tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-expandable-screenshot.json`
    - `tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-plugin-autoplay-pixels-changed.json`
- [x] CAR-320 Update `docs/audits/carousel-shadcn-embla-parity.md` with new evidence anchors.
  - Evidence: `docs/audits/carousel-shadcn-embla-parity.md`
- [x] CAR-330 Run layering checks if any cross-crate refactors are required.
  - Evidence: `python3 tools/check_layering.py`

## P4 — Shared snap utilities (cross-component, policy-only)

This is intentionally *not* "one snap model to rule them all". The goal is to share the smallest
headless helpers (e.g. nearest snap selection) across components that have snap-like behavior while
keeping their higher-level semantics separate (Carousel vs Drawer vs Slider).

- [x] CAR-410 Inventory snap-like behaviors in-tree and classify them:
  - Scroll/track snaps (Embla-like): Carousel
  - Drag-settle snap points (sheet-like): Drawer
  - Quantized value snaps (step/ticks): Slider / progress-like controls
  - Pixel snapping (rendering): `snap_to_device_pixels` (out of scope)
  - Evidence: `docs/workstreams/carousel-embla-fearless-refactor-v1/snap-inventory.md`
- [x] CAR-420 Add a tiny headless snap-point helper surface (if duplication persists):
  - candidates: `nearest_point`, `next_prev_point`, `projected_release_target`
  - keep it independent of `fret-dnd` and UI runtime types
  - Evidence:
    - `ecosystem/fret-ui-headless/src/snap_points.rs`
    - `ecosystem/fret-ui-shadcn/src/drawer.rs` (nearest snap selection)
- [x] CAR-430 Decide how Carousel drag and `fret-dnd` sensors should arbitrate pointer capture.
  - scope: policy only (likely `fret-ui-kit::dnd` sensor config + recipe opt-outs)
  - references: ADR 0149/0150/0151/0157
  - Evidence:
    - `docs/workstreams/carousel-embla-fearless-refactor-v1/dnd-arbitration.md`
    - `ecosystem/fret-ui-kit/src/dnd/controller.rs` (`pointer_is_tracking_any_sensor`)
    - `ecosystem/fret-ui-shadcn/src/carousel.rs` (skip swipe when DnD sensor tracks the pointer)
    - `apps/fret-ui-gallery/src/ui/pages/carousel.rs` (demo handle wiring)
    - `tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-demo-dnd-handle-gate.json`
    - `tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-demo-dnd-long-press-gate.json`
