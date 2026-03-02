# Carousel Embla parity (v2) — Milestones

This document defines “reviewable slices” for deeper Embla alignment.

## M0 — Contracts locked

Deliverables:

- `contracts.md` (Embla option semantics vs Fret mapping)
- `api-and-events.md` (Rust-native API + event semantics direction)
- (Optional later) Promote stable, hard-to-change pieces into ADRs only if/when these surfaces must
  be treated as long-lived contracts outside `ecosystem/*`.

Evidence:

- Links to the upstream source files in `repo-ref/embla-carousel/...`
- Evidence anchors to intended in-tree code locations

## M1 — Physics MVP (inertia exists)

Outcome:

- Releasing a drag can continue motion (velocity-driven) before snapping/settling.

Deliverables:

- Headless integrator (`ScrollBody`-like) with:
  - location/target/velocity
  - friction + duration shaping (at least for release)
- Recipe wiring that consumes engine output to position the track.

Gates:

- `nextest` unit test: velocity decays to ~0 and settles within bounded frames.
- `diag` script: swipe → post-release pixel change continues for N frames.
- `diag` script: fast vs slow duration settles at different speeds
  (`tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-duration-fast-vs-slow-settling-gate.json`).

## M2 — API + events (select/reInit)

Outcome:

- A Rust-native `CarouselApi` handle supports:
  - `scrollPrev/scrollNext/scrollTo`
  - `selectedScrollSnap`, `scrollSnapList`
  - `canScrollPrev/Next`
- `select` and `reInit` are observable and reliable.

Gates:

- `nextest` test: `select` fires exactly once per index transition.
- UI gallery: counter updates on swipe/buttons.
- Diag gate: resize during engine-driven motion does not panic and content remains visible
  (`tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-demo-reinit-resize-gate.json`).

## M3 — Seamless loop (loop=true)

Outcome:

- `loop=true` is no longer “wrap selection”; it is a continuous loop engine.

Follow-ups (deeper parity):

- Port Embla `SlideLooper` “gap fitting” semantics for non-uniform slide sizes and stable recycling.
- Implement Embla `canLoop` downgrade behavior (loop requested, but disabled when content cannot loop).

Risks / notes:

- Must avoid duplicate semantics/test ids.
- Must keep hit-testing deterministic (looped clones can’t fight for clicks).

Gates:

- `diag` script: repeated swipes never clamp at ends; continuity maintained.

## M4 — SlidesInView + focus + breakpoints

Outcome:

- `slidesInView` parity with `inViewThreshold` + `inViewMargin`.
- Focus semantics (focus entering a slide scrolls it into view) if in scope.
- Breakpoint option evaluation (Rust-native).

Gates:

- Unit tests for threshold/margin matrices.
- Diag scripts for focus-in scroll-to-view and breakpoint flips.
