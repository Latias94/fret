# ADR 0128: `delinea` Headless Chart Engine (ECharts-Inspired, Fret-Native)

Status: Accepted

## Context

Fret already has the rendering substrate needed for charts and plots:

- `SceneOp::Path` + `PathService` tessellation and caching (ADR 0080).
- `SceneOp::Text` + `TextService` for axes, ticks, labels, tooltips (ADR 0006 / ADR 0029).
- Portable clip/transform stacks and layering (ADR 0078, ADR 0081).

We also already have a retained, ImPlot-like plot surface in the ecosystem layer (`ecosystem/fret-plot`)
with a documented architecture and performance baseline (ADR 0096 / ADR 0098).

Separately, we want to support “application charts” that resemble Apache ECharts: rich chart taxonomy,
dataset-driven configuration, and high-level components (legend, tooltip, axis pointer, data zoom, etc.).
We do not want to embed ECharts itself (JS runtime) or rely on DOM/web rendering, because:

- Fret targets a non-DOM runtime and wants consistent behavior across desktop and wasm.
- We want deterministic, testable headless logic like our table headless engine (ADR 0100).
- We want stable performance on large datasets without per-frame reallocation or full geometry rebuilds.

Therefore we introduce a Fret-native, headless chart engine that is **inspired by ECharts concepts**
(dataset + encode + pipeline scheduling + progressive/incremental rendering), but implemented in Rust
and tailored to Fret’s retained and cache-driven model.

## Relationship to Existing ADRs

- ADR 0096 / ADR 0098 define plot crate placement and a retained plot architecture.
  `delinea` is not a replacement for `fret-plot`. It is a headless engine that can power higher-level
  charts and potentially share some math/LOD helpers in the future.
- ADR 0100 is the precedent for “headless engine + UI recipes” in Fret’s ecosystem.
- ADR 0080 is the contract for vector paths and caching used by UI-facing chart/plot widgets.
- ADR 0097 describes the 3D route (viewport surfaces). This ADR keeps `delinea` 2D-only in v1.

## Decision

### 1) Add a dedicated headless chart engine crate: `ecosystem/delinea`

`delinea` is a headless engine that:

- owns chart semantics, model validation, pipeline scheduling, and mark generation,
- is renderer-agnostic (no `wgpu` / `winit` / `fret-render` dependencies),
- is deterministic and unit-testable,
- supports incremental work via a fixed `WorkBudget` (time/points/marks-based), enabling stable UI
  responsiveness on large datasets.

UI crates (e.g. `ecosystem/fret-chart`) bridge `delinea` output into `SceneOp::{Path,Quad,Text}` and apply
Fret themes/tokens.

### 2) Adopt an ECharts-like pipeline shape (but Rust-native)

We adopt the *conceptual* structure from ECharts:

- **Dataset + encode**: series refer to fields/columns; encode maps fields to channels (x/y/color/etc.).
- **Scheduler + tasks**: stages run in a controlled order and can be incremental/progressive.
- **Large-data strategies**: decide when to decimate, when to render progressively, and when hover/tooltip
  should not force a full re-render.

But we do not mirror ECharts APIs 1:1. We keep the surface small and evolvable.

### 3) Keep 2D and 3D separated by contract

`delinea` v1 targets 2D charts rendered via portable scene primitives.
If we add “real 3D”, it should follow the viewport surface route (ADR 0097) and live in a separate crate
(e.g. `fret-chart3d` or a dedicated engine module behind a feature).

## Goals

- Provide a headless core capable of powering chart widgets without depending on renderer backends.
- Scale to large datasets with predictable CPU usage and without unbounded allocations.
- Support stable identity for series/components so interaction state and caches are not index-keyed.
- Enable future “ECharts-class” features (legend, axis pointer, data zoom) without a full redesign.
- Keep Fret’s layering rules intact (no backend types in ecosystem chart logic).

## Non-goals (v1)

- Full Apache ECharts option parity.
- Full ImPlot parity for low-level plot primitives (covered by `fret-plot`).
- 3D depth-correct rendering in the mark pipeline.
- Shipping a public “Vega-Lite” style grammar as the primary API (we may introduce a higher-level DSL later).

## Architecture

### A) Data model: columnar tables + stable IDs

`delinea` uses columnar data (`DataTable`) for hot paths:

- numeric columns are stored as contiguous slices (e.g. `&[f64]`),
- per-series references use stable IDs (`SeriesId`, `AxisId`, `DatasetId`) rather than `Vec` indices,
- revisions (`Revision`) are used for deterministic invalidation (model vs data vs view state).

The current P0 API already uses explicit field references (ECharts-like `encode` semantics):

- each dataset declares a schema via `DatasetSpec.fields`, mapping `FieldId -> column`,
- each series declares a `SeriesEncode` mapping channels (`x`, `y`, `y2`, …) to `FieldId`.

### B) Pipeline: staged computation with incremental budgets

The engine is staged. Each stage:

- declares its inputs (model/data/view state revisions),
- is “dirty” when any dependency changes,
- can do incremental work and yield when out of `WorkBudget`.

