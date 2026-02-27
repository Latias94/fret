# Carousel (Embla) Fearless Refactor v1 — TODO

Status: Draft

Goal: Align Fret's shadcn-style `Carousel` recipe with shadcn/ui v4 expectations while keeping the
core UI runtime mechanism-only. This workstream focuses on headless snap/contain semantics and
docs-aligned examples/diagnostics.

Non-goals (v1):

- Full Embla API surface (`setApi`, plugins, event subscriptions).
- Momentum physics / drag-free scrolling.
- Virtualization or lazy mounting of slides.

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

- [ ] CAR-010 Document the headless snap model contract (inputs, outputs, invariants).
  - Evidence target: add a short section to this workstream + code doc comments.
- [ ] CAR-020 Add unit tests for the snap model:
  - `slidesToScroll`: `auto` + `n`
  - `containScroll`: `false` / `keepSnaps` / `trimSnaps`
  - `align`: `start` / `center` / `end`
  - `contentSize <= viewSize + pixelTolerance` short-circuit

## P1 — Parity (docs-aligned outcomes)

- [ ] CAR-110 Ensure UI gallery examples mirror upstream widths and spacing recipes.
  - Gate: `tools/diag-scripts/ui-gallery-carousel-*-screenshot.json`
- [ ] CAR-120 Keep pointer/gesture arbitration aligned with Embla expectations:
  - descendant click should work
  - drag should steal capture only after threshold
  - Gate: `ecosystem/fret-ui-shadcn/tests/carousel_pointer_passthrough.rs`

## P2 — Integration (snap model used by the recipe)

- [ ] CAR-210 Wire `snap_model_1d` into `ecosystem/fret-ui-shadcn/src/carousel.rs`:
  - prev/next uses snap list instead of `index * extent`
  - `canScrollPrev/Next` matches Embla-style semantics (disabled until measurable)
- [ ] CAR-220 Add a minimal recipe-level option surface that stays policy-only:
  - `align` + `containScroll` + `slidesToScroll` (no Embla API exposure)
  - keep defaults matching shadcn/ui v4

## P3 — Evidence + guardrails

- [ ] CAR-310 Add/refresh diagnostics scripts for reproducible regressions.
- [ ] CAR-320 Update `docs/audits/carousel-shadcn-embla-parity.md` with new evidence anchors.
- [ ] CAR-330 Run layering checks if any cross-crate refactors are required.

