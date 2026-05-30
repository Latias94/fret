# Material 3 Canvas Draw Region Packet v1

Date: 2026-05-28

## Decision

Material3 now exposes recipe-level diagnostic anchors for deterministic rectangular canvas-painted
regions, but does not pretend these are exact named canvas operations.

The packet keeps the boundary explicit:

- exact per-op names are a future `SceneOp`/diagnostics mechanism concern;
- rectangular recipe anchors live in `fret-ui-material3` foundation and recipes;
- non-rectangular or animated paint remains covered by scene/golden gates.

## Implemented Anchors

ProgressIndicator:

- `m3-linear-progress.track`
- `m3-linear-progress.active-track`

Slider:

- `m3-slider.track`
- `m3-slider.active-track`
- `m3-slider.handle`

RangeSlider:

- `m3-range-slider.track`
- `m3-range-slider.active-track`
- `m3-range-slider.start.handle`
- `m3-range-slider.end.handle`

## Foundation Shape

The shared helper creates hidden, non-focusable `Generic` semantics nodes with explicit absolute
layout. It does not draw, participate in focus traversal, or add Material-specific metadata to
`SceneOp`.

Centering uses layout margin, not render transforms. A transform-based draft was rejected because it
changed headless scene output by adding `PushTransform`/`PopTransform` operations.

## Explicit Non-Goals

- Circular progress arc bounds.
- Indeterminate segmented progress regions.
- Slider tick-marker, stop-indicator, and state-layer part ids.
- Any Material-specific `SceneOp` label.

Those surfaces stay golden-only in this lane. Add them later only with a concrete diagnostic
consumer and a bounded naming scheme, or solve exact draw-region naming generically in `crates/*`.

## Verification

- `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface`
- `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_progress_indicator_suite_goldens_v1`
- `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_slider_suite_goldens_v1`
- `cargo check -p fret-ui-material3 --features diagnostics --tests`
- `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
