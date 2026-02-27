# Carousel (Embla) Fearless Refactor v1 — Milestone 3

Milestone: M3 — Docs Parity + Diagnostics + Cleanup

Outcome: UI gallery is visually aligned with shadcn docs examples, and we have repeatable
diagnostics/scripts to catch regressions.

## Deliverables

- `apps/fret-ui-gallery/src/ui/pages/carousel.rs`
  - examples ordered and sized like shadcn docs:
    - Demo
    - Sizes
    - Spacing
    - Orientation (Vertical)
  - code samples are complete and copyable
- Diagnostics gates (existing scripts may be updated):
  - `tools/diag-scripts/ui-gallery-carousel-demo-screenshot.json`
  - `tools/diag-scripts/ui-gallery-carousel-demo-dnd-handle-gate.json`
  - `tools/diag-scripts/ui-gallery-carousel-demo-dnd-long-press-gate.json`
  - `tools/diag-scripts/ui-gallery-carousel-basic-screenshot.json`
  - `tools/diag-scripts/ui-gallery-carousel-spacing-screenshot.json` (if missing, add)
  - `tools/diag-scripts/ui-gallery-carousel-orientation-vertical-screenshot.json`
  - `tools/diag-scripts/ui-gallery-carousel-expandable-screenshot.json` (non-upstream, motion pilot)
- Documentation updates:
  - `docs/audits/carousel-shadcn-embla-parity.md` evidence anchors refreshed

## Acceptance Criteria

- Targeted diags are green (native):
  - `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-carousel-*-screenshot.json --warmup-frames 5 --exit-after-run --launch -- cargo run -p fret-ui-gallery --release`
- No crate boundary violations (run layering checks if any new deps were introduced).
- Workstream TODO (`TODO.md`) updated with checked items + evidence links (paths/test names).

## Notes / Risks

- UI gallery is not a stable contract surface, but its test IDs are used by diag scripts. Keep
  `ui-gallery-carousel-*` IDs stable when possible.
