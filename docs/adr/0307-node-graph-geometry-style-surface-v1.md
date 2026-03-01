# ADR 0307: Node Graph Geometry-Affecting Style Surface (v1)

Status: Proposed

## Context

`ecosystem/fret-node` provides an editor-grade node graph canvas. v1 of the style workstream
introduced `NodeGraphSkin` as a **UI-only, paint-first** surface:

- per-node/per-edge/per-port chrome hints,
- explicit `revision()` invalidation,
- conformance tests ensuring skin changes are paint-only and do not rebuild derived geometry.

Today `NodeGraphStyle` is a monolithic bundle that mixes:

- paint-only tokens (colors, dashes, shadows),
- geometry-affecting tokens (padding, header height, pin metrics, hit-testing widths).

This creates two problems:

1) **No contracted invalidation for geometry-affecting style changes**
   - derived geometry caches and spatial indexes must rebuild when geometry tokens change.

2) **Ecosystem alignment**
   - reference systems like XyFlow support both global theming and per-entity overrides (including
     width/height), which implies a clear separation between “visual paint” vs “layout / geometry”.

Fret is docs/ADR-driven and prefers locking hard-to-change contracts early to avoid late rewrites.

## Decision

### D1 — Split node graph style into two planes: paint vs geometry

Define two explicit token bundles:

- **Paint tokens** (paint-only):
  - colors,
  - dash patterns,
  - shadow/glow parameters,
  - stroke widths that only affect visuals (hit-testing uses separate tokens).

- **Geometry tokens** (geometry-affecting):
  - node sizing metrics (width, header height, padding),
  - port metrics (row height, radius / anchor offsets),
  - hit-testing widths (edge interaction width, port hit slop),
  - overlay layout metrics (minimap margin, controls panel sizes, etc).

Each plane must have its own revision/fingerprint:

- `paint_fingerprint`: invalidates paint caches / scene op caches.
- `geometry_fingerprint`: invalidates derived geometry + spatial index + routing anchors.

### D2 — Preserve the v1 skin contract: `NodeGraphSkin` stays paint-only

`NodeGraphSkin` remains UI-only and paint-first:

- It can override paint tokens per node/edge/port.
- It must not change hit-testing or derived geometry.

If a feature needs geometry changes, it must go through the geometry style surface (D1), not the
skin surface.

Rationale:

- keeps “preset switching” fast and cache-safe,
- avoids silent geometry churn caused by policy code,
- keeps the mechanism/policy layering clean (skin is policy-like, geometry tokens are stable
  configuration).

### D3 — Geometry caches are keyed by geometry fingerprint

All caches that affect:

- node bounds,
- port anchors,
- edge routing anchors,
- hit testing / spatial index,

must include `geometry_fingerprint` in their cache key.

Paint caches (scene ops, tiles, etc) may include `paint_fingerprint` and skin revision as needed,
but must not depend on geometry-only changes unless those changes affect emitted ops.

### D4 — Define units explicitly (canvas space vs screen space)

To avoid drift and “semantic zoom” bugs, geometry tokens must clearly specify their space:

- **Canvas-space** metrics are expressed in canvas units at zoom=1 (scale with zoom).
  - examples: node width, header height, node padding.

- **Screen-space** metrics are expressed in logical px and are stable under zoom.
  - examples: edge interaction width, hit slop.

Paint-only effects that must remain stable under zoom (rings, glow blur radius) remain screen-space
and are part of the paint plane.

### D5 — Leave room for per-entity geometry overrides (v2 milestone)

Per-entity geometry overrides (like XyFlow `node.style.width/height`, `edge.interactionWidth`) are
useful but need a clear contract:

- they must be UI-only (not serialized into `Graph`),
- they must participate in `geometry_fingerprint` invalidation,
- they must have deterministic resolution order.

This ADR allows adding a second, optional “geometry override provider” surface later without
changing the core split introduced here.

## Consequences

- Breaking refactor is acceptable (repo is not open-source yet). We prefer a clean contract over
  compatibility shims.
- `NodeGraphStyle` will likely be replaced or renamed into separate bundles; public API names should
  be neutral (no upstream product names in the surface).
- New conformance tests are required to lock invalidation behavior.

## Alternatives considered

### A1 — Allow `NodeGraphSkin` to affect geometry and bump `revision()`

Rejected:

- mixes paint-only and geometry-affecting concerns in a single “policy” surface,
- makes it easy to accidentally invalidate expensive caches with paint-only changes,
- weakens the v1 contract and makes future maintenance harder.

### A2 — Keep a monolithic `NodeGraphStyle` and fingerprint everything

Possible but not preferred:

- simpler implementation, but encourages accidental geometry-affecting changes without explicit
  review,
- blurs which caches should rebuild, and makes performance tuning harder.

## Implementation sketch (non-normative)

Likely shape (exact naming may vary):

- `NodeGraphPaintTokensV1 { ... }` with `fingerprint() -> u64`
- `NodeGraphGeometryTokensV1 { ... }` with `fingerprint() -> u64`
- `NodeGraphStyleV2 { paint: ..., geometry: ... }` or a pair passed explicitly
- Update `GeometryCacheKey` to include `geometry_fingerprint`
- Add conformance tests:
  - geometry token change rebuilds derived geometry + spatial index,
  - paint token change does not.

