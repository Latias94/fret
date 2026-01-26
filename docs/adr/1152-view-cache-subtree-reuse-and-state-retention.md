# ADR 1152: View Cache Subtree Reuse and Element State Retention

Status: Accepted

## Context

Fret already has:

- a declarative, per-frame `AnyElement` tree (ADR 0028 / ADR 0039),
- a retained `UiTree` with per-node invalidation and a paint cache,
- an experimental `ViewCache` wrapper element (`ViewCacheProps`) intended to support GPUI-style view caching experiments.

However, the existing `ViewCache` wrapper only marks cache boundaries for `UiTree`-level invalidation containment and paint-cache gating.
The authoring/runtime layer still re-executes the declarative child render closure every frame, even when the subtree is clean.

GPUI’s user-perceived “snappiness” relies heavily on being able to skip view execution on cache hits while keeping:

- the mounted subtree (node identities) stable,
- per-element state alive across frames,
- per-frame registries (e.g. scroll handle bindings) consistent.

## Decision

When `UiTree` view-cache mode is active and a `ViewCache` node is a clean cache root (no layout/paint invalidation),
the declarative runtime may reuse the previously-mounted subtree and skip re-running the child render closure.

To preserve correctness and developer experience:

1. Cache-hit decisions are based on `UiTree` invalidation state of the existing cache-root `NodeId`.
2. Cache hits must keep per-element state alive by touching the recorded state keys for the cache root.
3. Mount must reuse the existing retained subtree and re-collect per-frame scroll handle bindings for the reused subtree.
4. Cache hits must keep dependency tracking alive by inheriting previously recorded per-element model/global observations for the reused subtree.

## Detailed Design

### 1) Cache-hit predicate

`UiTree::should_reuse_view_cache_node(NodeId)` returns true when:

- view-cache mode is active for the frame (`view_cache_enabled && !inspection_active`),
- the node is marked as a view-cache root (`node.view_cache.enabled`),
- the node is clean with respect to layout/paint invalidation.

### 2) Declarative execution behavior

`ElementContext::view_cache(...)`:

- resolves the cache root’s current `NodeId` via `WindowElementState::node_entry(...)`,
- consults the per-frame `should_reuse_view_cache_node` hook,
- on cache hit:
  - records the root in `WindowElementState.view_cache_reuse_roots`,
  - touches the previously-recorded `(GlobalElementId, TypeId)` state keys for that root,
  - returns a `ViewCache` element with an empty `children` list (mount will reuse the existing subtree).
- on cache miss:
  - executes the child render closure,
  - records state key accesses under the view-cache root for use on future cache hits.

### 3) Mount-time subtree reuse

When mounting a `ViewCache` element whose `GlobalElementId` is marked as reused for this frame:

- do not remount children from the (empty) element tree,
- keep the existing retained `UiTree` children unchanged,
- mark the entire subtree “seen” for the current frame (`last_seen_frame`) to avoid GC,
- inherit previously recorded model/global observations for the subtree so model changes continue to invalidate correctly without re-running render,
- re-collect scroll handle bindings from the existing subtree using the persisted `ElementFrame` instance records.

Note: the authoritative lifetime/GC contract under reuse is defined by `docs/adr/0191-declarative-liveness-roots-and-gc-under-view-cache-reuse.md`. In particular, cache-hit frames must preserve liveness deterministically via explicit liveness roots and subtree membership lists; "seen this frame" is not a sufficient semantic signal by itself.

### 4) Per-frame `ElementFrame` persistence

`ElementFrame`’s `WindowFrame.instances` / `children` are treated as a persistent retained snapshot across frames:

- they are no longer cleared purely due to `frame_id` advancing,
- entries are explicitly removed when `UiTree::remove_subtree(...)` deletes nodes during GC.

This enables mount-time reuse to consult instance metadata (e.g. scroll handle bindings) without re-running render.

## Consequences / Tradeoffs

- This is an experimental mechanism-only behavior intended for GPUI parity workstreams; it is not a stable authoring API yet.
- Correctness depends on invalidation being propagated before the next frame’s declarative render begins.
- The current implementation focuses on:
  - skipping child render closure execution,
  - keeping the mounted subtree stable,
  - keeping state alive via recorded state keys,
  - scroll-handle binding continuity.
  Additional per-frame registries may need similar treatment as they are introduced.

## Implementation Notes

Evidence anchors:

- Cache-hit predicate: `crates/fret-ui/src/tree/mod.rs` (`UiTree::should_reuse_view_cache_node`)
- Render-time reuse + state touch: `crates/fret-ui/src/elements/cx.rs` (`ElementContext::view_cache`)
- State key tracking + touch helpers: `crates/fret-ui/src/elements/runtime.rs` (`WindowElementState::*view_cache*`)
- Mount-time subtree reuse: `crates/fret-ui/src/declarative/mount.rs` (`mount_element`, subtree helpers)
- Conformance test: `crates/fret-ui/src/declarative/tests/view_cache.rs`
  - Notes: cache-hit frames are asserted to preserve painted scene ops, semantics output, and hit targets.
  - Notes: includes modal overlay (barrier root) input-gating outcomes under view-cache reuse.
