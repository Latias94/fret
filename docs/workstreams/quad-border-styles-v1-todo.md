# Quad Border Styles v1 — TODO Tracker

Status: Active (workstream tracker)

This document tracks executable TODOs for adding dashed border support as a first-class render
semantic.

Workstream narrative: `docs/workstreams/quad-border-styles-v1.md`
Milestone board (one-screen): `docs/workstreams/quad-border-styles-v1-milestones.md`

## Tracking format

Each TODO is labeled:

- ID: `QBST-MVP{n}-{area}-{nnn}`
- Status: `[ ]` (open), `[~]` (in progress), `[x]` (done), `[!]` (blocked)

## Milestones (make progress measurable)

### MVP0 — Contract + renderer spike (end-to-end dashed stroke)

- [ ] QBST-MVP0-contract-001 Decide the contract surface:
  - Recommended: add `SceneOp::StrokeRRect` (keep `Quad` semantics stable).
  - Record the decision and link to ADR 0030 (“Dashed borders deferred”).
- [ ] QBST-MVP0-contract-002 Write an ADR update or a new ADR that locks:
  - dash model (`dash/gap/phase`),
  - perimeter parameterization,
  - pixel snapping + transform interaction rules.
- [ ] QBST-MVP0-core-010 Implement new scene contract types in `fret-core`:
  - `DashPatternV1` (fixed-size, `Copy`)
  - `StrokeStyleV1` (reserve room for future joins/caps; v1 only needs optional dash)
  - `SceneOp::StrokeRRect { order, rect, corner_radii, stroke: Edges, stroke_paint: Paint, style }`
- [ ] QBST-MVP0-ui-020 Add an opt-in mechanism API for dashed borders at the UI layer:
  - `ContainerProps` (or a dedicated border chrome struct) gains `border_style` / `dash_pattern`.
  - Ensure existing callsites remain unchanged by default.
- [ ] QBST-MVP0-render-030 Extend the quad shader pipeline to support dashed border masking:
  - Add instance fields needed for dashing (dash, gap, phase, enabled flag).
  - Implement stable perimeter coordinate (`s`) for rrects.
  - Multiply `border_cov` by a AA’d `dash_mask`.
- [ ] QBST-MVP0-render-031 Add a renderer conformance test with GPU readback:
  - Render a dashed rrect at multiple scale factors (1.0, 1.25/1.5, 2.0).
  - Sample a set of points around the perimeter to validate periodicity and stability.

### MVP1 — shadcn parity wiring (`border-dashed` actually looks dashed)

- [ ] QBST-MVP1-shadcn-100 Map `border-dashed` to the new mechanism capability:
  - Buttons (`outline` + dashed border, tasks-style).
  - Empty states / drop zones that already assert the token.
- [ ] QBST-MVP1-demo-110 Add / update a UI Gallery demo that makes the dashed border visible and
  obvious at a glance:
  - Prefer a small “dashed outline control” panel (rect + rounded rect).
  - Ensure stable `test_id` anchors exist for automation.
- [ ] QBST-MVP1-gates-120 Add a regression gate:
  - If we can reliably script a stable capture: a `fretboard diag` script.
  - Otherwise: a targeted renderer test that validates output pixels changed (dashed vs solid).

### MVP2 — editor-grade polish (marching ants, drop zones)

- [ ] QBST-MVP2-anim-200 Add “marching ants” demo with animated `phase_px`.
- [ ] QBST-MVP2-anim-201 Add a determinism gate:
  - Phase update must be stable under pause/resume and not depend on wall-clock time.
- [ ] QBST-MVP2-interaction-210 Wire dashed borders into at least one real editor interaction
  surface (e.g. docking drop-zone highlight) if/when a consumer exists.

## Open questions (must resolve before landing MVP0)

- Perimeter semantics: perimeter-continuous vs per-edge restart.
- Phase anchoring: what is the exact “start point” for the dash loop?
- Rounded corners: how do we parameterize arcs consistently across corners?
- Pixel snapping: does snapping affect dash phase / pattern lengths?
- Transform semantics: do we accept deformation under non-uniform transforms (likely yes)?
