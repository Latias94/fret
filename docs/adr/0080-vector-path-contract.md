# ADR 0080: Vector Path Contract (PathService + SceneOp::Path)

Status: Accepted

## Context

Fret exposes a prepared vector path API:

- `fret-core::vector_path` (`PathCommand`, `PathStyle`, `PathService`)
- `SceneOp::Path` (render a prepared `PathId` at an origin)

The implementation exists today (`crates/fret-render-wgpu/src/renderer/path.rs` tessellates paths
via `lyon`). This ADR locks the base prepared-path contract so component and renderer work does not
drift across AA expectations, transform interaction, clip composition, metrics, and cache keys.

Follow-on ADRs extend this base contract without changing the v1 compatibility surface:

- ADR 0277 adds `StrokeStyleV2` join/cap/miter/dash semantics.
- ADR 0278 upgrades `SceneOp::Path` to the bounded scene `Paint` surface.

References:

- Display list ordering: `docs/adr/0002-display-list.md`
- Renderer ordering/batching constraints: `docs/adr/0009-renderer-ordering-and-batching.md`
- Shape semantics (quads/shadows/AA): `docs/adr/0030-shape-rendering-and-sdf-semantics.md`
- Scene state stack (transform/opacity/clip): `docs/adr/0019-scene-state-stack-and-layers.md`
- Transform + clip composition (affine v1): `docs/adr/0078-scene-transform-and-clip-composition.md`

## Decision

### 1) Paths are a prepared geometry primitive behind stable IDs

- `PathService::prepare(...) -> (PathId, PathMetrics)` produces a stable handle for a prepared path.
- `SceneOp::Path` draws a prepared path by `PathId` with a bounded `Paint` surface (ADR 0278).
- UI/runtime code does not own tessellation or GPU resources; it only holds `PathId` handles (ADR 0004).

### 2) Coordinate space and transforms

- Path commands are expressed in **logical pixels** in the current local coordinate space.
- `SceneOp::Path { origin, ... }` adds a local translation for the prepared path.
- The cumulative transform stack applies to both the origin and the path geometry (ADR 0019 / ADR 0078).

### 3) Fill and stroke semantics

#### Fill

- `PathStyle::Fill` fills the path interior.
- `FillRule::NonZero` and `FillRule::EvenOdd` match the standard winding semantics used by SVG/canvas.

#### Stroke v1 compatibility

- `PathStyle::Stroke { width }` draws a stroked outline centered on the path.
- `width` is in logical pixels and is clamped to `>= 0`.
- If `width == 0`, the stroke is treated as having no visible coverage.

Stroke joins/caps are intentionally fixed in v1 (to avoid exposing incomplete surface area):

- join: round
- start cap: round
- end cap: round

If the framework needs configurable joins/caps/dashes, it must be added as a v2 extension to the
core contract (new fields/types and updated conformance tests), not as renderer-only behavior.

That v2 extension is now accepted as ADR 0277:

- `PathStyle::StrokeV2(StrokeStyleV2)` keeps width-only v1 callsites valid.
- `StrokeStyleV2` adds join, cap, miter limit, and optional `DashPatternV1`.
- Renderer cache keys and conformance gates must include v2 style fields.

### 4) Metrics and bounds

`PathMetrics.bounds` is a conservative bounds rectangle in logical pixels:

- It must contain all pixels that the renderer may touch for this path and style.
- For strokes, it must include `width / 2` expansion in all directions.

These bounds are used for fast culling and clip/scissor optimizations; underestimating bounds is a
correctness bug.

### 5) Command interpretation (robustness)

To keep the contract tolerant of upstream command streams (SVG conversions, editor tools):

- `MoveTo` begins a new subpath.
- `LineTo/QuadTo/CubicTo` when no subpath is active behave as an implicit `MoveTo(to)`.
- `Close` closes the current subpath if one is active; otherwise it is a no-op.

### 6) Clipping and opacity

- Clip operations (`PushClipRect/PushClipRRect/PopClip`) apply to `SceneOp::Path` like any other draw op.
- Opacity stacks multiply into the path’s color alpha (ADR 0019).

## Consequences

- Component work can rely on stable, portable path behavior without depending on a specific tessellation backend.
- Renderer implementations may evolve (tessellation strategy, AA, caching) without changing UI semantics.
- The contract is compatible with future full-affine rendering and shader-based clipping (ADR 0078).

## Remaining Work

- Keep the base conformance gates extended when new renderer backends or path-style variants are
  added. Current WGPU coverage locks intersecting same-winding overlap fill rules, representative bounds
  conservativeness against tessellated vertices, and transformed path rendering under clips:
  `crates/fret-render-wgpu/tests/path_base_conformance.rs` and
  `crates/fret-render-wgpu/src/renderer/path.rs`
  (`path_metrics_bounds_contain_tessellated_vertices`).
- Additional future path surface should be additive ADR work above this base contract. Stroke
  join/cap/dash is already owned by ADR 0277; path paint is already owned by ADR 0278.
