# ADR 0192: Retained Windowed Surface Hosts (Sliver-Style, GPUI/Flutter-Aligned)

Status: Accepted (v0 fixed-height hosts; extended coverage in progress)

## Context

ADR 0190 establishes the contract for “prepaint-windowed virtual surfaces”: derive a visible window during `prepaint` and
emit per-frame ephemeral items without forcing a full view-cache rerender for scroll-only deltas.

This is straightforward for **paint-driven** surfaces (e.g. a `Scroll` + leaf `Canvas` that paints only the visible rows),
and we already provide an ecosystem building block (`windowed_rows_surface`) that follows this model.

However, for **composable** list/tree/table surfaces backed by `VirtualList`, we currently have a structural limitation:

- the visible set (`VirtualListProps.visible_items`) is derived during declarative render (`ElementContext`),
- view-cache reuse can legitimately skip re-running the declarative render closure for a cache root,
- therefore, on cache-hit frames we cannot “spawn new row subtrees” for a changed visible window without rerendering the
  cache root that owns the `VirtualList`.

In practice this means a scroll delta that crosses a window boundary (outside overscan) can force rerendering a large
cache-root subtree, even though the logical change is local to the windowed surface.

Both GPUI and Flutter avoid this coupling:

- **GPUI**: the window drives dirty views, and cached prepaint/paint ranges are composable at view granularity.
- **Flutter**: `RenderSliver` / `SliverMultiBoxAdaptor` performs incremental child creation/disposal driven by scroll
  constraints during layout/paint; parents do not rebuild to “discover” new children.

We want a Fret-aligned mechanism that:

- keeps the authoring layer declarative (rows remain composable element subtrees),
- but moves “window membership and incremental mount/unmount” into runtime-owned bookkeeping,
- so scroll/window updates do not require rerendering a whole cache root.

## Decision

Introduce a **retained windowed surface host** primitive (“sliver-style host”) that owns the lifecycle of a windowed
child-subtree set independently of the parent’s declarative rerender.

### 1) Host responsibilities

A retained windowed surface host MUST:

- compute the desired visible window during `prepaint` (ADR 0190),
- maintain a retained set of mounted item subtrees keyed by stable item identity,
- incrementally attach/detach item roots as the window changes,
- keep hit-testing and semantics coherent under scroll transforms,
- provide bounded caching / retention policy for off-window items (keep-alive extent).

#### Keep-alive bucket (Flutter-aligned)

To minimize churn during scroll oscillation / backtracking, the host SHOULD maintain a bounded “keep-alive bucket” of
recently-detached item subtrees:

- Keys: stable per-item identity (Fret: `ItemKey`).
- Values: the detached item root `NodeId` (entire subtree).
- Budget: `keep_alive` item count (a hard upper bound).
- Reuse: when an item re-enters the desired window, reuse the keep-alive subtree without re-mounting.
- Eviction: when the budget is exceeded, evict arbitrary items (v1); future iterations may use LRU to better match user
  scroll patterns.

Correctness constraints:

- A kept-alive subtree is **not** reachable from the window/layer roots, so it must be included in the window's explicit
  GC liveness roots (ADR 0191); otherwise cache-hit frames can sweep the subtree as a stale “island”.
- The keep-alive bucket itself is stored in element-local state. If it is only accessed during reconcile, it can be
  dropped by the element-state buffer advance before the next reconcile frame. Retained hosts SHOULD touch the keep-alive
  state key during normal render (and under view-cache scope) so cache-hit frames can keep it alive via recorded
  view-cache state keys.
- Keep-alive reuse MUST preserve the item's stable identity boundaries (no cross-key reuse), and MUST clear any per-item
  ephemeral prepaint outputs when the host's prepaint key changes.

### 2) Authoring surface

The host exposes an authoring API that separates:

- **data identity** (`len`, `items_revision`, `item_key_at`),
- **window policy** (overscan, keep-alive extent),
- and **row rendering** (`render_item` callback).

To be usable from the existing declarative layer, the host MUST support row rendering via a `'static` callback stored in
element-local state (same pattern as `Canvas` paint handlers; ADR 0156), rather than encoding it in cloneable props.

### 3) Fixed/known-height first

The v1 implementation focuses on `Fixed` and `Known` height modes:

- item placement can be derived from `VirtualListMetrics` without measuring item subtrees,
- window updates can be applied without a full parent layout invalidation walk.

Measured/variable-height mode is explicitly deferred; it requires a measurement pipeline that can attach new children and
write sizes back into the virtualizer without destabilizing cache-root placement.

### 4) Cache-root interaction

The host MUST compose with view caching without requiring whole-root rerenders:

- window membership changes are treated as an **ephemeral prepaint update** for the host,
- item subtrees may still rerender when their own state changes, but a scroll-only window change should not imply a
  rerender of the parent cache root.

This requires that the runtime be able to mount/unmount item subtrees without re-running the parent’s render closure.
The host therefore becomes a dedicated runtime boundary that can perform controlled subtree updates.

### 5) Diagnostics and correctness gates

The host MUST expose debug-only signals sufficient to answer:

- “Why did the window change?” (scroll offset / viewport / revision / command),
- “How many items were attached/detached this frame?”
- “Did we reuse off-window items, or recreate them?”

Scripted harnesses (fretboard diag bundles) SHOULD include:

- window telemetry,
- attach/detach counts,
- stale-paint checks for scroll and drag interactions.

## Consequences

- Composable virtualized surfaces (tables, trees, code views with rich rows) can become scroll-stable without forcing
  parent cache-root rerenders on window boundary crossings.
- We introduce a runtime-owned “mini reconciliation” boundary (sliver-style) that can evolve independently from the
  top-level declarative rebuild model (ADR 0028).
- The API surface becomes more explicit: window updates are a first-class runtime concern, not a side-effect of
  rerendering.

## Relationship to existing ADRs

- ADR 0190: defines the windowed-surface contract; this ADR defines the composable retained-host implementation strategy.
- ADR 0191: liveness/GC under view-cache reuse; the host’s keep-alive policy and membership lists should align with the
  “explicit liveness roots” model.
- ADR 0180 / ADR 1152: view-cache gates and cache-root semantics; the host must not rely on implicit rerendering to keep
  liveness or geometry correct.

## References

- GPUI VirtualList (prepaint-driven windowing):
  - `repo-ref/gpui-component/crates/ui/src/virtual_list.rs`
- Flutter slivers / retained element lifecycle:
  - `repo-ref/flutter/packages/flutter/lib/src/widgets/framework.dart` (`_InactiveElements`, `finalizeTree`)
  - `repo-ref/flutter/packages/flutter/lib/src/rendering/sliver_multi_box_adaptor.dart` (`_keepAliveBucket`)
