# Transform Pipeline Notes (P0)

This note describes how `delinea` currently derives a per-series “visible row selection” and how
we intend to evolve it into an ECharts-like transform pipeline (dataset → transform → view) without
forcing a large redesign later.

## Current shape (P0)

### Inputs

The effective row slice for a series is derived from:

- Dataset-local row constraints: `ChartState.dataset_row_ranges[dataset]` (optional).
- Axis constraints and view windows:
  - Model constraints: `AxisRange::{Fixed, LockMin, LockMax}` (durable).
  - View state: `ChartState.data_window_x[axis]` (ephemeral zoom/pan window).

### Policy and precedence

We use a single policy function to derive “what X values are allowed”:

- `engine/window_policy.rs`:
  - `axis_filter_1d(axis_range, state_window)` → `AxisFilter1D { min, max }`

Semantics:

- `AxisRange::Fixed` overrides view windows (`data_window_x` is ignored).
- `LockMin/LockMax` constrain the allowed region but do not fully override `data_window_x`.
- If both min and max are present, the filter behaves like a window.
- If only one side is present, it behaves like a half-space constraint (e.g. `x >= min`).

### Row slicing

Given:

- `base_range`: the dataset-local row range (or full dataset),
- `mapping window`: the X window used for coordinate mapping (`AxisRange::Fixed` or `data_window_x`),
- `x_filter`: the X constraint used for bounds filtering,

we derive a continuous `RowRange` (stored as `RowSelection::Range`) for the series:

- `transform/x_slice.rs`:
  - `row_range_for_x_window(x_values, base_range, window)` → `RowRange`

When the X column is probably monotonic, we use binary search (`partition_point`) for O(log n)
slicing. Otherwise we fall back to a linear scan to find the first/last matching row.

### Filter mode (P0)

We also support a minimal filter mode toggle for `data_window_x`:

- `FilterMode::Filter` (default): row slicing uses the mapping window, and bounds/LOD are filtered to the window.
- `FilterMode::None`: row slicing does not apply the window (only dataset row constraints do), and bounds are global.
  This is more expensive, but can be useful when users want zoomed X without re-scaling Y.

## Why this is not a full transform system yet

Today we only support “slicing” transforms that can be represented as a continuous `RowRange`.
This keeps the LOD/marks pipeline simple and allocation-free.

However, many ECharts transforms are not representable as a single range:

- arbitrary filters (sparse selection),
- downsampling with “keep extrema” policies,
- aggregation and grouping,
- stacking that needs intermediate columns.

## Next steps (P0 → P1)

1. Introduce an internal selection type that can represent:
   - `All`
   - `Range(RowRange)`
   - (later) `Indices(Vec<u32>)` or a compact bitmap
2. Add a small set of transform nodes with revision-based caching:
   - `SliceRows` (current behavior)
   - `FilterRange` (explicit min/max filters per channel)
   - `Downsample` (min/max per pixel, decimation policies)
3. Move `dataZoom` / `filterMode` into a transform node rather than hard-coding it into view logic.

The intent is to keep the engine deterministic and testable while staying performant on large datasets.
