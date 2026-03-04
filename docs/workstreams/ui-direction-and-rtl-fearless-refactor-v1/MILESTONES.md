# UI Direction + RTL Parity (Fearless Refactor v1) — Milestones

Tracking doc: `docs/workstreams/ui-direction-and-rtl-fearless-refactor-v1/DESIGN.md`
TODO board: `docs/workstreams/ui-direction-and-rtl-fearless-refactor-v1/TODO.md`

## M1 — Direction substrate + invariants captured

Goal:

- The repo has a single place that states the invariants for direction (provider semantics, logical
  alignment, overlay-root caveats).

Exit gates:

- `DESIGN.md` lists the invariants + current evidence anchors.
- `TODO.md` contains a parity matrix table with owners + gates.

## M2 — “Logical alignment” is consistent by default

Goal:

- In the ecosystem authoring layer, `TextAlign::Start/End` behave as logical start/end under RTL.

Exit gates:

- Unit test exists that fails if Start/End do not flip under RTL.

## M3 — Gallery snippets match upstream shadcn RTL examples

Goal:

- Direction-related visual drift in `apps/fret-ui-gallery` is treated as a recipe issue first
  (missing wrappers like `p-4`, wrong width constraints, etc.).

Exit gates:

- At least one scripted screenshot gate exists for an RTL example (per component family as needed).

## M4 — Overlay-root direction propagation is not a footgun

Goal:

- Direction-sensitive behaviors do not silently regress when a component creates an overlay root.

Exit gates:

- At least one gate covers a provider-installed root + overlay-root creation + direction-dependent
  outcome (placement, alignment, or nav).

