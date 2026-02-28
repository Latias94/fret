# Milestones — Carousel Embla fearless refactor (v1)

Milestones are structured to keep changes reviewable and reversible. Each milestone must ship a
“3-pack”: Repro (smallest surface), Gate (tests/scripts), Evidence (anchors + upstream refs).

Current status (as of 2026-02-27): M0–M3 shipped locally with gates. M4 is in progress.

## M0 — Gate set (fearless foundation) ✅

**Goal:** Create enough regression protection that we can refactor without fear.

**Deliverables**

- Headless unit tests for:
  - variable slide size snap derivation,
  - `align`,
  - `containScroll=trimSnaps`.
- Geometry parity coverage expanded for:
  - vertical constrained viewport,
  - at least one variable-size slide case.
- Diag scripts:
  - drag from interactive descendant cancels activation,
  - (optional) touch cross-axis scroll lock.

**Exit criteria**

- All gates green locally (`cargo nextest run -p fret-ui-shadcn web_vs_fret_layout_carousel` plus the new headless tests).
- At least one diag script produces a packed bundle and is deterministic with fixed frame delta.

## M1 — Recipe snap wiring ✅

**Goal:** Drive prev/next/keys from a snap list (not `index * extent`) while keeping the recipe
policy-only.

**Deliverables**

- `snap_model_1d` wired into `ecosystem/fret-ui-shadcn::Carousel`.
- Minimal options surface (`align`, `containScroll`, `slidesToScroll`) stays recipe-only.

**Exit criteria**

- Screenshot + web-vs-fret layout parity gates pass for Demo/Sizes/Spacing/Vertical/Expandable.

## M2 — Geometry-derived snaps ✅

**Goal:** Replace uniform extent snapping with geometry-derived snap list.

**Deliverables**

- Snap list derived from measured slide sizes + gaps.
- Selection/index computed from snaps.
- Buttons/keys operate on snaps (not “index * extent”).

**Exit criteria**

- Variable slide size gates pass.
- Orientation vertical gates still pass.

## M3 — Docs parity extras (API snapshot + autoplay) ✅

**Goal:** Align with shadcn docs “API” and “Plugins” examples without exposing Embla's imperative API
surface.

**Deliverables**

- Deterministic API snapshot surface for slide counters.
- Recipe-level autoplay policy surface + UI gallery demo.
- Diag gate that proves autoplay advances without interaction (`--check-pixels-changed`).

**Exit criteria**

- `fretboard diag run ...ui-gallery-carousel-plugin-autoplay-pixels-changed.json --check-pixels-changed ui-gallery-carousel-plugin` passes.

## M4 — Remaining drift + ergonomics (next)

**Goal:** Close remaining deltas against shadcn docs/Embla expectations with minimal surfaces, and
align the remaining motion/timing behavior with Fret's motion foundations.

**Deliverables**

- Decide/lock any missing option semantics (e.g. `loop`, `slidesToScroll` edge cases) in headless.
- Align `duration`-driven settle with motion foundations:
  - use a duration-driven transition driver (not a carousel-specific fixed-tick loop)
  - refresh-rate scaling and reduced-motion behavior are consistent with other shadcn recipes
- Fix any remaining UI gallery layout drift (e.g. vertical layout, text wrapping) with gates.

**Exit criteria**

- Updated TODO entries are executable and have at least one new gate per drift class.
