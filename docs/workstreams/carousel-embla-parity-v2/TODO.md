# Carousel Embla parity (v2) — TODO

Status: In progress (contracts locked; deeper parity ongoing)

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
- [x] CAR2-020 Workstream design: `CarouselApi` surface in Rust (methods + events + lifetimes).
  - Deliverable: `docs/workstreams/carousel-embla-parity-v2/api-and-events.md`
  - Note: promote to an ADR only if/when the surface becomes stable and/or must be treated as
    a long-lived contract outside `ecosystem/*`.
- [x] CAR2-030 Workstream design: scroll physics determinism + reduced-motion behavior (MVP).
  - Deliverable:
    - `docs/workstreams/carousel-embla-parity-v2/contracts.md` (fixed-step time model + reduced motion)
  - Evidence:
    - `ecosystem/fret-ui-shadcn/src/carousel.rs` (reduced-motion disables embla engine + instant settle)
    - Gate: `ecosystem/fret-ui-shadcn/tests/carousel_reduced_motion.rs`
  - Note: promote to an ADR only if/when the physics semantics become a stable public contract.
- [ ] CAR2-040 Workstream design: seamless loop engine semantics (if in scope).
  - Deliverable: `docs/workstreams/carousel-embla-parity-v2/contracts.md` (loop section) + gates
  - Note: promote to an ADR only if/when we commit to a stable, long-lived loop contract.

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

- [x] CAR2-140 Implement `loop=true` as **seamless looping** (not selection wrap) (MVP).
  - Implemented:
    - scroll/body wrap normalization (loop distance applied without resetting velocity)
    - per-slide translation recycling by `±content_size`
  - Evidence:
    - `ecosystem/fret-ui-headless/src/embla/engine.rs` (`Engine::normalize_loop_entities`)
    - `ecosystem/fret-ui-headless/src/embla/scroll_body.rs` (`ScrollBody::add_loop_distance`)
    - `ecosystem/fret-ui-headless/src/embla/slide_looper.rs`
    - `ecosystem/fret-ui-shadcn/src/carousel.rs` (enables embla engine for loop + per-slide `RenderTransform`)
  - Gates:
    - `cargo test -p fret-ui-headless` (loop + slide looper unit tests)
    - `tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-loop-continuity-touch-gate.json`

### Slides in view

- [x] CAR2-150 Implement `slidesInView` with `inViewThreshold` + `inViewMargin`.
  - Provide both: current in-view set and “changed since last frame” signals.
  - Evidence:
    - Headless tracker: `ecosystem/fret-ui-headless/src/embla/slides_in_view.rs`
    - Recipe wiring + snapshot model: `ecosystem/fret-ui-shadcn/src/carousel.rs`
  - Gate:
    - `ecosystem/fret-ui-shadcn/tests/carousel_slides_in_view_snapshot.rs`

### ReInit + resize + slide changes

- [x] CAR2-160 Implement `reInit` event emission when geometry/options change (MVP).
  - Implemented (internal): headless `Engine::reinit` and shadcn recipe wiring on snap/viewport
    changes.
  - MVP: observable via monotonic generation counters published in `CarouselApiSnapshot`
    (`reinit_generation` / `select_generation`).
  - Missing: a full event subscription API surface (`on_reinit` / `on_select`) on a stable
    `CarouselApi` handle.
  - Evidence:
    - `ecosystem/fret-ui-headless/src/embla/engine.rs` (`Engine::reinit`)
    - `ecosystem/fret-ui-shadcn/src/carousel.rs` (calls `engine.reinit(...)` when snaps/viewport change)
- [x] CAR2-170 Implement `resize` handling semantics (throttling + stable “reInit once” contract) (MVP).
  - Implemented: observable `reInit` is throttled and coalesced (see `api-and-events.md`).
  - Decision (v2 MVP): throttle observable `reInit` to “at most once per N frames” during continuous
    geometry churn (see `api-and-events.md`).
  - Gate: `ecosystem/fret-ui-shadcn/tests/carousel_api_generations.rs`
  - Gate (MVP): `tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-demo-reinit-resize-gate.json`
- [x] CAR2-180 Implement `slideChanges` semantics (detect add/remove/reorder in retained tree).
  - Evidence:
    - Content-id change detection: `ecosystem/fret-ui-shadcn/src/carousel.rs`
  - Gate:
    - `ecosystem/fret-ui-shadcn/tests/carousel_slide_changes_reinit.rs`

---

## P2 — API surface parity (recipe-level, Rust-native)

- [x] CAR2-210 Provide a `CarouselApi` handle with:
  - `scrollPrev/scrollNext/scrollTo(index)`
  - `selectedScrollSnap`
  - `scrollSnapList`
  - `canScrollPrev/canScrollNext`
  - `slidesInView` (if implemented)
-  Evidence:
  - `ecosystem/fret-ui-shadcn/src/carousel.rs` (`CarouselApi`)
  - Gate: `ecosystem/fret-ui-shadcn/tests/carousel_api_handle.rs`
- [x] CAR2-220 Provide an event subscription surface:
  - `on_select` and `on_reinit` (at least)
  - make it usable without storing arbitrary closures inside models
-  Evidence:
  - `ecosystem/fret-ui-shadcn/src/carousel.rs` (`CarouselEventCursor` + `events_since`)
  - UI gallery: `apps/fret-ui-gallery/src/ui/pages/carousel.rs` (API demo)
- [x] CAR2-230 Align shadcn `setApi` example ergonomics in Rust (state + effect-like updates).
  - Workstream design note: `docs/workstreams/carousel-embla-parity-v2/api-and-events.md`

---

## P3 — Breakpoints / responsive options

- [x] CAR2-310 Add a breakpoint evaluation mechanism (Rust-native; no CSS media query parsing).
  - Option A: explicit `Vec<(min_width_px, opts_override)>`
  - Option B: container query integration if already present in `fret-ui-kit`
  - Implemented: `CarouselOptionsPatch` + `CarouselBreakpoint` evaluated from the measured carousel
    viewport width (previous layout pass).
  - Evidence: `ecosystem/fret-ui-shadcn/src/carousel.rs`
- [x] CAR2-320 Regression gates for breakpoint changes (diag + tests).
  - Gate: `ecosystem/fret-ui-shadcn/tests/carousel_breakpoints.rs`

---

## P4 — Focus semantics

- [ ] CAR2-410 Implement Embla-like `focus` behavior:
  - focusing a slide (or focus entering slide) scrolls it into view
  - keyboard navigation remains predictable with roving focus policies
-  Evidence:
  - `ecosystem/fret-ui-shadcn/src/carousel.rs` (`watch_focus`, Tab watcher)
  - Gate: `ecosystem/fret-ui-shadcn/tests/carousel_focus_watch_tab_scrolls.rs`
- [x] CAR2-410 Implement Embla-like `focus` behavior (MVP).
- [x] CAR2-420 A11y: role/roledescription + slide semantics parity audit (with known gaps).
  - Note: we currently stamp role/label/orientation, but do not yet have a portable
    `aria-roledescription` equivalent in core semantics.
  - Evidence:
    - `ecosystem/fret-ui-shadcn/src/carousel.rs` (Region root + slide labels)
    - Gate: `ecosystem/fret-ui-shadcn/tests/carousel_a11y_semantics.rs`

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
