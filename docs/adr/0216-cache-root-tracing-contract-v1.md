# ADR 0216: Cache Root Tracing Contract (ViewCache v1)

Status: Proposed

## Context

Fret's GPUI-style view caching work (ADR 0213) introduces explicit cache-root boundaries (`ElementKind::ViewCache`)
whose correctness depends on invalidation, observation attribution, and subtree reuse behavior.

When the symptom is "why did this frame render" or "why did this cache root rerun layout/paint", ad-hoc logging is
not enough:

- cache hits can skip render but still run layout/paint,
- model/global changes can invalidate specific cache roots,
- inspection/diagnostics can disable caching and change performance characteristics,
- nested cache roots must compose without replaying stale output.

Fret already adopts structured tracing as the primary instrumentation mechanism (ADR 0036) and supports timeline
profiling via Tracy (`tracing` → Tracy layer in `fret-bootstrap`).

This ADR defines a small, stable tracing contract for cache roots so that:

- the Tracy workflow remains reliable across refactors,
- cache-root behavior is observable without bespoke debug builds,
- future tooling (inspector / diag bundles) can correlate frame cost to cache-root decisions.

## Decision

### 1) Canonical span names for cache roots (nested under frame / phase spans)

Cache-root spans use the `ui.cache_root.*` namespace:

- `ui.cache_root.layout` (TRACE): emitted when executing layout for a cache-root node.
- `ui.cache_root.paint` (TRACE): emitted when executing paint for a cache-root node.

Optional / follow-up spans (not required for v1 correctness, but standardized for tooling):

- `ui.cache_root.mount` (TRACE): emitted when mounting a cache-root node and deciding whether to reuse its subtree.
- `ui.cache_root.reuse` (TRACE): emitted as a short span or event describing cache-hit vs cache-miss decisions.

These spans are intended to appear under a per-frame root span and higher-level UI phase spans:

- `fret.frame` (INFO)
- `fret.ui.layout` / `fret.ui.paint` (INFO)

### 2) Span emission and volume controls

Cache-root spans MUST be:

- cheap when disabled (no expensive string allocation),
- gated by tracing level (TRACE) and by whether the node is a cache root,
- present in release builds when the `tracing` feature is enabled.

The recommended "turn it on" workflow is:

- `RUST_LOG="info,fret_ui=trace"` to include cache-root spans,
- avoid enabling `trace` globally.

### 3) Required fields for v1 cache-root spans

`ui.cache_root.layout` MUST include:

- `node`: the retained `NodeId` of the cache root
- `pass`: the layout pass kind (e.g. probe vs final)
- `view_cache_active`: whether the window is currently in view-cache mode
- `contained_layout`: the cache-root hint (`ViewCacheProps.contained_layout`)
- `invalidated`: whether the cache root is considered dirty for the current pass

`ui.cache_root.paint` MUST include:

- `node`: the retained `NodeId` of the cache root
- `view_cache_active`
- `contained_layout`
- `invalidated`

Rationale:

- `node` is stable and cheap; it is the primary identity for tying runtime work to debug stats and diagnostics bundles.
- `invalidated` answers the key question "why did this root do work".
- `view_cache_active` and `contained_layout` explain why caching/replay might be enabled/disabled or constrained.

### 4) Optional fields (best-effort, debug-only when needed)

Cache-root spans MAY include additional fields when available cheaply:

- `window`: window identity (if not already present on an ancestor span)
- `element`: declarative element identity (e.g. `GlobalElementId`) for correlation with element debug paths
- `label`: best-effort debug label (see ADR 0212)
- `cache_hit`: whether mount/reuse used a cached subtree (when emitted from mount/reuse instrumentation)
- `reason`: a coarse reason enum when the invalidation source is known (model/global/scroll-handle/child-root/etc.)

These are optional to keep the core contract stable and low-overhead.

## Consequences

- Timeline profiling can answer "which cache roots were dirty" without bespoke instrumentation.
- GPUI parity work can rely on stable span names and required fields while refactoring internals.
- Diagnostics bundles and scripted interaction tests can correlate slow frames to cache-root work when tracing is enabled.

Known gaps (explicitly allowed in v1):

- per-cache-root hit/miss counters are not part of this contract (tracked by ADR 0213 follow-ups).
- mount/reuse spans are standardized here but may be implemented incrementally.

## References

- ADR 0036: Observability: `docs/adr/0036-observability-tracing-and-ui-inspector-hooks.md`
- ADR 0213: Cache roots: `docs/adr/0213-cache-roots-and-cached-subtree-semantics-v1.md`
- ADR 0212: Element identity debug paths: `docs/adr/0212-element-identity-debug-paths-and-frame-staged-element-state.md`
- Debugging workflow: `docs/debugging-playbook.md`
- Tracy workflow: `docs/tracy.md`
