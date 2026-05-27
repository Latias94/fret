# Material 3 Canvas Draw Region Diagnostics v1 - TODO

Status: Closed
Last updated: 2026-05-28

Task IDs use `M3CD-*`.

## M0 - Classification

- [x] M3CD-010 [owner=codex] [deps=none] [scope=ecosystem/fret-ui-material3/src/progress_indicator.rs,ecosystem/fret-ui-material3/src/slider.rs,crates/fret-ui/src/canvas.rs,crates/fret-core/src/scene/mod.rs,ecosystem/fret-bootstrap/src/ui_diagnostics]
  Goal: Classify whether ProgressIndicator/Slider named draw-region coverage is a component gap,
  Material foundation gap, diagnostics script gap, or mechanism gap.
  Validation: source audit recorded with exact owner boundaries.
  Review: DONE_WITH_SPLIT. Exact named `SceneOp` regions are a mechanism gap; rectangular recipe
  anchors remain viable for bounded Material3 parts.
  Evidence: `artifacts/canvas_draw_region_gap_audit_v1.md`.
  Handoff: Continue with M3CD-020; do not add Material-specific metadata to `SceneOp`.

## M1 - Foundation Helper

- [x] M3CD-020 [owner=codex] [deps=M3CD-010] [scope=ecosystem/fret-ui-material3/src/foundation,ecosystem/fret-ui-material3/tests/automation_surface.rs]
  Goal: Add a reusable Material3 hidden diagnostic anchor helper and gate that it produces stable
  live `test_id` bounds without visible paint or focus semantics.
  Validation: targeted `automation_surface` test or existing selector helper proof.
  Review: DONE. Added a layout-only hidden diagnostic anchor helper in Material3 foundation; an
  attempted render-transform centering helper was rejected because it polluted scene goldens with
  `PushTransform`/`PopTransform`.
  Evidence: `src/foundation/test_id.rs`; `tests/automation_surface.rs`;
  `artifacts/material3_canvas_draw_region_packet_v1.md`.
  Handoff: Use only for rectangular, deterministic paint regions.

## M2 - ProgressIndicator Anchors

- [x] M3CD-030 [owner=codex] [deps=M3CD-020] [scope=ecosystem/fret-ui-material3/src/progress_indicator.rs,ecosystem/fret-ui-material3/tests/automation_surface.rs,ecosystem/fret-ui-material3/tests/radio_alignment.rs]
  Goal: Add rectangular anchors for linear progress track/active track and classify circular/animated
  regions as scene-golden-only unless a precise generic scene-label mechanism exists.
  Validation: focused selector test plus `material3_headless_progress_indicator_suite_goldens_v1`.
  Review: DONE_WITH_KNOWN_FOLLOW_ONS. Linear progress now exposes `track` and `active-track`
  anchors; circular progress and indeterminate animation stay scene/golden-only.
  Evidence: `src/progress_indicator.rs`; `tests/automation_surface.rs`;
  `material3_headless_progress_indicator_suite_goldens_v1`.
  Handoff: Do not fake circular arc bounds as exact draw regions.

## M3 - Slider Anchors

- [x] M3CD-040 [owner=codex] [deps=M3CD-020] [scope=ecosystem/fret-ui-material3/src/slider.rs,ecosystem/fret-ui-material3/tests/automation_surface.rs,tools/diag-scripts/ui-gallery/material3]
  Goal: Add deterministic anchors for slider/range-slider track, active track, handle, and stable
  tick/stop/state-layer surfaces where geometry can be represented as rectangles.
  Validation: focused selector test; optional UI Gallery diag script if existing gallery surfaces
  expose the anchored ids.
  Review: DONE_WITH_KNOWN_FOLLOW_ONS. Slider and RangeSlider now expose `track`, `active-track`,
  and `handle` anchors. Tick markers, stop indicators, and state-layer paint remain golden-only in
  this slice to avoid anchor explosion and false exactness.
  Evidence: `src/slider.rs`; `tests/automation_surface.rs`;
  `material3_headless_slider_suite_goldens_v1`.
  Handoff: Keep pointer/touch behavior unchanged.

## M4 - Verification And Closeout

- [x] M3CD-050 [owner=codex] [deps=M3CD-030,M3CD-040] [scope=docs/workstreams/material3-canvas-draw-region-diagnostics-v1]
  Goal: Close the lane with fresh gates, a packet artifact, and any mechanism follow-on split.
  Validation: JSON/catalog gates, targeted Rust gates, headless progress/slider gates, and diff
  audit.
  Review: DONE. Fresh Rust, JSON, catalog, and headless gates pass; exact named scene-op diagnostics
  are split as a future mechanism lane only if multiple design systems need it.
  Evidence: `CLOSEOUT_AUDIT_2026-05-28.md`.
  Handoff: Return to the broader Material3 goal and pick the next packet.
