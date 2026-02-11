# ADR 0194: `delinea` Large Data + Progressive Rendering Strategy (P0/P1 Baseline)

Status: Proposed

## Context

Charts that feel “commercial-grade” must stay responsive under:

- large datasets (100k–10M points),
- frequent updates (streaming, zoom/pan),
- interactive hover (tooltips, crosshair, legend highlight).

ECharts has a mature streaming/progressive model (scheduler + pipelines + “large” modes).
`delinea` already supports incremental work via `WorkBudget` (ADR 0190), but we need to lock down
baseline policies so future features do not force a large rewrite.

## Relationship to Other ADRs

- ADR 0190: headless engine + `WorkBudget`.
- ADR 0191: transform pipeline and dataZoom semantics (filtering vs windowing).
- ADR 0095: renderer perf/stress harness (repo-level performance culture).

## Decision

### 1) Define “large mode” as a pipeline policy, not a chart type

Large-data behavior is a policy decided by:

- series kind (line/bar/scatter have different constraints),
- visible point count after transforms/windowing,
- plot rect pixel budget.

The engine may select a large-mode strategy per series:

- `Exact`: emit full geometry (small data).
- `LOD`: emit downsampled geometry (typical large line charts).
- `Progressive`: emit geometry incrementally across multiple steps (very large data / heavy transforms).

### 2) Downsampling strategy for line-family charts is “min/max per pixel column” (baseline)

For cartesian line-family charts, the baseline LOD strategy is:

- determine the visible X range,
- bucket samples by pixel column (or by a fixed horizontal resolution),
- for each bucket, emit at most `(min_y, max_y)` (and optionally endpoints) to preserve spikes.

This is deterministic, fast, and allocation-friendly.

#### Scatter baseline (v1)

Scatter series cannot preserve spikes the same way as line strokes. The v1 baseline for large scatter is:

- clip to the visible X window,
- apply a deterministic, pixel-budgeted sampling strategy,
- ensure the total emitted point count is bounded by the plot rect width (in pixels).

Current implementation policy (subject to a future spec surface):

- large behavior is automatically enabled when the visible point count is “large enough”.
- emitted points are bounded to `O(plot_width_px)` (currently `<= 4 * plot_width_px`).

Follow-up (P1):

- optional LTTB sampling for better visual fidelity at moderate sizes,
- aggregation transforms for statistical charts (histogram/boxplot).

### 3) Monotonic-X fast path is allowed and encouraged

Many interactive behaviors (axis-trigger tooltip sampling, visible range search) become O(log n)
when X is monotonic.

Contract:

- The engine may use a monotonic-X fast path when it can prove the precondition
  (or when the dataset is declared monotonic).
- When the precondition is not met, the engine must fall back to a safe (but potentially slower)
  strategy, or produce a deterministic “missing” output for that series in hover results.

### 4) Budgeted work is a hard requirement in hot paths

The engine must be able to:

- split heavy work across frames via `WorkBudget`,
- avoid per-frame allocations by reusing buffers,
- maintain precise invalidation keys (data revision, transform revision, window revision).

The UI adapter may call `step()` multiple times per frame, but the engine must never assume it
can finish a full rebuild in one call.

UI integration requirement (event-driven scheduling):

- When `step()` reports unfinished work, the UI adapter must continue driving future frames by
  requesting animation frames (ADR 0034). Progressive rendering must not depend on pointer-driven
  redraws.
- UI runtimes with paint caching must ensure progressive adapters are not skipped by cache replays
  while the engine is unfinished (e.g. disable cache for that node until the engine finishes).

### 5) Hover/tooltip must not force full geometry rebuild

Hover and tooltip sampling should:

- use precomputed indices (visible range) and cached scale mappings,
- avoid scanning all points or rebuilding marks,
- remain O(log n) per series in the monotonic fast path.

## Consequences

- We can scale to large datasets without blocking the UI thread.
- The engine architecture stays compatible with wasm (no threads required for baseline).
- Some parity features (e.g. fancy hover snapping on unsorted X) may be deferred unless we add
  spatial indices later.

## Follow-ups

P0:

- Add explicit “visible range index” caching per series (if not already present).
- Add unit tests for LOD determinism (same input -> same output).

P1:

- Add progressive chunking for bar/stack (rects) and scatter (symbols).
- Add optional spatial indices for unsorted data (grid/KD-tree) behind feature gates.
- Expose ECharts-aligned knobs (`large`, `largeThreshold`, `progressive`, `progressiveThreshold`) in spec/style.

## References

- ECharts scheduler/progressive pipeline: `F:\\SourceCodes\\Rust\\fret\\repo-ref\\echarts\\src\\core\\Scheduler.ts`
- ADR 0190: `docs/adr/0190-delinea-headless-chart-engine.md`
- ADR 0191: `docs/adr/0191-delinea-transform-pipeline-and-datazoom-semantics.md`
- Scheduling contract: `docs/adr/0034-timers-animation-and-redraw-scheduling.md`
- Zed/GPUI reference: `F:\\SourceCodes\\Rust\\fret\\repo-ref\\zed\\crates\\gpui\\src\\window.rs` (`Window::request_animation_frame`)
