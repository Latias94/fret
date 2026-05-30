# Material 3 Canvas Draw Region Diagnostics v1 - Closeout Audit

Date: 2026-05-28
Status: Closed

## Result

The lane is closed.

It classified exact named canvas draw regions as a mechanism gap, added Material3 layout-only
diagnostic anchors for truthful rectangular regions, and kept non-rectangular or animated canvas
paint under scene/golden evidence.

## Completed Work

- Added Material3 foundation helpers for absolute hidden diagnostic anchors.
- Added linear progress `track` and `active-track` anchors.
- Added slider and range-slider `track`, `active-track`, and `handle` anchors.
- Expanded `automation_surface` expectations to cover the new part ids.
- Preserved progress and slider headless scene goldens.
- Recorded the split between recipe-level anchors and future generic named `SceneOp` diagnostics.

## Gate Evidence

- `cargo fmt --package fret-ui-material3 -- --check`: passed.
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface`:
  passed, 20 tests.
- `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_progress_indicator_suite_goldens_v1`:
  passed.
- `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_slider_suite_goldens_v1`:
  passed.
- `cargo check -p fret-ui-material3 --features diagnostics --tests`: passed.
- `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`:
  passed.
- `python -m json.tool docs/workstreams/material3-canvas-draw-region-diagnostics-v1/WORKSTREAM.json | Out-Null`:
  passed.
- `python tools/check_workstream_catalog.py`: passed, 477 dedicated directories and 47 standalone
  markdown files.

## Boundary Notes

- No `crates/*` contract was changed.
- No Material-specific metadata was added to `SceneOp`.
- Circular progress arcs, indeterminate segments, tick markers, stop indicators, and state-layer
  paint remain golden-only until a concrete consumer proves a better bounded contract is needed.

## Follow-On Policy

Start a new lane only for one of these cases:

- a generic mechanism contract for named canvas/scene draw regions across design systems;
- a narrow Material3 diagnostic script that needs tick, stop, or state-layer ids and proves a stable
  naming scheme.
