# Carousel (Embla) Fearless Refactor v1 — Milestone 2

Milestone: M2 — Recipe Integration (shadcn carousel uses snap model)

Outcome: `ecosystem/fret-ui-shadcn::Carousel` uses the headless snap model for prev/next and drag
release targeting (instead of `index * extent`), enabling docs-aligned behaviors for sizing,
containment, and alignment.

## Deliverables

- `ecosystem/fret-ui-shadcn/src/carousel.rs`
  - Compute slide geometry (track/view size + slide rects).
  - Drive prev/next targeting using `CarouselSnapModel1D.snaps_px`.
  - Keep current pointer threshold + capture stealing behavior.
- Minimal, recipe-level configuration (policy layer):
  - `align` (start/center/end)
  - `contain_scroll` (none/keep/trim)
  - `slides_to_scroll` (auto/fixed)

## Acceptance Criteria

- Focused tests pass:
  - `cargo nextest run -p fret-ui-shadcn -E "test(web_vs_fret_layout_carousel_*)"`
  - `cargo nextest run -p fret-ui-shadcn -E "test(carousel_pointer_passthrough)"`
- Buttons disable state matches Embla-style semantics:
  - disabled until the viewport/extent is measurable
  - disabled at bounds when containment is enabled
- No mechanism changes leak into `crates/fret-ui` (if mechanism changes are needed, stop and write
  an ADR before proceeding).

## Notes / Risks

- Slide rect collection must be stable and deterministic across platforms (native + wasm).
- Keep allocations bounded during drag/move; prefer cached vectors in models/state where possible.

