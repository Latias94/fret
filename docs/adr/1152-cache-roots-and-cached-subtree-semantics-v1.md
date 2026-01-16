# ADR 1152: Cache Roots and Cached Subtree Semantics (ViewCache v1)

Status: Proposed

## Context

Fret already implements paint-stream range replay caching (ADR 0055) and model-driven invalidation
propagation (ADR 0051). This provides a strong baseline for editor-scale performance, but we still
lack a first-class, composition-friendly caching unit equivalent to Zed/GPUI's `AnyView::cached`.

Today, `ElementKind::ViewCache` exists as an experimental marker, and the runtime has partial
support for:

- paint-cache gating to boundary nodes (`crates/fret-ui/src/tree/paint.rs`)
- invalidation truncation at boundaries (`UiTree::mark_invalidation` in `crates/fret-ui/src/tree/mod.rs`)
- contained relayout for boundary roots (`crates/fret-ui/src/tree/layout.rs`)

However, the current semantics are not fully locked, especially for nested boundaries. Without a
stable contract, future "fearless" refactors (multi-stream recording, authoring ergonomics) risk
repeated rewrites across `fret-ui`, `fret-runtime`, and ecosystem crates.

References (non-normative):

- Fret paint replay caching: `docs/adr/0055-frame-recording-and-subtree-replay-caching.md`
- Fret invalidation propagation: `docs/adr/0051-model-observation-and-ui-invalidation-propagation.md`
- GPUI cached views + range replay: `repo-ref/zed/crates/gpui/src/view.rs`, `repo-ref/zed/crates/gpui/src/window.rs`
- Planned refactor notes: `docs/workstreams/gpui-parity-refactor.md`

## Decision

### 1) Define a "cache root" as an explicit authoring boundary

When view caching is enabled, the runtime treats `ElementKind::ViewCache` as a **cache root**
(a first-class cached subtree boundary).

Cache roots are intended to be used at "panel/view" granularity (editor-grade surfaces such as
sidebars, panels, toolbars, inspectors, and complex subtrees).

### 2) Cache roots are the only nodes eligible for replay caching in view-cache mode

When `UiTree.view_cache_enabled` is active, the runtime MUST NOT perform paint-stream replay caching
for non-cache-root nodes.

This keeps the caching model compositional and reduces correctness hazards from nested, implicit
caching at many small nodes.

### 3) Observations are uplifted to the nearest cache root

While executing layout/paint for nodes inside a cache root, model/global observations MUST be
attributed to the nearest enclosing cache root (including the cache root node itself).

This makes "cached subtree reacts to data changes" a stable, testable contract:

- A model change invalidates the cache root, not an arbitrary leaf that happened to observe it.
- The runtime can skip leaf execution on cache hits without losing dependency wiring.

### 4) Invalidation containment is allowed, but cache roots must compose correctly (nested boundaries)

Cache roots MAY be nested.

Correctness rule: if a cache root's recorded output contains the output of a descendant cache root,
then the ancestor cache root MUST be considered invalidated whenever the descendant cache root is
invalidated for the relevant stream(s).

Implementation is free to optimize how this propagation is represented, but the observable behavior
must match:

- When a descendant cache root is dirty for paint, the ancestor cache root must not replay a stale
  paint range that includes the old descendant output.
- When a descendant cache root is dirty for layout, the ancestor cache root must not reuse a stale
  layout-dependent recording (future multi-stream work).

### 5) `contained_layout` is an explicit layout containment hint (not a correctness gate)

`ViewCacheProps.contained_layout` indicates that a cache root is intended to be layout-contained
relative to its parent (i.e. it can be relaid out independently, using the last known bounds).

This flag:

- MAY be used to schedule a targeted relayout pass for invalidated cache roots during the final
  layout pass.
- MUST NOT be used to suppress correctness-critical invalidation propagation across cache roots.

### 6) Inspection/probe modes disable view caching

When the UI runtime is in an inspection/probe mode (picking, semantics inspection, scripted
interaction diagnostics), view caching MUST be disabled, consistent with ADR 0055 and GPUI's
approach ("cache is an optimization; introspection correctness comes first").

## Consequences

### Benefits

- Establishes a stable, author-driven caching unit comparable to GPUI's cached views.
- Enables safe nesting of cache boundaries with predictable correctness rules.
- Improves debuggability: "which cache root is dirty and why" becomes observable and attributable.
- Provides a clean bridge to ADR 0055's multi-stream recording direction (prepaint/interaction
  streams can reuse the same cache root identity and propagation rules).

### Trade-offs

- Uplifting observations and propagating invalidation across cache roots can reduce cache hit rates
  if boundaries are too fine-grained or nested excessively.
- The runtime needs explicit metadata for "nearest cache root" ownership; this adds bookkeeping in
  the `GlobalElementId -> NodeId` bridge.

## Implementation Sketch (Non-Normative)

1) Cache root ownership map:
   - Record `cache_root_parent: Option<NodeId>` for each cache root and `nearest_cache_root: Option<NodeId>`
     for each node during mount.
2) Observation uplift:
   - When recording observations in layout/paint, record them against `nearest_cache_root` when present.
3) Invalidation propagation:
   - When a node requests invalidation, mark the nearest cache root dirty for that invalidation.
   - Additionally, walk `cache_root_parent` chain to mark ancestor cache roots dirty for the same invalidation
     categories that affect recording reuse.
4) Replay gate:
   - Only attempt paint replay on cache root nodes (and only when caching is enabled and inspection is inactive).
5) Tests:
   - Nested cache roots: inner model change must invalidate outer cache root (no stale replay).
   - Observation uplift: model change invalidates cache root even if only leaf observed.
   - Inspection mode: caching disabled; subtree executed.

## Rollout Plan

1) Land tests + diagnostics counters for cache roots (hit/miss per root; invalidation fan-out).
2) Implement observation uplift + cache-root invalidation chaining for paint stream.
3) Promote `ViewCache` authoring usage in ecosystem demos (panel-level boundaries).
4) Extend recording to additional streams as they are introduced (ADR 0055 follow-up).

