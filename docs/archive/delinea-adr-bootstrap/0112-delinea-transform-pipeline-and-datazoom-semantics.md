# ADR 0129: `delinea` Transform Pipeline + DataZoom Semantics (ECharts-Inspired)

Status: Proposed

## Context

`delinea` is the headless, renderer-agnostic chart engine intended to support “application charts” in Fret
with an ECharts-inspired mental model (dataset + encode + transform pipeline + components like dataZoom).

We already have:

- A portable rendering substrate (`SceneOp::{Path,Quad,Text}` with caching) sufficient for 2D charts.
- A retained UI bridge (`ecosystem/fret-chart`) that forwards input as headless actions.
- A bootstrap `delinea` engine that produces stable marks, hit testing, and large-data LOD.

The next architectural risk is **drift**: adding chart features ad-hoc can create overlapping responsibilities
between the headless engine, the UI widget, and renderer primitives, causing future refactors to be expensive.

Apache ECharts provides several “battle-tested” conceptual constraints we want to preserve:

- A scheduler/pipeline that can run progressively/incrementally when data is large.
- Dataset-driven dataflow with transform caching.
- DataZoom with explicit filtering modes and documented ordering constraints (X filtering can affect Y extents).

This ADR locks down a minimal, Fret-native version of those constraints so we can evolve towards “ECharts-class”
features without destabilizing core contracts.

## Relationship to Other ADRs

- ADR 0128 introduces `delinea` as the headless chart engine and sets scope boundaries.
- ADR 0096 / ADR 0098 cover the ImPlot-like retained plot surface (`fret-plot`), which is not replaced by `delinea`.
- ADR 0080 defines `SceneOp::Path` and `PathService` caching; this ADR lists chart-driven “must-have” semantics
  that may require future contract extensions.

## Decision

### 1) Treat chart features as first-class spec/model nodes, not widget-only state

If a feature is intended to be durable and serializable (e.g. “this axis has dataZoom behavior with filterMode”),
it must live in the chart spec/model (`delinea::spec` / `delinea::engine::model`), not only as UI widget state.

UI widgets may still maintain *pure UI policy* (temporary locks, gesture preferences), but the engine owns:

- component identity and configuration defaults,
- model validation (e.g. invalid axis references),
- deterministic state evolution given actions/patches.

### 2) Use an internal transform pipeline with revision-based caching

We adopt an ECharts-like separation:

- **Source / dataset**: immutable columnar storage (`DataTable`) with a stable schema (`FieldId -> column`).
- **Transforms**: pure-ish nodes that produce:
  - `RowSelection` (range/sparse selection; P0 is range-only),
  - derived columns (P1+; e.g. stack bases, aggregates),
  - LOD views (bounded point sets).
- **View**: per-series view inputs derived from transforms and axis windows.
- **Marks**: stable mark output that the UI layer maps to `SceneOp`.

Transform outputs are cached by:

- upstream dataset revision,
- model revision (encode/series config changes),
- view revision (window changes),
- transform parameters (e.g. filterMode, window, downsample budget).

This is the headless equivalent of ECharts’ `SourceManager` + `Scheduler` caching, but expressed via explicit
revisions to keep behavior deterministic and testable.

### 3) DataZoom is a component with explicit axis bindings (v1: one per axis)

We model DataZoom as a first-class component node:

- Each data zoom has a stable `DataZoomId`.
- Each data zoom targets exactly one axis.
- In v1, **at most one** data zoom may target a given axis (to keep composition rules simple and predictable).

This is stricter than ECharts (where multiple dataZoom models may share an `AxisProxy`), but avoids early
complexity. If we later need multiple zoom components per axis, we will introduce an explicit composition
rule (e.g. “first wins”, “intersection”, “stacked constraints”) rather than implicit ordering.

### 4) Lock down `FilterMode` semantics as a subset of ECharts

ECharts supports several dataZoom filtering modes (`filter`, `weakFilter`, `empty`, `none`).
In `delinea` v1 we standardize on a minimal subset:

- `FilterMode::Filter` (ECharts-like `filter`):
  - the current data window is used to slice rows (when monotonic-X heuristics allow),
  - bounds/LOD are computed on the filtered selection,
  - Y auto-scales to visible X by default (good “plot” ergonomics and performance).
- `FilterMode::None` (ECharts-like `none`):
  - no row slicing occurs for the data window,
  - bounds/LOD remain global (can keep global Y scale while zooming X),
  - this mode can be significantly more expensive on large datasets.

We intentionally do **not** implement ECharts’ `weakFilter` / `empty` in v1 because their semantics depend on
series types (stacking, category axes, multi-dimensional filtering) and can easily become inconsistent if
introduced without a full component model.

We keep the design open to adding them later without reworking the model surface:

- by extending `FilterMode` with additional variants,
- by implementing them as transform nodes (rather than special-casing in view/marks).

### 5) Define ordering constraints for future multi-zoom and multi-axis filtering

ECharts documents that dataZoom filtering is order-sensitive (e.g. filtering X first can change the data extent
used by Y’s zoom calculations). We adopt the same high-level rule:

- When multiple zoom/filter transforms exist across dimensions, apply X filters before Y filters inside a grid.

In v1 this is mostly a forward-compatibility constraint (we primarily support a single X zoom), but it prevents
us from designing transforms that cannot be composed later.

### 6) Chart-driven renderer contract (what `SceneOp::Path` must support well)

Charts stress a narrower set of geometry primitives than general UI, but they stress them *hard*:

- Long polylines and filled areas with clip rects.
- Many repeated primitives (ticks/gridlines) with stable caching keys.
- Very frequent small updates (window changes) where we want to reuse cached tessellation.

We require the following to remain “first-class fast paths”:

- Polyline + polygon fill paths with deterministic tessellation.
- Clip-rect correctness (no bleed outside plot rect).
- Stable caching keyed by path content + stroke/fill style + transform/scale.

If we later need richer paint semantics (dashes, joins/caps control, gradients, patterns), we will:

- extend `SceneOp::Path` / path paint contracts in a dedicated ADR,
- add conformance tests to prevent backend divergence,
- avoid adding chart-specific scene ops unless we can prove a general-purpose need.

## Consequences

- `delinea` remains the single source of truth for durable chart semantics (components + dataflow).
- `fret-chart` remains a UI bridge and interaction policy layer, not a semantics store.
- We have an explicit path to add ECharts-class features (dataset transforms, dataZoom slider, axis pointer)
  without rewriting the engine’s core invalidation model.
- We accept that some ECharts behaviors will not be supported in v1 until we design consistent, composable
  transform semantics for them.

## Follow-ups

- ADR 0130: axis scales + coordinate mapping (Value/Category v1).
- ADR 0131: marks output + stable identity + renderer contract.
- ADR 0132: large data + progressive rendering baseline.
- ADR 0133: interaction + hit testing contract (axis lock / zoom lock path).

- Implement `DataZoomX` as a transform node rather than ad-hoc view policy (internal-only first).
- Add `minSpan/maxSpan` and `zoomLock`-like constraints (spec-level) once we have multi-axis + slider UI.
- Add series-type-specific transforms (stack, aggregate) with derived columns and cached outputs.
- Evaluate adding `FilterMode::{WeakFilter,Empty}` only after stacking and category axes have a clear contract.
