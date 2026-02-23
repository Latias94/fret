# Length percentage semantics v1 — TODO

Last updated: 2026-02-23

This file is the *living checklist* for `length-percentage-semantics-v1`.
Use `length-percentage-semantics-v1-milestones.md` for phase planning.

## M0 — Baseline percent sizing (landed)

- [x] Add `Length::Fraction(f32)` representation.
- [x] Add authoring shorthands: `w_fraction/w_percent`, `h_fraction/h_percent`, `basis_fraction/basis_percent`.
- [x] Resolve `Fill`/`Fraction` only when `AvailableSpace` is definite; otherwise treat as `auto`.
- [x] Ensure wrapper chains promote percent descendants so they can resolve under a definite containing block.
- [x] Add focused unit tests for resolution rules and “no collapse under min-content measurement”.

## M1 — Size constraints percent support (min/max)

- [ ] Decide the v1 surface:
  - [ ] `min_w/min_h/max_w/max_h` accept `LengthRefinement` (px/fraction/fill/auto?) or a new `DefiniteLengthRefinement`?
  - [ ] clamp policy for negative ratios and non-finite values.
- [ ] Implement percent/fraction for min/max sizes in the declarative bridge.
- [ ] Add a unit test proving “min/max percent does not collapse under min-content probes”.

## M2 — Spacing percent support (padding + gap)

- [ ] Decide a “definite-only” length type for `padding` and `gap` (no `auto`).
- [ ] Add `padding_fraction/padding_percent` (edges) shorthands at the kit layer.
- [ ] Add `gap_fraction/gap_percent` shorthands (x/y) where it makes sense.
- [ ] Implement the mapping in the declarative host (Taffy style generation) using v1 resolution rules.
- [ ] Add a focused unit test covering percent padding/gap under both definite and intrinsic measurement.

## M3 — Positioning percent support (inset + margin)

- [ ] Decide whether percent inset/margin uses:
  - [ ] the containing block width for left/right and height for top/bottom (CSS-like), or
  - [ ] per-axis base size (simpler mental model for v1).
- [ ] Extend `InsetRefinement` and `MarginEdgeRefinement` to express percent/fraction.
- [ ] Add a unit test for percent inset positioning (basic “inset-0”, “inset-10%” outcomes).

## M4 — Ecosystem migration (remove workarounds)

- [ ] Audit `ecosystem/fret-ui-shadcn` for px clamping patterns used to patch percent collapse.
- [ ] Migrate components to native percent/fraction fields:
  - [ ] carousel (basis-full default)
  - [ ] overlay roots / sheets / drawers that currently rely on explicit px extents
- [ ] Add a gate per migration:
  - [ ] a unit test when the invariant is layout-only
  - [ ] a diag script when the invariant is “docs-aligned UI outcome”

## Diagnostics / evidence

- [ ] Ensure each milestone has:
  - [ ] an evidence anchor (file + key helper/function)
  - [ ] at least one regression gate (test and/or diag script)

