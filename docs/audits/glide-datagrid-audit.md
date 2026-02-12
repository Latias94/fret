# Glide Data Grid Audit (DataGridCanvas)

Status: Draft

This audit compares Fret’s canvas-backed data grid surface with Glide Data Grid to guide a
“performance ceiling” implementation direction.

## Scope

- Fret surface: `fret-ui-shadcn::DataGrid` (alias for `DataGridCanvas`)
  - Implementation: `ecosystem/fret-ui-shadcn/src/data_grid_canvas.rs`
  - Headless axis math: `ecosystem/fret-ui-headless/src/grid_viewport.rs`
- Reference: Glide Data Grid (repo-ref)
  - Checkout: `repo-ref/glide-data-grid` @ `ab7042389afd`

Non-goals for this doc:

- Full spreadsheet engine (formulas, merge cells, pivots)
- Editing UX parity (we only capture perf-relevant hooks)

## What We Have Today (Fret)

### Rendering model

- Canvas rendering: backgrounds, grid lines, and text are drawn in a small number of paint passes.
- Visible cell list is derived from a 2D viewport computation (`compute_grid_viewport_2d`), then rendered in a nested
  loop over visible rows × visible columns.
- Text draw uses a stable `cache_key` derived from `(row_key, col_key)` (see `DataGridCanvas.cell_text` key scope).

### Virtualization model (2D)

- Row/column axes are modeled as independent 1D virtualizers (`GridAxisMetrics`) with:
  - Fixed sizing (constant estimate per item)
  - Measured sizing (estimate + measurement write-back, keyed by stable axis key)
- A `GridViewport2D` binds those into a visible window via scroll offsets and viewport size.

## What Glide Suggests (When Pushing “Spreadsheet Scale”)

Glide’s core architectural idea is: keep per-cell UI node count ~constant by rendering dense regions via canvas,
and push “rich UI” to lightweight overlays (selection rects, editor popovers).

Key reference anchors:

- Render pipeline helpers:
  - `repo-ref/glide-data-grid/packages/core/src/internal/data-grid/render/data-grid-render.cells.ts`
  - `repo-ref/glide-data-grid/packages/core/src/internal/data-grid/render/data-grid-render.walk.ts`
- Public “on-demand cell” contract:
  - `repo-ref/glide-data-grid/packages/core/src/docs/01-getting-started.stories.tsx` (`getCellContent`)

## Gap List (Fret vs Glide) — Performance-Critical

### 1) Render pipeline granularity (dirty rect / partial redraw)

Glide has a more explicit render pipeline (walkers, draw helpers, and separation between lines/headers/cells).
Fret currently repaints the visible grid region each frame the element is painted.

Follow-up direction:

- Introduce an explicit “render plan” object for the grid region:
  - inputs: viewport + visible axis items + selection state + theme
  - outputs: a compact list of draw ops (or at least a measured timing breakdown)
- Consider incremental redraw (dirty rect) once selection/editing overlays exist.

### 2) Cell renderers / prep cache

Glide has a cell renderer system (including “prep” steps and per-renderer caches) to avoid redoing expensive work.
Fret currently draws only text via a callback and relies on the painter cache key.

Follow-up direction:

- Define a `CellRenderer`-like contract for canvas cells:
  - `draw(ctx, rect, cell, theme, state)`
  - optional `prep` hook for caching per cell type
- Keep the core grid contract `rows + cols + get_cell(row, col)` to make this pluggable.

### 3) Large scroll handling (precision / scaling)

Glide’s walkers and sizing management emphasize stable geometry under large scroll offsets.
Fret scales `Px` into integer units in `GridAxisMetrics`, which is good, but we do not yet have a dedicated
“large scroll scaling” layer for extreme totals (multi-million rows/cols).

Follow-up direction:

- Add an explicit scaling strategy (similar to “scale virtualization math into a stable range”) once we measure
  total sizes large enough to trigger precision issues.

### 4) Variable sizing contract (source vs measured)

Fret supports variable row/col sizing via measurement write-back (`GridAxisMetrics::measure`) and also supports
caller-provided sizes via `DataGridCanvasAxis::size_override`.

The missing piece is a stable, explicit contract for:

- when sizes are authoritative (data-driven) vs best-effort (UI-measured),
- how invalidation works when widths change (text reflow changes height),
- how measurement caches are reset (revision semantics).

## Immediate P0 Fixes Implemented

These are small reductions in per-frame overhead without changing behavior:

- Avoid the “pre-viewport compute” pass when no size overrides are configured.
- Avoid per-frame allocation of index vectors for the visible range (iterate by start/end directly).

