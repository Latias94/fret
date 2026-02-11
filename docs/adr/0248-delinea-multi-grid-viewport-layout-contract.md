# ADR 0248: `delinea` Multi-Grid Viewport/Layout Contract (Cartesian v1)

Status: Proposed (M1)

## Context

`delinea` already supports multiple grids in the *data/model layer* (`GridSpec` + `AxisSpec.grid`), and the
ECharts option adapter (`fret-chart::echarts`) binds `gridIndex` into that surface. However, the runtime
layout/input contract is still single-viewport:

- `ChartSpec.viewport` / `ChartModel.viewport` represent one plot viewport (in px).
- Marks emission, axisPointer sampling, hit testing, and brush computations assume that viewport.

To render multi-grid charts today, the UI adapter works around this by splitting a multi-grid `ChartSpec`
into multiple single-grid charts, hosting one `ChartEngine` per grid, and laying multiple canvases out in
the UI. This keeps v1 shippable, but it blocks engine-owned multi-grid contracts:

- deterministic per-grid routing for axisPointer/tooltips/brush,
- cross-grid linking semantics (crosshair/zoom/tooltip/brush),
- and conformance gates that can prevent semantic drift while refactors land.

This ADR defines the “hard-to-change” contract surface needed to support multiple grids inside a single
engine instance without pushing pixel/layout semantics into the engine.

## Relationship to Other ADRs

- ADR 0190: headless engine boundary.
- ADR 0192: axis scales + coordinate mapping.
- ADR 0195: interaction and hit testing contract.
- ADR 0196: multi-axis + layout contract (single-grid v1; explicitly calls out multi-grid as a follow-up).
- Workstream tracker: `docs/workstreams/delinea-engine-contract-closure-v1.md` (M1).

## Decision

### 1) The engine consumes a per-grid **plot viewport** (px rect)

The UI host (adapter) provides a **plot viewport** `Rect` for each `GridId`.

Definition:

- “plot viewport” is the rectangle in px used for data-to-pixel mapping and mark emission for that grid.
- It is **not** the full chart bounding box including axis bands, legends, titles, etc.
- It is the multi-grid generalization of the existing single-grid `ChartSpec.viewport` contract.

### 2) Viewports are layout inputs (revision family: layout), not durable spec

Per-grid plot viewports are **layout inputs** and must not cause spec/model re-validation churn.

Contract requirements:

- Changing a grid’s plot viewport bumps the **layout** revision family (not spec/data/visual).
- Layout input changes must remain deterministic and cache-friendly for budgeted stages.

Implementation note (non-normative):

- Prefer an explicit layout patch surface (e.g. a `ChartModelPatch` layout field) rather than encoding
  grid viewports as durable `GridSpec` fields.

### 3) Engine output echoes per-grid viewports for debugging and downstream routing

The engine output must include the resolved per-grid plot viewports so that:

- headless tests can assert routing/layout invariants without a GPU renderer,
- adapters can debug “wrong grid” issues by inspecting output snapshots.

### 4) Routing invariants (v1)

The engine treats each series as belonging to exactly one grid (via its referenced axes).

For any series `S` in grid `G`:

- Marks emission uses `viewport[G]` for coordinate mapping.
- Hit testing and axisPointer sampling for marks in `G` use `viewport[G]`.
- Brush export is scoped to `G` unless a higher-level link policy explicitly aggregates outputs.

The UI adapter remains responsible for *pixel layout policy* (how plot viewports are arranged) and for
*pointer-to-grid routing* (which grid a pointer event belongs to), but the engine owns the semantics once
the grid is identified.

### 5) Linking across grids (v1: opt-in)

Default behavior:

- A pointer interaction (axisPointer sampling, tooltip sampling, brush selection export, hit testing) is
  scoped to a single grid (the grid whose plot viewport contains the pointer, or the grid of the hit
  series when applicable).
- No implicit “cross-grid linking” is performed by default.

Opt-in behavior (contract decision; implementation may land incrementally):

- Cross-grid linking is enabled only when an explicit policy is set (e.g. a `LinkConfig` flag or an
  explicit link rule list). When enabled, the engine may emit link events that are *grid-addressable*
  (either carrying `GridId` directly, or being unambiguous via `SeriesId -> GridId` mapping).
  - v1 evidence: brush selection carries `grid: Option<GridId>` and link events propagate the selection (ADR 0207).
  - v1 evidence: `LinkConfig.brush_x_export_policy` can opt into cross-grid derived X row ranges (ADR 0206).

This keeps v1 deterministic and avoids accidentally coupling grids as we close M1.

### 6) Filter plan ordering (v1)

For order-sensitive transforms (DataZoom/filter plan; ECharts-style “processor” semantics):

- **Within a grid**: steps are applied X-before-Y when both are materialized in the same frame.
- **Across grids**: the engine processes grids in a deterministic order (v1: ascending `GridId`), and
  the plan ordering must remain stable for a stable model/spec.

### 5) Backward compatibility (single-grid)

To keep v1 codepaths simple:

- Single-grid charts continue to work with the existing `ChartSpec.viewport` path.
- Multi-grid charts require per-grid plot viewports for any visible grids; missing viewports are treated as
  an error (preferred) or a well-documented fallback (only if necessary for incremental migration).

## Consequences

- Multi-grid becomes an engine-owned contract, not an adapter hack.
- The UI adapter can stop splitting specs once the engine can host multiple grids.
- Cross-grid linking becomes a policy layer decision (opt-in) built on deterministic per-grid outputs.

## Follow-ups (Workstream M1)

- Add a per-grid viewport/layout carrier to the engine contract surface (model + patch + output).
- Teach marks emission, axisPointer sampling, and hit testing to select the viewport by `GridId`.
- Add a headless regression gate that asserts multi-grid window writes and marks outputs are stable.

## Evidence Anchors (Current v1)

- ECharts `gridIndex` translation: `ecosystem/fret-chart/src/echarts/mod.rs`
- Retained multi-canvas builder: `ecosystem/fret-chart/src/retained/multi_grid.rs`
- Per-grid plot viewport patching: `ecosystem/fret-chart/src/retained/canvas.rs` (`grid_override`)
- Workstream context: `docs/workstreams/delinea-engine-contract-closure-v1.md`