This is analogous to ECharts’ `Scheduler` + `Task` system:

- ECharts decides between “normal” and “progressive/incremental” modes per series and dataset size.
- `delinea` decides how much work to do per UI frame via `WorkBudget` and optional per-series policies.

Recommended stage shape (v1):

1. **Model validation / patch application**
2. **Layout and axis windows** (data extents, constraints)
3. **LOD / sampling** (e.g. min/max per pixel, decimation)
4. **Mark generation** (polylines, rects, text)
5. **Hit testing / hover selection** (optional, based on hover point)

### C) Output: stable marks for UI to render

The engine outputs a `MarkTree` and a compact interaction snapshot:

- marks are stable-identity nodes, referencing point ranges in an arena,
- the UI layer turns mark nodes into `PathCommand`s and uses `PathService` caching (ADR 0080),
- hover is computed in the engine to keep hit-test policy deterministic and testable.

### D) Large-data strategy and responsiveness

We follow the principle used by ECharts:

- *Do not* rebuild huge geometry every frame.
- When data is large, support incremental/progressive work and avoid hover forcing a full recompute.

In `delinea`, this becomes:

- `WorkBudget`-bounded stages (points/marks budgets),
- LOD strategies targeting `O(viewport_px_width)` points for monotonic-X series,
- a default “safe” policy: always emit a bounded number of points proportional to viewport pixels.

## Performance and Safety Considerations (P0/P1 Decisions)

### 1) Avoid per-frame allocations in hot paths

- Stages own reusable scratch buffers (e.g. LOD buckets, temporary indices).
- Mark storage uses arenas reused across steps; caches are keyed by `Revision`.

### 2) Keep invalidation precise

- Separate revisions for model structure, marks-affecting properties, and view state.
- Avoid “global dirty” whenever possible; allow series-level dirty if needed.

### 3) Deterministic handling of invalid data

- NaN/Inf samples are skipped and break segments (consistent with many chart libs).
- Axis windows clamp to non-degenerate values.
- Locked ranges and constraints are applied in a deterministic order.

## Naming and Crate Placement

- Headless engine: `ecosystem/delinea` (crate: `delinea`)
- UI bridge: `ecosystem/fret-chart` (crate: `fret-chart`)
- Low-level plot surface remains: `ecosystem/fret-plot` (ADR 0096)

This avoids mixing “framework kernel” responsibilities into `crates/`, and keeps the ecosystem extractable.

## Alternatives Considered

### A) Embed Apache ECharts via a JS runtime (Charming-style)

Pros:

- instant chart type coverage and themes
- proven interactivity model in browsers

Cons for Fret:

- requires a JS runtime (and possibly a DOM-like environment) for interactive rendering
- cross-platform consistency becomes harder (desktop vs wasm vs mobile)
- not aligned with Fret’s retained caching and headless-testable policies

### B) Use only `fret-plot` for everything

Pros:

- fewer crates

Cons:

- `fret-plot` is optimized for ImPlot-like numeric plots; “ECharts-class” charts need richer components
  and dataset/encode semantics.
- conflates low-level plot policy with high-level chart composition.

### C) Put chart engine in `crates/` (kernel)

Rejected: chart semantics are high-entropy and should remain in the ecosystem layer.

## Consequences

- We gain a Fret-native path to “application charts” without embedding external runtimes.
- We create a stable headless surface that can be tested and evolved independently of rendering backends.
- We must actively prevent overlap/duplication between `delinea`, `fret-chart`, and `fret-plot` by keeping
  responsibilities explicit (plot vs chart vs headless helpers).

## Follow-ups

P0 (bootstrap):

- Keep the current minimal `ChartSpec` + `DataTable` design, but enforce stable IDs and validation.
- Provide a small set of series kinds with solid performance baselines (Line/Area/Band).
- Document interaction defaults and axis locking behavior in `fret-chart`.

P1 (ECharts-inspired expansion):

- Extend the transform pipeline beyond continuous row ranges (sparse selection, aggregate, stack) with
  revision-based caching (see ADR 0129).
- Expand first-class components: legend, axis pointer, richer dataZoom (min/max span, zoom lock, slider UI),
  tooltip configuration, and component linking.
- Define a typed “theme mapping” layer: map Fret token keys to chart paints/metrics without hard-coding
  colors in the engine.

## References

- Apache ECharts source (vendored): `repo-ref/echarts`
  - scheduler/task/progressive rendering: `repo-ref/echarts/src/core/Scheduler.ts`
  - source formats and encode semantics: `repo-ref/echarts/src/data/Source.ts`
- Charming (ECharts JSON builder, reference only): `repo-ref/charming`
- Plot crate placement: `docs/adr/0096-plot-widgets-and-crate-placement.md`
- Plot architecture baseline: `docs/adr/0098-plot-architecture-and-performance.md`
- Headless engine precedent (tables): `docs/adr/0100-headless-table-engine.md`
- Vector path contract: `docs/adr/0080-vector-path-contract.md`
