# Carousel Embla parity (v2) — Plan

This plan aims to land deeper Embla alignment incrementally, with early regression gates and clear
layer ownership. It assumes v1 already delivered docs-aligned shadcn examples and a deterministic
snap model.

## Guiding principles

- Keep `crates/fret-ui` mechanism-only.
- Keep Embla-like physics/math in a **headless engine** (no UI types, no renderer types).
- Keep shadcn recipe (`ecosystem/fret-ui-shadcn`) as composition + tokens + ergonomic builder API.
- Prefer evidence-first: each behavior change gets a minimal gate (test and/or diag script).

## Step 0 — Scope confirmation (1–2 days)

- Decide which Embla subsystems are v2 requirements:
  - API + events (`setApi`, `select`, `reInit`)
  - physics/inertia (`ScrollBody`, friction/duration shaping)
  - seamless loopers (`loop=true`)
  - slidesInView (threshold/margin)
  - breakpoints
  - focus semantics
- Write down the non-goals explicitly so review stays tractable.

## Step 1 — Contracts + workstream design notes (2–5 days)

- Lock `contracts.md` (observable outcomes + option semantics).
- Update `api-and-events.md` with a Rust-native `CarouselApi` direction and the MVP event semantics.
- Update `docs/audits/carousel-shadcn-embla-parity.md` with a v2 parity matrix section.
- (Optional later) Promote stable, hard-to-change pieces into ADRs **only** if/when these surfaces
  must be treated as long-lived contracts outside `ecosystem/*`.

Definition of done:

- Each doc has 1–3 evidence anchors (expected code locations + gating approach).

## Step 2 — Headless engine MVP (5–10 days)

- Implement a minimal Embla-style engine surface:
  - measured slide rects + container rect
  - options
  - per-frame `tick(dt)` returning `{ location, target, velocity, selected_index, snap_list }`
- Port `ScrollBody` semantics first (integrator + settle threshold).
- Wire the existing shadcn Carousel to use the engine behind a feature flag or an internal toggle.

Gates:

- `nextest` unit tests for integrator invariants.
- A diag script that asserts “drag produces continued motion after release” (inertia presence).
- A diag script that asserts `embla_duration` affects settling speed (fast vs slow duration)
  (`tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-duration-fast-vs-slow-settling-gate.json`).
- A diag script that resizes during engine-driven motion without panics/hangs
  (`tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-demo-reinit-resize-gate.json`).

Optional suite (recommended for Embla engine regressions):

- `ui-gallery-carousel-embla-engine` runs the screenshot-based, engine-enabled Carousel gates.
  - Note: Embla engine is enabled by default; set `FRET_DEBUG_CAROUSEL_EMBLA_ENGINE=0` to force-disable.
  - Run:
    - `cargo run -p fretboard -- diag suite ui-gallery-carousel-embla-engine --launch -- cargo run -p fret-ui-gallery --release`

## Step 3 — Events + API handle (3–6 days)

- Add a Rust-native `CarouselApi` handle:
  - methods route through models/effects (no direct interior mutability required by callers)
  - events expose “version bumps” or an explicit event queue that can be consumed safely in render
- Update UI gallery API demo to use the new handle (keep the old snapshot demo until parity is
  proven, then migrate).

Gates:

- `nextest` test verifying `select` fires exactly once per index change.
- Diag script verifying `Slide N of M` updates on swipe and on next/prev.

## Step 4 — Seamless loopers (optional, 5–12 days)

- Implement scroll looper + slide looper translation wrapping.
- Ensure hit-testing + semantics remain stable (no duplicate `test_id`).
- Deepen parity by porting Embla `SlideLooper` gap-fitting behavior and `canLoop` downgrade semantics.

Gates:

- Loop continuity diag: N swipes never reaches an “end” state; visual continuity maintained.
  - Optional additional gate: loop requested but `canLoop=false` behaves like loop disabled
    (`tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-loop-downgrade-cannot-loop-gate.json`).

## Step 5 — Slides in view + focus + breakpoints (optional, 5–10 days)

- Implement `slidesInView` with threshold/margin.
- Implement focus-driven scroll-to-view.
- Add breakpoint option evaluation (Rust-native).

Gates:

- `slidesInView` deterministic tests.
- Focus diag: focusing a slide scrolls it into view.
- Breakpoint diag: resizing window flips options once.

## Exit criteria

We call v2 “done” when:

- the engine is in use by the recipe by default (no feature flag),
- the API + event surfaces are stable enough to document,
- and the key parity claims are guarded by tests/scripts.
