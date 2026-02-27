# TODO — Carousel Embla fearless refactor (v1)

This TODO list is ordered for landability: add gates first, then swap internals.

## 0) Pre-flight (no behavior changes)

- [ ] Confirm parity priorities and acceptance criteria in `carousel-embla-fearless-refactor-v1.md`.
- [ ] Decide engine location:
  - Preferred: `ecosystem/fret-ui-headless` (pure + reusable).
  - Alternate: `ecosystem/fret-ui-kit` (if we need kit-level policy helpers).

## 1) Gates first (fearless foundation)

### 1.1 Headless unit tests

- [ ] Add `snap_model` tests (variable slide sizes + gap).
- [ ] Add `align=start|center|end` tests.
- [ ] Add `containScroll=trimSnaps` tests (edge clamping).
- [ ] Add `slidesToScroll` grouping tests (P1).

### 1.2 Web-vs-Fret geometry tests

- [ ] Add a new web golden that uses non-uniform slide sizes (or extend an existing one).
- [ ] Extend `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/carousel.rs` with:
  - [ ] variable slide sizes case
  - [ ] vertical constrained viewport case

### 1.3 Diag scripts (interaction)

- [ ] New script: “drag from inner button cancels activation”.
- [ ] New script: “touch cross-axis scroll lock does not start drag”.
- [ ] Ensure scripts use stable `test_id` and (when needed) fixed frame delta.

## 2) Engine scaffolding (behind toggle)

- [ ] Introduce `CarouselSnapModel` (pure) + unit tests.
- [ ] Introduce `CarouselEngine` state update API:
  - Inputs: measured slide geometry, pointer deltas, options.
  - Outputs: offset, selected snap, canPrev/canNext.
- [ ] Wire engine into `ecosystem/fret-ui-shadcn::Carousel` behind a private “v1” path.
- [ ] Switch UI gallery Carousel page to the v1 path (not default yet).

## 3) Parity increments (P0 → P1)

- [ ] Replace uniform `extent * index` snapping with geometry-derived snap list.
- [ ] Implement `align` semantics in snap model.
- [ ] Implement `containScroll=trimSnaps` clamping.
- [ ] Implement `slidesToScroll` (P1) and add gates.

## 4) Rollout + cleanup

- [ ] Flip default to engine-backed v1.
- [ ] Remove v0 code path once all gates are green.
- [ ] Add/refresh diag bundles for before/after comparisons (optional, but useful evidence).

## 5) Post-v1 backlog (P2)

- [ ] `loop` parity (slide looper).
- [ ] `dragFree` + scroll-body physics parity.
- [ ] API parity (`setApi`, event surfaces).

