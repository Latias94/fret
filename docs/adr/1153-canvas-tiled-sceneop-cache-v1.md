# ADR 1153: Canvas Tiled SceneOp Cache (v1)

Status: Proposed
Scope: Ecosystem canvas infrastructure (`ecosystem/fret-canvas`) and retained canvas widgets that emit large quad/path scenes.

## Context

Fret’s retained canvases (ADR 0137) frequently draw large, spatially-local scenes:

- node graphs (background grids, lots of nodes/wires),
- plots (heatmaps / histogram2d with many quads),
- charts (dense marks).

Today, the primary reuse mechanism for “static-ish” paint is whole-layer replay caching
(`SceneOpCache` / subtree replay; ADR 0055). This works well when the view is stable, but it
degenerates for common editor interactions:

- **pan/scroll** changes translation every frame, causing cache keys to miss if they include view
  bounds/translation,
- even when content is mostly unchanged, we end up rebuilding and re-recording a large op vector.

We want a policy-light substrate that enables high hit rates for pan/scroll without forcing all
canvases into a single data model or a second rendering system.

## Decision

### 1) Add a spatially-aware replay cache: `SceneOpTileCache`

Introduce `SceneOpTileCache<K>` in `ecosystem/fret-canvas`:

- records `Vec<SceneOp>` per tile key `K`,
- replays cached ops with a caller-provided translation delta,
- supports simple age/budget pruning.

This cache is intended for “many quads/paths, spatially local, translation-heavy” layers where a
monolithic cache would miss too often.

### 2) Canonical keying guidance: separate content from translation

For tile caching to be effective:

- **tile content key** MUST exclude view translation (pan offsets / view min/max), and instead
  include only:
  - model revision / content hash,
  - zoom-dependent parameters (scale bits, mip level, style knobs),
  - tile coordinate (x/y) and tile size (or a version tag).
- **replay delta** MUST capture the current translation (pan) so cached tile ops land in the right
  place on screen.

This keeps cache hits high across pan/scroll while still invalidating correctly on zoom or content
changes.

### 3) Keep `fret-canvas` policy-light and modular

`ecosystem/fret-canvas` remains a substrate crate (ADR 0137):

- provides generic infrastructure (`cache`, `view`, `scale`, `drag`, `spatial`, `text`, `budget`),
- does not define domain models (node graphs / plots / charts),
- keeps heavier integration optional behind features (e.g. `ui`).

## Consequences

Pros:

- High cache hit rates for pan/scroll in large retained canvases.
- Low-risk adoption: callers can incrementally tile only the heaviest layers (e.g. background grid,
  heatmap quads) without refactoring the whole widget.
- Clear ownership boundary: `fret-canvas` provides mechanisms; ecosystem canvases choose policies.

Cons / Risks:

- Tile caching can increase memory usage (many tiles retained); pruning must be tuned per widget.
- Cached `SceneOp`s may reference renderer-owned resources; callers must ensure resource lifetimes
  outlive cached ops (same caveat as `SceneOpCache`).
- Poor tile-size choices can lead to “wasted ops” (too much offscreen work) or excessive tile
  bookkeeping (too many tiny tiles).

## Implementation Notes (Non-normative)

Initial integrations (evidence anchors):

- Tile cache implementation: `ecosystem/fret-canvas/src/cache/scene_op_tile_cache.rs`
- Node graph background grid (quad-heavy) uses tile caching:
  `ecosystem/fret-node/src/ui/canvas/widget.rs`
- Plot heatmap / histogram2d quads use tile caching fast path:
  `ecosystem/fret-plot/src/retained/layers.rs`
  `ecosystem/fret-plot/src/retained/canvas/mod.rs`

Recommended future follow-ups:

- Add diagnostics hooks (per-window cache stats) for tile caches similar to path/text caches.
- Add common “tile size selection” guidance (e.g. choose a stable power-of-two in logical px and
  keep it constant per layer version).
- Consider a reusable helper for composing `(base_key, TileCoord) -> u64` to reduce per-widget
  boilerplate and avoid accidental inclusion of translation in cache keys.

## References

- Canvas contract and placement: `docs/adr/0137-canvas-widgets-and-interactive-surfaces.md`
- Whole-layer replay caching: `docs/adr/0055-frame-recording-and-subtree-replay-caching.md`
- Render transform + hit testing: `docs/adr/0083-render-transform-hit-testing.md`

