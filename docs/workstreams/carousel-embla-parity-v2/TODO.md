# Carousel Embla parity (v2) — TODO

Status: Draft (needs scope confirmation)

Goal: Deeper Embla alignment for Carousel beyond the shadcn/ui docs outcomes, while keeping Fret’s
layering contract intact (mechanism vs policy vs recipes).

This workstream focuses on **Embla engine semantics** (physics, loopers, in-view, reInit/select
events, breakpoints) and an **Embla-like API surface** that is usable in Rust without importing JS
or mirroring React patterns 1:1.

Upstream references (local snapshots):

- Embla options + defaults:
  - `repo-ref/embla-carousel/packages/embla-carousel/src/components/Options.ts`
- Drag semantics (preventClick, force/duration/friction shaping):
  - `repo-ref/embla-carousel/packages/embla-carousel/src/components/DragHandler.ts`
- Scroll body integrator:
  - `repo-ref/embla-carousel/packages/embla-carousel/src/components/ScrollBody.ts`
- Engine composition (loopers, slidesInView, slideFocus, reInit):
  - `repo-ref/embla-carousel/packages/embla-carousel/src/components/Engine.ts`

In-tree surfaces (current baseline):

- Recipe: `ecosystem/fret-ui-shadcn/src/carousel.rs`
- Headless snap model (v1): `ecosystem/fret-ui-headless/src/carousel.rs` (`snap_model_1d`)
- UI gallery: `apps/fret-ui-gallery/src/ui/pages/carousel.rs`
- Web-vs-Fret layout harness: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/carousel.rs`

Non-goals (v2):

- Perfect pixel parity with web DOM (that’s covered by the existing web-golden harness).
- Reproducing React-only composition patterns (we target Rust ergonomics first).

---

## P0 — Contracts (hard-to-change)

- [x] CAR2-010 Decide and document **Embla option semantics** vs existing Fret shadcn options.
  - Key point: Embla `duration` is a numeric integrator parameter (not a `Duration` in ms).
  - Deliverable: `docs/workstreams/carousel-embla-parity-v2/contracts.md`
  - Evidence: `docs/workstreams/carousel-embla-parity-v2/contracts.md`
- [ ] CAR2-020 ADR: `CarouselApi` surface in Rust (methods + events + lifetimes).
  - Deliverable: `docs/adr/xxxx-carousel-api-surface.md`
- [ ] CAR2-030 ADR: Scroll physics determinism + reduced-motion behavior.
  - Deliverable: `docs/adr/xxxx-carousel-scroll-physics.md`
- [ ] CAR2-040 ADR: Seamless loop engine semantics (if in scope).
  - Deliverable: `docs/adr/xxxx-carousel-loop-engine.md`

---

## P1 — Engine parity (headless)

### Scroll body + animator

- [x] CAR2-110 Implement an Embla-style `ScrollBody` integrator (location/target/velocity/direction).
  - Evidence: `ecosystem/fret-ui-headless/src/embla/scroll_body.rs`
- [x] CAR2-120 Implement friction/duration shaping from `DragHandler`:
  - baseDuration, forceBoost, forceFactor, baseFriction adjustments
  - Evidence:
    - `ecosystem/fret-ui-headless/src/embla/drag_release.rs`
    - `ecosystem/fret-ui-headless/src/embla/engine.rs` (`Engine::on_drag_release`)
- [x] CAR2-125 Port core targeting helpers (snap selection + limits) needed by the engine.
  - Evidence:
    - `ecosystem/fret-ui-headless/src/embla/scroll_target.rs`
    - `ecosystem/fret-ui-headless/src/embla/scroll_limit.rs`
- [x] CAR2-126 Port edge constraint helper (`ScrollBounds`) and apply it during engine ticks.
  - Evidence:
    - `ecosystem/fret-ui-headless/src/embla/scroll_bounds.rs`
    - `ecosystem/fret-ui-headless/src/embla/engine.rs` (`Engine::tick`)
- [x] CAR2-127 Apply `ScrollBounds` while dragging (pointerDown=true) for Embla-like edge friction.
  - Evidence: `ecosystem/fret-ui-shadcn/src/carousel.rs` (pointer move uses `Engine::constrain_bounds`)
- [x] CAR2-130 Define edge constraints behavior (contain/trim/keep) with physics applied.
  - Evidence:
    - `ecosystem/fret-ui-headless/src/embla/scroll_contain.rs` (ported `ScrollContain`)
    - `ecosystem/fret-ui-headless/src/carousel.rs` (`snap_model_1d` already matches Embla fixtures)
    - `ecosystem/fret-ui-headless/src/embla/scroll_bounds.rs` (edge friction + pull-back)

### Loopers

- [ ] CAR2-140 Implement `loop=true` as **seamless looping** (not selection wrap).
  - Requires scroll looper + slide looper + translate wrapping semantics.

### Slides in view

- [ ] CAR2-150 Implement `slidesInView` with `inViewThreshold` + `inViewMargin`.
  - Provide both: current in-view set and “changed since last frame” signals.

### ReInit + resize + slide changes

- [ ] CAR2-160 Implement `reInit` event emission when geometry/options change.
  - Implemented (internal): headless `Engine::reinit` and shadcn recipe wiring on snap/viewport
    changes.
  - MVP: observable via monotonic generation counters published in `CarouselApiSnapshot`
    (`reinit_generation` / `select_generation`).
  - Missing: a full event subscription API surface (`on_reinit` / `on_select`) on a stable
    `CarouselApi` handle.
  - Evidence:
    - `ecosystem/fret-ui-headless/src/embla/engine.rs` (`Engine::reinit`)
    - `ecosystem/fret-ui-shadcn/src/carousel.rs` (calls `engine.reinit(...)` when snaps/viewport change)
- [ ] CAR2-170 Implement `resize` handling semantics (throttling + stable “reInit once” contract).
  - Partial: re-init is triggered when `viewSize`/snaps/max offset change; no explicit throttling
    contract yet.
  - Gate (MVP): `tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-demo-reinit-resize-gate.json`
- [ ] CAR2-180 Implement `slideChanges` semantics (detect add/remove/reorder in retained tree).

---

## P2 — API surface parity (recipe-level, Rust-native)

- [ ] CAR2-210 Provide a `CarouselApi` handle with:
  - `scrollPrev/scrollNext/scrollTo(index)`
  - `selectedScrollSnap`
  - `scrollSnapList`
  - `canScrollPrev/canScrollNext`
  - `slidesInView` (if implemented)
- [ ] CAR2-220 Provide an event subscription surface:
  - `on_select` and `on_reinit` (at least)
  - make it usable without storing arbitrary closures inside models
- [ ] CAR2-230 Align shadcn `setApi` example ergonomics in Rust (state + effect-like updates).
  - Workstream design note (draft): `docs/workstreams/carousel-embla-parity-v2/api-and-events.md`

---

## P3 — Breakpoints / responsive options

- [ ] CAR2-310 Add a breakpoint evaluation mechanism (Rust-native; no CSS media query parsing).
  - Option A: explicit `Vec<(min_width_px, opts_override)>`
  - Option B: container query integration if already present in `fret-ui-kit`
- [ ] CAR2-320 Regression gates for breakpoint changes (diag + tests).

---

## P4 — Focus semantics

- [ ] CAR2-410 Implement Embla-like `focus` behavior:
  - focusing a slide (or focus entering slide) scrolls it into view
  - keyboard navigation remains predictable with roving focus policies
- [ ] CAR2-420 A11y: role/roledescription + slide semantics parity audit.

---

## P5 — Evidence + gates

- [ ] CAR2-510 Add targeted `nextest` tests for the engine:
  - integrator stability, velocity decay, settle thresholds
  - loop wrapping invariants
  - slidesInView thresholds + margins
- [ ] CAR2-520 Add `fretboard diag` scripts for:
  - inertial swipe (touch + mouse)
  - loop visual continuity
  - focus-in triggers scroll-to-view
