# Carousel (Embla) Fearless Refactor v1 — Plan

Status: Complete (remaining work moved to v2)

This plan summarizes what is already landed and proposes the next, highest-leverage steps.

## What is already landed

- M1: Headless Embla-aligned snap model + tests
  - `ecosystem/fret-ui-headless/src/carousel.rs` (`snap_model_1d`)
  - Workstream contract: `docs/workstreams/carousel-embla-fearless-refactor-v1/snap-model-contract.md`
- M2: Recipe integration + minimal options surface (`align`, `containScroll`, `slidesToScroll`)
  - `ecosystem/fret-ui-shadcn/src/carousel.rs`
- M2.5: Measured slide geometry drives snap inputs (per-item bounds, first-frame fallback)
  - `ecosystem/fret-ui-shadcn/src/carousel.rs`
- M3: UI gallery parity with shadcn docs + deterministic geometry harness
  - `apps/fret-ui-gallery/src/ui/pages/carousel.rs`
  - `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/carousel.rs`
- M3.5: Docs parity extras (shadcn “API” + “Plugins” examples)
  - API snapshot surface: `ecosystem/fret-ui-shadcn/src/carousel.rs` (`CarouselApiSnapshot`)
  - Autoplay policy surface: `ecosystem/fret-ui-shadcn/src/carousel.rs` (`CarouselAutoplayConfig`)
  - UI gallery demo: `apps/fret-ui-gallery/src/ui/pages/carousel.rs` ("Plugin (Autoplay)")
  - Gate: `tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-plugin-autoplay-pixels-changed.json`
- M4: Motion alignment for `duration` (settle timeline driver)
  - `ecosystem/fret-ui-shadcn/src/carousel.rs` (duration-driven settle; reduced-motion aware)
  - `ecosystem/fret-ui-kit/src/declarative/transition.rs` (duration → 60Hz ticks + frame scaling)
- M4.5: Embla option parity (best-effort, policy-level)
  - `startSnap` + `draggable`: `ecosystem/fret-ui-shadcn/src/carousel.rs` (`CarouselOptions`)
  - `direction` (RTL): `ecosystem/fret-ui-headless/src/carousel.rs` (drag sign) + `ecosystem/fret-ui-shadcn/src/carousel.rs` (controls/keys)
- P4: Carousel × DnD pointer arbitration (mouse handle path)
  - Decision + notes: `docs/workstreams/carousel-embla-fearless-refactor-v1/dnd-arbitration.md`
  - Policy hook: `fret-ui-shadcn::Carousel` skips swiping while a DnD sensor tracks the pointer.
  - Gates:
    - `tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-demo-dnd-handle-gate.json`
    - `tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-demo-dnd-long-press-gate.json`

## Next steps (recommended order)

Further Embla parity work (API/events, loopers, slidesInView, breakpoints) is tracked under:

- `docs/workstreams/carousel-embla-parity-v2/`

### 1) Close remaining UI gallery drift (P2)

Goal: ensure the Carousel page demos match shadcn/ui docs composition outcomes (constraints first,
then styling), especially for the most visible mismatch reports.

Focus areas:

- Expandable demo: avoid unexpected text wrapping (usually missing `min-w-0` / `flex` constraints).
- Vertical orientation demo: verify track/item constraints match upstream intent.

Exit criteria:

- UI gallery matches shadcn docs for Expandable + Vertical demos.
- Existing carousel gates remain green (web-vs-fret layout + diag scripts).

### 2) Evidence hardening (P3)

Goal: turn the current alignment into stable, repeatable gates.

- Add/refresh diag scripts so we can catch regressions without relying on manual UI runs:
  - `ui-gallery-carousel-*-screenshot.json` set should cover Demo/Sizes/Spacing/Vertical/Expandable.
  - Autoplay should have a deterministic gate:
    - `ui-gallery-carousel-plugin-autoplay-pixels-changed.json` + `--check-pixels-changed ui-gallery-carousel-plugin`
  - Interaction gate: `tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-demo-dnd-handle-gate.json` (Carousel vs DnD arbitration)
- Add an audit note with evidence anchors (if the repo expects it):
  - `docs/audits/carousel-shadcn-embla-parity.md` (anchors: recipe + headless + tests + scripts).

Exit criteria:

- A minimal diag suite can be executed in one command and produces screenshots/bundles with stable
  `test_id` selectors.

### 3) Additional Embla options (P3/P4, selective)

Goal: keep extending option parity where it produces real recipe outcomes, without pulling in
momentum physics or a seamless loop engine.

Candidates to audit next:

- `watchDrag` / `watchSlides`-like re-init knobs (likely not applicable outside the DOM).
- `inViewThreshold` / in-view detection (only if a recipe/demo depends on it).
- `dragThreshold` mapping (if we want 1:1 default feel).

### 4) Shared snap utilities (P4, optional but likely valuable)

Goal: reduce duplicated “nearest snap point” math across components without conflating semantics.

Approach:

- Write a short inventory of snap-like behavior patterns:
  - Carousel: scroll snaps derived from geometry
  - Drawer: explicit snap points + fling projection + dismiss threshold
- If we find repeated helpers, factor them into `fret-ui-headless` as pure functions:
  - `nearest_point(points, value)` returning index + value
  - `next_prev_point(points, current, direction)` for keyboard/buttons

Exit criteria:

- At least one other component (likely Drawer) uses the shared helper without changing its public
  behavior.

### 4) Carousel vs DnD gesture arbitration (P4, later)

Goal: extend the current mouse-handle policy to touch-friendly recipes (long-press).

Constraints:

- Must remain policy-level (ecosystem), not `crates/fret-ui` mechanisms.
- DnD references:
  - ADR 0149: headless toolbox + UI integration registry
  - ADR 0150/0151: pointer identity and drag session routing keys
  - ADR 0157: contract surface

Potential policy direction:

- Carousel keeps its “threshold then steal capture” behavior.
- DnD sensors opt in only on long-press / higher threshold / handle-only regions, or when a modifier
  key is pressed, depending on the recipe.

Exit criteria:

- A documented policy choice and at least one regression gate proving no accidental capture
  starvation (e.g. carousel swipe still works; dnd drag still starts when intended).
