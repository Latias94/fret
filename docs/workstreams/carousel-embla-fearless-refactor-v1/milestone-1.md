# Carousel (Embla) Fearless Refactor v1 — Milestone 1

Milestone: M1 — Headless Snap Model (Embla-aligned)

Outcome: A deterministic, Embla-aligned snap model in `fret-ui-headless` with tests that lock the
behavior as a stable contract.

## Deliverables

- `ecosystem/fret-ui-headless/src/carousel.rs`
  - `snap_model_1d(...) -> CarouselSnapModel1D`
  - options mapped to Embla vocabulary:
    - `slidesToScroll`: `auto` / `n`
    - `containScroll`: `false` / `keepSnaps` / `trimSnaps`
    - `align`: `start` / `center` / `end`
- Unit tests that cover:
  - snap list values (rounded like Embla)
  - contained snaps + trim/keep semantics
  - slide-grouping (`auto` grouping by size + gaps)
  - edge cases (empty slides, small content, pixel tolerance)

## Acceptance Criteria

- `cargo nextest run -p fret-ui-headless` passes.
- Tests clearly state expected behavior and include the smallest possible geometry fixtures.
- No new dependencies leak into contract crates (keep this in `ecosystem/` only).

## Notes / Risks

- Percent sizing vs px sizing differences: tests should operate on px geometry (slide rects and
  view size) so results are runtime-independent.
- Rounding behavior: Embla rounds bounded snaps to 3 decimals. Keep that deterministic.

