# Shadow Portable Softness (Fearless Refactor v1) — TODO

Status: In Progress

Last updated: 2026-04-01

Related:

- Design: `docs/workstreams/shadow-portable-softness-fearless-refactor-v1/DESIGN.md`
- Milestones: `docs/workstreams/shadow-portable-softness-fearless-refactor-v1/MILESTONES.md`
- Prior closure lane: `docs/workstreams/shadow-surface-fearless-refactor-v1/DESIGN.md`

Tracking legend:

- `[ ]` open
- `[~]` in progress
- `[x]` done

## A. Root-cause freeze

- [x] SPSFR-audit-001 Confirm that shared shadcn control shadow geometry is already source-aligned.
  - Evidence now covers `button-demo`, `button-group-demo` leaf buttons, `input-group-demo`,
    `input-demo`, `textarea-demo`, `select-demo`, and `native-select-demo`.

- [x] SPSFR-audit-002 Confirm whether the remaining "hardness" is token drift or painter drift.
  - Result: the main remaining issue is painter drift in `crates/fret-ui/src/paint.rs`, not shadcn
    preset geometry.

## B. Portable painter correction

- [x] SPSFR-paint-010 Normalize per-step portable shadow alpha so softness does not inflate total
  layer opacity.
  - Landed in `crates/fret-ui/src/paint.rs`.

- [x] SPSFR-paint-011 Add mechanism-layer tests for:
  - normalized alpha budget across multi-step blur,
  - zero-blur single-layer preservation.

## C. Evidence updates

- [x] SPSFR-doc-020 Record the painter-fidelity follow-on explicitly in workstream docs.

- [x] SPSFR-doc-021 Update shadcn alignment audit notes so shared shadow lanes are not blamed for
  the remaining subjective hardness.

## D. Follow-on quality gates

- [x] SPSFR-gate-030 Add a stronger softness-oriented gate for portable `ShadowStyle`.
  - Landed as a mechanism-layer composited profile gate in `crates/fret-ui/src/paint.rs`.
  - The gate asserts:
    - darkness increases monotonically toward the edge,
    - full-overlap darkness stays within the recipe-owned alpha budget under layer compositing,
    - outer bands stay lighter than the full-overlap stack.
  - This is intentionally stronger than footprint-only evidence, while still staying deterministic
    and backend-independent.

## E. Renderer evidence follow-on

- [x] SPSFR-diag-040 Add a curated screenshot repro suite for representative shadow surfaces.
  - Landed as `tools/diag-scripts/suites/ui-gallery-shadow-surface-screenshots/suite.json`.
  - Current suite members:
    - `tools/diag-scripts/ui-gallery/card/ui-gallery-card-demo-screenshot.json`
    - `tools/diag-scripts/ui-gallery/calendar/ui-gallery-calendar-demo-shadow-screenshot.json`
    - `tools/diag-scripts/ui-gallery/sonner/ui-gallery-sonner-demo-layout-collapse-screenshots.json`
  - This is renderer-level review evidence, not yet an automated screenshot-diff or pixel-threshold
    gate.

- [ ] SPSFR-gate-041 Add an automated renderer-level shadow comparison gate once screenshot ROI /
  pixel-diff policy is ready.
  - Candidate paths:
    - screenshot diff on a bounded ROI for representative elevated surfaces,
    - or renderer readback assertions over a deterministic shadow strip/demo scene.
