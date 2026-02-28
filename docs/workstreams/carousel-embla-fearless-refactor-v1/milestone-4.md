# Carousel (Embla) Fearless Refactor v1 — Milestone 4

Milestone: M4 — Motion + Option Semantics Hardening

Outcome: Carousel motion timing (`duration`) uses the shared motion foundations (duration-driven,
refresh-rate scaled, reduced-motion aware), and remaining option semantics are documented and gated.

## Deliverables

- Motion alignment for settle:
  - `ecosystem/fret-ui-shadcn/src/carousel.rs`
    - replace fixed-tick settle logic with a duration-driven transition driver
    - keep existing easing choice (shadcn-aligned), but let duration be wall-clock based
    - ensure reduced-motion results in instant snaps (no animation)
  - evidence anchor: `ecosystem/fret-ui-kit/src/declarative/transition.rs`
    - `ticks_60hz_for_duration(...)` and duration-driven drivers
- Option semantics hardening:
  - document intended behavior for `loop_enabled`, `skip_snaps`, and `drag_free` (best-effort parity)
  - explicitly call out non-goals (no momentum physics, no seamless loop engine)
  - ensure behavior is covered by at least one regression gate (unit test, web-vs-fret, or diag)

## Acceptance Criteria

- Layout parity gates still pass:
  - `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout carousel`
- Headless snap + release tests still pass:
  - `cargo nextest run -p fret-ui-headless --tests carousel`
- Diag scripts remain deterministic under fixed delta:
  - `tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-api-screenshot.json`
  - `tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-plugin-autoplay-pixels-changed.json`

## Notes / Risks

- The transition driver performs refresh-rate scaling only when the host provides window metrics or
  an explicit fixed frame delta. Headless tests should remain deterministic.
- This milestone does not attempt to port Embla's inertial scroll physics or seamless loop engine.
