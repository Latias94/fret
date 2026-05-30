# Material 3 Canvas Draw Region Diagnostics v1 - Handoff

Status: Closed
Last updated: 2026-05-28

## Current State

This narrow follow-on from the Material 3 component alignment sweep is closed.

M3CD-010 found:

- `SceneOp` has no label/metadata field and diagnostics bundle snapshots do not export named draw
  regions.
- ProgressIndicator and Slider both draw critical chrome in canvas paint closures.
- Existing headless goldens are the exact scene proof, while automation selectors need stable
  recipe-level anchors for rectangular painted regions.

## Completed Tasks

- M3CD-020: added Material3 layout-only hidden diagnostic anchor helpers.
- M3CD-030: added linear progress `track` and `active-track` anchors; left circular/animated
  progress scene-golden-only.
- M3CD-040: added slider and range-slider `track`, `active-track`, and `handle` anchors; left tick,
  stop, and state-layer paint scene-golden-only for now.
- M3CD-050: closed with fresh Rust, JSON, and catalog gates.

## Closeout Evidence

- `artifacts/canvas_draw_region_gap_audit_v1.md`
- `artifacts/material3_canvas_draw_region_packet_v1.md`
- `CLOSEOUT_AUDIT_2026-05-28.md`

## Guardrails

- Do not add Material-specific labels to `SceneOp`.
- Do not replace circular progress arcs with fake rectangular "exact" regions.
- Do not expand tick/stop/state-layer anchors without a concrete diagnostic consumer and a bounded
  naming scheme.
- Keep `crates/*` out of scope unless a follow-on ADR/contract task is explicitly opened for
  generic named scene-op diagnostics.

## Follow-On Candidates

- Open a mechanism lane only if multiple recipe crates need named canvas `SceneOp` regions.
- Open a narrow Material3 lane only if a UI Gallery diagnostic script proves it needs tick, stop, or
  state-layer part ids beyond the current headless scene gates.
