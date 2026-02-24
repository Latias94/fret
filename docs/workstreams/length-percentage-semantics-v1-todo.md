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

- [x] Decide the v1 surface:
  - [x] `min_w/min_h/max_w/max_h` accept `LengthRefinement` (px/fraction/fill/auto; `Auto` means “unset constraint”).
  - [x] Clamp policy: negative ratios clamp to `0`, non-finite behaves like `0`.
- [x] Implement percent/fraction for min/max sizes in the declarative bridge.
- [x] Add a unit test proving “min/max percent does not collapse under min/max-content probes”.
  - Evidence: `crates/fret-ui/src/declarative/tests/layout/basics.rs` (`min_max_fraction_only_resolve_under_definite_available_space_in_measurement`)

## M2 — Spacing percent support (padding + gap)

- [x] Decide a “definite-only” length type for `padding` and `gap` (no `auto`).
  - Evidence: `crates/fret-ui/src/element.rs` (`SpacingLength`, `SpacingEdges`)
- [x] Add `padding_fraction/padding_percent` (edges) shorthands at the kit layer.
  - Evidence: `ecosystem/fret-ui-kit/src/ui_builder.rs` (`padding_fraction`, `padding_percent`, `paddings_*`)
- [x] Add `gap_fraction/gap_percent` shorthands (x/y) where it makes sense.
  - Evidence: `ecosystem/fret-ui-kit/src/ui_builder.rs` (`gap_fraction`, `gap_percent`, `gap_full`)
- [x] Implement the mapping in the declarative host (measurement + layout) using v1 resolution rules.
  - Evidence: `crates/fret-ui/src/declarative/host_widget/measure.rs` (`spacing_px_for_basis`)
  - Evidence: `crates/fret-ui/src/layout/engine/flow.rs` (`taffy_lp_from_spacing`)
- [x] Add a focused unit test covering percent padding/gap under both definite and intrinsic measurement.
  - Evidence: `crates/fret-ui/src/declarative/tests/layout/basics.rs` (`spacing_fraction_only_resolve_under_definite_available_space_in_measurement`)

## M3 — Positioning percent support (inset + margin)

- [x] Decide percent semantics:
  - [x] `inset`: per-axis containing block size (left/right use width, top/bottom use height).
  - [x] `margin`: follows Taffy/CSS-like behavior (percent margins resolve against containing block width, including top/bottom).
- [x] Extend `InsetRefinement` and `MarginEdgeRefinement` to express percent/fraction.
  - Evidence: `crates/fret-ui/src/element.rs` (`InsetEdge`, `MarginEdge`)
  - Evidence: `ecosystem/fret-ui-kit/src/style/chrome.rs` (`InsetEdgeRefinement`, `MarginEdgeRefinement`)
  - Evidence: `ecosystem/fret-ui-kit/src/style/layout.rs` (`inset_*`, `top_*`, `mx_*`, etc)
- [x] Add focused unit tests for percent inset/margin positioning.
  - Evidence: `crates/fret-ui/src/declarative/tests/layout/basics.rs` (`absolute_inset_fraction_resolves_against_containing_block`)
  - Evidence: `crates/fret-ui/src/declarative/tests/layout/basics.rs` (`flex_margin_fraction_uses_containing_block_width_for_top`)

## M4 — Ecosystem migration (remove workarounds)

- [ ] Audit `ecosystem/fret-ui-shadcn` for px clamping patterns used to patch percent collapse.
- [ ] Migrate components to native percent/fraction fields:
  - [x] carousel (basis-full default)
    - Evidence: `ecosystem/fret-ui-shadcn/src/carousel.rs` (basis-full uses `LengthRefinement::Fill`)
    - Gate: `tools/diag-scripts/ui-gallery-carousel-basic-screenshot.json`
  - [x] overlay roots / sheets that currently rely on explicit px extents
    - Evidence: `ecosystem/fret-ui-shadcn/src/sheet.rs` (remove viewport clamp; use `Length::Fill`/`Length::Fraction` max constraints)
    - Gate: `ecosystem/fret-ui-shadcn/src/sheet.rs` (`sheet_bottom_auto_max_height_fraction_clamps_tall_content_with_edge_gap`)
    - Gate: `tools/diag-scripts/ui-gallery-sheet-escape-focus-restore.json`
  - [x] drawers that currently rely on explicit px extents
    - Evidence: `ecosystem/fret-ui-shadcn/src/drawer.rs` (`DrawerContent` uses `max_h_fraction` vs viewport math)
    - Gate: `ecosystem/fret-ui-shadcn/src/drawer.rs` (`drawer_content_max_height_fraction_clamps_tall_content`)
    - Gate: `tools/diag-scripts/ui-gallery-drawer-docs-smoke.json`
- [ ] Add a gate per migration:
  - [x] a unit test when the invariant is layout-only
  - [x] a diag script when the invariant is “docs-aligned UI outcome”

## Diagnostics / evidence

- [ ] Ensure each milestone has:
  - [ ] an evidence anchor (file + key helper/function)
  - [ ] at least one regression gate (test and/or diag script)

Known gates in this workstream:

- M0/M1/M2 unit tests: `crates/fret-ui/src/declarative/tests/layout/basics.rs`
- Carousel basic screenshot gate: `tools/diag-scripts/ui-gallery-carousel-basic-screenshot.json`
