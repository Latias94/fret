# `delinea` LOD Conformance (P0 Baseline)

This document defines the **minimum invariants** for `delinea` level-of-detail (LOD) / downsampling
behavior, plus the **test and demo harnesses** that lock those invariants.

Goal: ensure large-data behavior stays deterministic and pixel-bounded while we keep adding
ECharts-class features.

## Terminology

- **Pixel-bounded**: emitted geometry is bounded by plot pixel width, not dataset length.
- **Monotonic X**: input X values are non-decreasing in raw row order.
- **Raw index**: the dataset row index, preserved through transforms as `data_indices` (ADR 0202).

## Current v1 strategies

### Line-family (line/area/band): min/max per pixel column

Implementation: `ecosystem/delinea/src/engine/lod/minmax_per_pixel.rs`

- Bucket samples by plot width in pixels.
- For each bucket, emit up to 4 candidates: `first`, `min`, `max`, `last`.
- Deduplicate by raw index within the bucket.

This preserves spikes while staying stable under large data.

### Scatter: exact for small, pixel-bounded for large

Implementation: `ecosystem/delinea/src/engine/stages/marks.rs`

- Default threshold: `20_000` visible rows.
- If `visible_len <= threshold`, emit all visible points (exact mode).
- If `visible_len > threshold`, switch to the same min/max-per-pixel strategy as line-family.
- Threshold and progressive cap are configurable via `SeriesSpec.lod` (v1 subset):
  - `large` / `large_threshold`
  - `progressive` / `progressive_threshold` (forces multi-step building even with large `WorkBudget`)

### Bar: exact rectangles (not pixel-bounded)

Implementation: `ecosystem/delinea/src/engine/stages/marks.rs`

- Bars currently emit one rectangle per visible row (exact mode).
- This is budgeted (incremental stepping), but not pixel-bounded. A future LOD/progressive strategy
  is expected for large categorical datasets and stacked bars.

## P0 invariants (must stay stable)

### Geometry bounds and cardinality

For a plot viewport with `plot_width_px = ceil(viewport.size.width)`:

- Pixel-bounded output: emitted points must satisfy `points.len() <= 4 * plot_width_px`.
- Identity alignment: `points.len() == data_indices.len()`.
- Index validity: every emitted `data_index` must be within the dataset row range.
- View bounds: emitted points must lie inside the viewport rectangle (after clamping).

For bars:

- Identity alignment: `rects.len() == rect_data_indices.len()`.
- Index validity: every emitted `rect_data_index` must be within the dataset row range.

### Ordering and determinism (monotonic X precondition)

When the input X is monotonic:

- Emitted `data_indices` are strictly increasing.
- The output is deterministic: same inputs and viewport produce the same output sequence.

Note: if X is not monotonic, ordering invariants are not guaranteed by the current algorithm.

### Budget invariance (incremental stepping)

Given the same spec, dataset, state, and viewport:

- Running the engine to completion with different `WorkBudget` values must converge to the same
  `marks` output and `axis_windows` (i.e. budgets only affect *how fast* we build, not *what* we
  build).

## Where these invariants are enforced

### Unit tests

- `ecosystem/delinea/src/engine/lod/minmax_per_pixel.rs`
  - Validates `minmax_per_pixel_finalize` is pixel-bounded and index-aligned for monotonic inputs.

### Engine tests

- `ecosystem/delinea/src/engine/tests.rs`
  - `scatter_large_mode_is_pixel_bounded`
  - `scatter_large_threshold_can_force_large_mode`
  - `scatter_progressive_can_force_multiple_steps`
  - `line_large_mode_is_pixel_bounded`
  - `lod_scatter_large_mode_is_budget_invariant`
  - `lod_line_large_mode_is_budget_invariant`
  - `lod_bar_mode_is_budget_invariant`

### Manual stress harness

- Demo: `apps/fret-examples/src/chart_stress_demo.rs`
- Runner: `cargo run -p fret-demo --bin chart_stress_demo`

Environment knobs:

- `FRET_CHART_STRESS_POINTS=<usize>` (default `1_000_000`, clamped `1..=10_000_000`)
- `FRET_CHART_STRESS_EXIT_AFTER_FRAMES=<u64>` (optional)
- `FRET_CHART_STRESS_HELP=1` (prints help on start)

## Follow-ups (P1)

- Extend per-series knobs coverage (`SeriesSpec.lod` exists; wiring beyond scatter is still pending).
- Add optional higher-fidelity sampling (e.g. LTTB) for moderate sizes.
- Add a benchmark harness that can gate frame-time regressions on CI.
