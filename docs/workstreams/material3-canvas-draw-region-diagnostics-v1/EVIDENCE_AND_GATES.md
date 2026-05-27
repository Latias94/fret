# Material 3 Canvas Draw Region Diagnostics v1 - Evidence And Gates

Status: Closed
Last updated: 2026-05-28

## Current Evidence

- ProgressIndicator paints track/active regions through `cx.canvas` and anonymous
  `SceneOp::Quad` entries:
  - `ecosystem/fret-ui-material3/src/progress_indicator.rs`
- Slider and RangeSlider paint track, handle, state layer, tick marks, and stop indicators through
  `cx.canvas` and anonymous `SceneOp::Quad` entries:
  - `ecosystem/fret-ui-material3/src/slider.rs`
- Canvas and scene contracts currently expose retained `SceneOp`s but no per-op labels or metadata:
  - `crates/fret-ui/src/canvas.rs`
  - `crates/fret-core/src/scene/mod.rs`
- Diagnostics bundle snapshots expose scene op counts/fingerprints and paint hotspots, not named
  draw regions:
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/snapshot_types.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/service.rs`

## Gates

Run as the lane advances:

```powershell
python -m json.tool docs/workstreams/material3-canvas-draw-region-diagnostics-v1/WORKSTREAM.json | Out-Null
python tools/check_workstream_catalog.py
cargo fmt --package fret-ui-material3 -- --check
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_progress_indicator_suite_goldens_v1
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_slider_suite_goldens_v1
cargo check -p fret-ui-material3 --features diagnostics --tests
```

Use `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
before closeout if code changed outside narrow tests/docs.

## Evidence Log

- 2026-05-28: M3CD-010 source audit completed. Exact named canvas draw regions require a generic
  diagnostics/scene-label mechanism; rectangular Material recipe anchors are valid only when they
  can truthfully match the painted region.
- 2026-05-28: M3CD-020 through M3CD-040 completed. Added Material3 hidden diagnostic anchor
  helpers, linear progress `track`/`active-track` anchors, and slider/range-slider
  `track`/`active-track`/`handle` anchors.
  - Red proof before implementation:
    `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_surface_data_display_expose_stable_part_test_ids`
    failed on missing `m3-linear-progress.track`.
  - Red proof before implementation:
    `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_choice_controls_expose_stable_part_test_ids`
    failed on missing `m3-slider.track`.
  - Golden guardrail: an early render-transform helper made
    `material3_headless_slider_suite_goldens_v1` fail by adding `PushTransform`/`PopTransform`; the
    final helper uses pure layout plus margin centering.
- 2026-05-28: M3CD-050 closeout verification.
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
  - `python tools/check_workstream_catalog.py`: passed, 477 dedicated directories and 47
    standalone markdown files.
