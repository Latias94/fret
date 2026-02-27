# Carousel (Embla) Fearless Refactor v1 — Plan

Status: Updated (living document)

This plan summarizes what is already landed and proposes the next, highest-leverage steps.

## What is already landed

- M1: Headless Embla-aligned snap model + tests
  - `ecosystem/fret-ui-headless/src/carousel.rs` (`snap_model_1d`)
  - Workstream contract: `docs/workstreams/carousel-embla-fearless-refactor-v1/snap-model-contract.md`
- M2: Recipe integration + minimal options surface (`align`, `containScroll`, `slidesToScroll`)
  - `ecosystem/fret-ui-shadcn/src/carousel.rs`
- M3: UI gallery parity with shadcn docs + deterministic geometry harness
  - `apps/fret-ui-gallery/src/ui/pages/carousel.rs`
  - `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/carousel.rs`
- P4: Carousel × DnD pointer arbitration (mouse handle path)
  - Decision + notes: `docs/workstreams/carousel-embla-fearless-refactor-v1/dnd-arbitration.md`
  - Policy hook: `fret-ui-shadcn::Carousel` skips swiping while a DnD sensor tracks the pointer.
  - Gates:
    - `tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-demo-dnd-handle-gate.json`
    - `tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-demo-dnd-long-press-gate.json`

## Next steps (recommended order)

### 1) Evidence hardening (P3)

Goal: turn the current alignment into stable, repeatable gates.

- Add/refresh diag scripts so we can catch regressions without relying on manual UI runs:
  - `ui-gallery-carousel-*-screenshot.json` set should cover Demo/Sizes/Spacing/Vertical/Expandable.
  - Interaction gate: `tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-demo-dnd-handle-gate.json` (Carousel vs DnD arbitration)
- Add an audit note with evidence anchors (if the repo expects it):
  - `docs/audits/carousel-shadcn-embla-parity.md` (anchors: recipe + headless + tests + scripts).

Exit criteria:

- A minimal diag suite can be executed in one command and produces screenshots/bundles with stable
  `test_id` selectors.

### 2) Shared snap utilities (P4, optional but likely valuable)

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

### 3) Carousel vs DnD gesture arbitration (P4, later)

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