Implementation: `ecosystem/fret-ui-shadcn/src/data_grid_canvas.rs`

## Baseline Measurements (Fret)

Benchmark harness:

- Demo: `cargo run -p fret-demo --bin canvas_datagrid_stress_demo --release`
- Script: `tools/bench_canvas_datagrid.py`
- Env: `FRET_CANVAS_GRID_AUTO_SCROLL=1`, `FRET_CANVAS_GRID_EXIT_AFTER_FRAMES=600`, `FRET_CANVAS_GRID_STATS_WINDOW=240`

Environment (sample run):

- Commit: `eafbdda750bc3b8cae3f1c19c49af6531d602600`
- GPU: NVIDIA GeForce RTX 4090 (Vulkan)
- Toolchain: `rustc 1.92.0`

Summary (last rolling window, 240 samples):

| Case | Grid compute avg/p95 (ms) | Renderer `prepare_text` (ms) | Renderer `draws` |
| --- | --- | --- | --- |
| 200k × 200 (fixed) | 0.005 / 0.007 | 10.510 | 33060 |
| 200k × 200 (variable) | 0.006 / 0.006 | 4.140 | 23700 |
| 1m × 200 (fixed) | 0.004 / 0.005 | 4.740 | 33060 |
| 1m × 200 (variable) | 0.006 / 0.008 | 4.120 | 24095 |

Updated baseline (stable, 3 iterations, median; output moved off repo disk):

- Commit: `fd4cae9f539ada0d8c4a1d4caad760dcba3931a4`
- Output: `G:\\sccache\\bench\\canvas-datagrid\\20260115-001530\\summary_agg.csv`

| Case | Visible (rows×cols/cells) | Grid compute median (ms) | Renderer `prepare_text` median (ms) | Renderer `draws` median |
| --- | --- | --- | --- | --- |
| 200k × 200 (fixed) | 39×14 / 546 | 0.006 | 10.600 | 33611 |
| 200k × 200 (variable) | 30×13 / 390 | 0.008 | 8.420 | 23700 |
| 1m × 200 (fixed) | 39×14 / 546 | 0.005 | 10.290 | 33060 |
| 1m × 200 (variable) | 30×13 / 390 | 0.009 | 9.610 | 23700 |

Text caching improvement (shared plain-text blobs, 3 iterations, median):

- Commit: `c9abd0c7c0706f9684e8016442f26cb9458776cc`
- Output: `G:\\sccache\\bench\\canvas-datagrid\\20260115-093254\\summary_agg.csv`

| Case | Grid compute median (ms) | Renderer `prepare_text` median (ms) |
| --- | --- | --- |
| 200k × 200 (fixed) | 0.003 | 4.940 |
| 200k × 200 (variable) | 0.006 | 4.590 |
| 1m × 200 (fixed) | 0.004 | 5.830 |
| 1m × 200 (variable) | 0.006 | 4.710 |

Text draw-call improvement (coalesce adjacent text draws, 3 iterations, median):

- Commit: `3b80e3b4d03a46e850643a7e8a53f5f99cde79b1`
- Output: `G:\\sccache\\bench\\canvas-datagrid\\20260115-100710\\summary_agg.csv`

| Case | Renderer `prepare_text` median (ms) | Renderer `draws` median |
| --- | --- | --- |
| 200k × 200 (fixed) | 4.980 | 366 |
| 200k × 200 (variable) | 5.030 | 366 |
| 1m × 200 (fixed) | 5.240 | 366 |
| 1m × 200 (variable) | 3.980 | 366 |

Interpretation:

- Grid viewport/visible list math is already “in the noise” on high-end hardware.
- The primary bottleneck is renderer-side text preparation and draw call count (not axis math).
- Per-cell cache keying is worst-case under scrolling; we now additionally share plain-text blobs by a content/style
  fingerprint (independent of `(row_key, col_key)`), which significantly reduces `prepare_text` churn.

## Next P0/P1 Tasks (Recommended Order)

P0 (fast, low risk):

- Add cheap instrumentation hooks (counts + timings) to the stress demo:
  - visible rows/cols/cells, time spent computing viewport, time spent drawing text.
- Add an opt-in “cell text cache policy” so large grids can avoid shaping churn when data changes rapidly.

P1 (medium scope):

- Introduce a canvas cell renderer registry (text, number, bool, pill/tag, etc.) with optional prep caching.
- Add overlay layers (selection rect, caret, editor popover) without increasing per-cell node count.
