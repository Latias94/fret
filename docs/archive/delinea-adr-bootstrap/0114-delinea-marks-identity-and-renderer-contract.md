# ADR 0131: `delinea` Marks + Stable Identity + Renderer Contract (2D Cartesian v1)

Status: Proposed

## Context

`delinea` is a headless engine and must remain renderer-agnostic (no `wgpu`/`winit` dependencies).
At the same time, to reach “commercial-grade” charting, we need:

- stable identity for series and marks (no index-keyed UI state),
- a clean boundary between headless semantics and UI rendering,
- a mark/output format that supports large datasets and incremental updates without per-frame churn.

ECharts conceptually produces a display list of ZRender elements; we produce portable `SceneOp::*`
through a UI adapter (`fret-chart`). This ADR defines the contract between them.

## Relationship to Other ADRs

- ADR 0128: introduces `delinea` and establishes “headless engine + UI adapter”.
- ADR 0129: transform pipeline and dataZoom ordering.
- ADR 0080: vector path contract (portable scene primitives).
- ADR 0096 / ADR 0098: plot widgets (different API; some math/LOD ideas may be shared later).

## Decision

### 1) Engine outputs a small set of mark primitives, not renderer-specific ops

The `delinea` engine outputs a stable set of mark primitives, expressed in **chart-local pixel space**
(within the chart’s plot rect), and a small amount of layout metadata:

- plot rect, axis rects, and clip rects
- axis windows (for mapping and UI overlays)
- a list of mark batches grouped by “layer role” (background/axes/series/overlays)

The UI adapter (`fret-chart`) is responsible for mapping these marks into `SceneOp::{Path,Quad,Text}`
and applying theme tokens (colors, fonts, corner radii, etc.).

This keeps:

- headless logic deterministic and testable,
- renderer details localized to the adapter,
- future renderer improvements (e.g. better path caching) independent of chart semantics.

### 2) Stable identity is required at the mark level

Stable identity is required for:

- legend toggles,
- hover state,
- pinned tooltips,
- cached geometry / LOD buffers.

Contract:

- Series are identified by stable `SeriesId` (already in place).
- Each mark batch carries a stable `MarkKey` composed of:
  - `series_id`
  - `kind` (line/area/bar/etc.)
  - an optional “part” (e.g. `stroke` vs `fill`, or `baseline`)
  - optional data identity hints (e.g. `item_index_range` for progressive chunks)

The UI adapter must not key caches by `Vec` indices.

### 3) Styling uses semantic roles and light per-series parameters

The engine does not own theme resolution. Instead, it emits:

- semantic style roles (e.g. `AxisGridLine`, `AxisLabel`, `SeriesStroke`, `SeriesFill`, `Crosshair`)
- minimal per-series parameters (e.g. “palette index” or “series color hint”)

The adapter resolves these roles using the active Fret theme token registry (ADR 0101 / ADR 0050).

### 4) Path semantics required for charts are explicit follow-ups, not hidden assumptions

Charts commonly need:

- dashed strokes,
- join/cap control,
- closed polygon fill rules,
- clip rect correctness,
- consistent pixel snapping rules for crisp 1px lines.

If our current portable primitives are insufficient, we evolve them by extending ADR 0080 and/or
introducing a paint-style ADR. We do not smuggle chart-specific behavior into `fret-render`.

## Consequences

- `delinea` can remain headless and testable while still producing render-ready marks.
- The adapter (`fret-chart`) becomes the single location for theme/token usage and renderer integration.
- Stable identity prevents future rewrite when we add animation, progressive rendering, or multi-grid.

## Follow-ups

P0:

- Add a dedicated mark output module (if needed) to keep the output surface small and stable.
- Ensure all existing outputs (axis pointer/tooltip) are keyed by stable series identity.

P1:

- Add bar/rect marks (depends on ADR 0130 category semantics).
- Add multi-grid/multi-axis layout decisions (see ADR 0134).
- Evaluate whether `SceneOp::Path` needs dash/join/cap upgrades for chart parity (ADR 0080 follow-up).

## References

- ADR 0109: `docs/archive/delinea-adr-bootstrap/0190-delinea-headless-chart-engine.md`
- ADR 0110: `docs/archive/delinea-adr-bootstrap/0191-delinea-transform-pipeline-and-datazoom-semantics.md`
- ADR 0080: `docs/adr/0080-vector-path-contract.md`
- ECharts scheduler/progressive pipeline: `F:\\SourceCodes\\Rust\\fret\\repo-ref\\echarts\\src\\core\\Scheduler.ts`
