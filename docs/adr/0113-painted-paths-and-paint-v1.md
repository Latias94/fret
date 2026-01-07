# ADR 0113: Painted Paths and Paint v1 (Dashes, Joins/Caps, Gradients) for Charts and Plots

Status: Proposed
Scope: `fret-core` scene/paint contracts, renderer conformance, and ecosystem consumers (`fret-plot`, `fret-chart`)

## Context

`delinea`/`fret-chart` aim to scale toward ECharts-class visuals. Even in P0/P1, charts commonly require:

- dashed grid lines and reference lines,
- configurable stroke joins/caps (visual fidelity; compatibility with design systems),
- gradients for area fills and emphasis states,
- consistent paint behavior across desktop and wasm.

Fret’s current vector path contract (ADR 0080) intentionally fixes joins/caps (round) and has no dash pattern.
`SceneOp::Path` draws a prepared `PathId` with a solid color only.

This was the right choice for early stabilization, but charts and advanced plots will eventually need richer paint
semantics. If we do not design an extension path early, we risk a large refactor later where:

- plot/chart engines encode paint workarounds (e.g. “dash by emitting many tiny segments”),
- renderers diverge in behavior,
- caches become unstable due to ad-hoc geometry duplication.

## Goals

- Provide a forward-compatible paint contract that charts and plots can target without renderer-specific hacks.
- Preserve the “prepared geometry behind stable IDs” model (ADR 0080).
- Keep the renderer swappable; paint behavior must be part of a stable contract with conformance tests.
- Avoid forcing immediate adoption: consumers can keep using solid paths while the paint surface is rolled out.

## Non-goals (P0)

- Implement every ECharts paint feature immediately (textures, complex patterns).
- Commit to a full CSS/SVG paint model.
- Add plot-specific scene ops.

## Decision

### 1) Extend the path style contract to Path v2 (joins/caps + dash pattern)

We extend `fret-core::vector_path::PathStyle` to support a richer stroke style:

- `StrokeStyleV2` includes:
  - `width: Px`,
  - `join: LineJoin` (`Miter`, `Bevel`, `Round`),
  - `cap: LineCap` (`Butt`, `Square`, `Round`),
  - `miter_limit: f32` (when `join == Miter`),
  - `dash: Option<DashPattern>`.

`DashPattern` is expressed in logical pixels:

- `segments: SmallVec<[Px; N]>` (on/off lengths; even-length required),
- `phase: Px` (dash offset).

Notes:

- Path v1 behavior remains the default (`Round` join/cap, no dash).
- Renderers that do not support dashes must either:
  - degrade deterministically (solid stroke) and expose a debug flag, or
  - implement a reference CPU dashing step behind `PathService` with caching.

### 2) Introduce a paint registry and `PaintId` (Paint v1)

We introduce a small paint contract in `fret-core`:

- `PaintId` is a stable handle produced by `PaintService::prepare(Paint)`.
- `Paint` is one of:
  - `Solid(Color)`,
  - `LinearGradient(LinearGradient)`,
  - `RadialGradient(RadialGradient)`.

Gradient coordinate space:

- Gradients are defined in **local logical pixels** of the draw op (before the global transform stack).
- This keeps gradients stable under scene transforms and enables caching keyed by `(PaintId, transform)` where needed.

### 3) Add painted path draw ops without breaking existing `SceneOp::Path`

We keep `SceneOp::Path` as a convenience for solid colors.

We add a new scene op for painted paths:

- `SceneOp::PaintedPath { path: PathId, origin: Point, fill: Option<PaintId>, stroke: Option<(PaintId, StrokeStyleV2)> }`

This mirrors the separation in modern renderers:

- geometry is prepared and cached (`PathId`),
- paint is prepared and cached (`PaintId`),
- draw op composes them.

### 4) Conformance tests are required for paint semantics

To avoid renderer drift, we add a conformance suite (similar in spirit to existing ADR-driven tests) that validates:

- dash pattern phase behavior and segment lengths (within tolerance),
- join/cap correctness at acute angles and thin strokes,
- gradient interpolation and transform interaction,
- bounds conservativeness (paint does not draw outside declared bounds).

## Performance considerations

- Painted paths must be cache-friendly:
  - For a series line, geometry should be stable across hover/emphasis changes; only paint changes.
- Dashing should not require per-frame path rebuilding:
  - If the renderer cannot dash in the shader, it should dash in a cached CPU pre-pass keyed by `(PathId, StrokeStyleV2)`.
- Gradients should not force per-frame tessellation:
  - Paint changes should not invalidate geometry caches.

## Relationship to charts and plots

- `delinea` and `fret-plot` should model style in terms of future paint capabilities (see ADR 0112).
- In early phases, if `PaintedPath` is not implemented, components may degrade gradients/dashes to solids.
  The degradation must be deterministic and observable in debug builds.

## Alternatives considered

### A) Implement dashes as many tiny quads/segments in component code

Rejected: it creates huge scene op counts, defeats caching, and makes behavior inconsistent across components.

### B) Extend `SceneOp::Path` with many optional fields

Rejected: it conflates geometry and paint concerns and makes the draw op high-entropy.

### C) Add chart-specific scene ops

Rejected: paint semantics are cross-cutting and should be shared by plots, charts, and general UI components.

## Consequences

- Charts and plots gain a clear path to ECharts-class visuals without rewriting engines.
- Renderer work can land incrementally, guarded by conformance tests.
- Existing code can remain on solid paths until the paint surface is fully implemented.

## References

- Vector path contract v1: `docs/adr/0080-vector-path-contract.md`
- Shape/AA semantics: `docs/adr/0030-shape-rendering-and-sdf-semantics.md`
- Renderer ordering and batching: `docs/adr/0009-renderer-ordering-and-batching.md`
- Delenia large-data and rendering prerequisites: `docs/adr/0112-delinea-large-data-and-rendering-contracts.md`
