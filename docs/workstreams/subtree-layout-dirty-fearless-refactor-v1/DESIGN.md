# Design: Subtree Layout Dirty Aggregation (v1)

## Problem statement

We need a **mechanism-level** way to detect “a descendant is layout-dirty” without assuming that
invalidation always bubbles to the nearest ancestor that a caller will check.

Concrete symptom:

- A scrollable page is at (or near) the current `max_offset`.
- A descendant subtree changes its intrinsic height (e.g. switching docs tabs from Preview → Code).
- The scroll container does not update its content extent / `max_offset`, so the user cannot scroll
  further and the bottom content is clipped.

This can happen when the scroll implementation relies on cached extent probes and decides to reuse
the cache because the **direct child root** does not appear to need layout.

## Goals

1. Provide an O(1) query: `subtree_layout_dirty(node) -> bool`.
2. Keep the existing invalidation semantics intact (view-cache truncation, containment, etc.).
3. Avoid deep subtree scans in hot paths (scroll layout, view-cache reconciliation).
4. Make the mechanism resilient to “partial bubbling” failures: even if an ancestor’s
   `invalidation.layout` is false, it can still know “a descendant is dirty”.

## Proposed mechanism

Introduce a per-node counter (or bit) stored in `UiTree` node storage:

- `subtree_layout_dirty_count: u32`

Definition:

> The number of nodes in this node’s subtree (including self) whose `invalidation.layout == true`.

Then:

- `subtree_layout_dirty(node) = subtree_layout_dirty_count(node) > 0`

This is an **aggregation** structure, not an invalidation. It does not imply we must relayout the
node; it only provides a reliable signal for systems that need to avoid reusing stale caches.

### Why a counter (not a bool)?

- A counter supports nested transitions without recomputation:
  - mark a second descendant dirty ⇒ count increments
  - clear one dirty descendant after layout ⇒ count decrements
- It avoids “lost set” bugs (bools can be accidentally overwritten by competing updates).

## Update rules

The counter must be updated when a node’s `invalidation.layout` transitions.

### Marking layout invalidation (false → true)

When `mark_invalidation(...)` causes a node’s `invalidation.layout` to transition `false -> true`:

- increment `subtree_layout_dirty_count` for:
  - the node itself
  - and all its ancestors up to the nearest root

### Clearing layout invalidation (true → false)

When the layout pass clears `invalidation.layout` on a node (after successful measurement/layout):

- decrement `subtree_layout_dirty_count` for:
  - the node itself
  - and all its ancestors

### Boundary semantics: contained view-cache roots

View-cache truncation (especially `contained_layout`) is a *contract* about which parts of the tree
may be laid out independently.

The aggregation is a correctness signal for cache reuse decisions, but it must **not** accidentally
break contained-layout semantics by forcing ancestor relayouts to “pierce” cache boundaries.

Current v1 stance:

- We allow aggregation to be computed independently of invalidation truncation (so consumers can
  detect “descendant dirty” even if a local ancestor bit is unexpectedly clear).
- We do **not** use `subtree_layout_dirty` to force ancestor relayouts in the generic layout loop.
  Contained cache roots remain owned by the contained relayout pass.

If we later want to use subtree-dirty to drive generic relayout decisions, we must define an
explicit policy such as “subtree dirty excluding contained cache roots” (see open questions).

## Performance strategy

Naively walking to the root on every transition is O(height) per change and may be too expensive
for high-frequency invalidation sources (typing, selection, animations).

We propose a two-tier propagation strategy:

1. **Eager to the nearest cache root (fast path)**:
   - walk parent pointers up until the first view-cache boundary (or tree root),
   - update counters along this short path.
2. **Deferred propagation across cache roots (amortized)**:
   - record a pending delta on the cache root, and schedule one “propagate deltas upward” repair
     pass once per frame (or per tick) to apply those deltas to ancestors above the cache root.

This preserves the performance intent of truncation while still ensuring that “subtree dirty” can
be observed at higher levels when needed.

### Alternative (simpler, higher risk)

Always propagate to the root immediately. This is simplest but may be too costly in real apps.

The workstream should land the mechanism behind a runtime flag first so we can measure the real
cost and choose between eager-only vs deferred.

### Current implementation (v1)

We currently ship an **eager-to-root** implementation with two optimizations:

1. **Invalidation walks update aggregation in a single pass** (no nested “walk-to-root per node”):
   - while walking ancestors to mark invalidation, maintain a `pending_delta` that represents “how
     many newly-dirty nodes exist below the current node”
   - apply `pending_delta + self_transition_delta` to each visited node’s counter
2. **Aggregation propagation is not truncated by view-cache boundaries**:
   - invalidation marking may truncate at view-cache roots
   - aggregation continues walking parent pointers above the truncation point and applies the
     pending delta without marking invalidation bits

This preserves correctness for cache reuse decisions while keeping the hot path O(height) per
invalidation root.

## API surface (internal)

Add internal helpers on `UiTree` (non-public, mechanism-level):

- `node_subtree_layout_dirty(node) -> bool`
- `node_subtree_layout_dirty_count(node) -> u32` (debug-only)

## Integration points

### Invalidation marking

- `crates/fret-ui/src/tree/ui_tree_invalidation_walk/mark.rs`
  - detect per-node transitions for `invalidation.layout` and update the aggregation structure.
  - keep existing view-cache truncation semantics intact for `invalidation.*`.

### Invalidation clearing

- `crates/fret-ui/src/tree/layout/node.rs`
  - when `n.invalidation.layout` is cleared after layout, update the aggregation structure.

### Scroll fast path (consumer)

Replace any “deep scan” workaround with:

> At the scroll edge, if the direct child root is *subtree-dirty*, treat it as dirty for the
> purposes of scroll extent probing.

Note: the intended contract is **not** “always relayout ancestors when any descendant is dirty”.
Instead, scroll uses the subtree-dirty signal as a guard to avoid reusing stale content-extent
caches when the user is at the scroll extent edge.

## Correctness invariants (debug asserts)

We should be able to assert (in debug builds) that:

- `subtree_layout_dirty_count(node) == (self_layout_dirty(node) as u32) + sum(children counts)`

This can be checked:

- opportunistically for changed nodes only, or
- via a diagnostic command that validates the tree after a frame.

## Testing strategy

- Unit tests in `crates/fret-ui/src/declarative/tests/layout/scroll.rs`:
  - ensure scroll extents update when a descendant changes while at the end.
  - include a test that simulates a “lost bubbling” failure mode and verifies the aggregation
    signal still forces the correct extent refresh.
- Optional diag script:
  - UI Gallery: scroll to bottom, switch Preview → Code on a bottom section, ensure `max_offset`
    increases and the section becomes fully scrollable.

## Rollout plan

See `MILESTONES.md` for a gated rollout:

1. Land aggregation as a no-op (collect-only) behind a flag + add debug validation.
2. Switch scroll to consume the new API and remove workarounds.
3. Expand usage to other cache-sensitive systems (optional).

## Future refactor directions (beyond v1)

This workstream is intentionally scoped to a **mechanism primitive**. If we keep the consumer-facing
query stable (`subtree_layout_dirty(node) -> bool`), we can evolve the internal implementation as the
architecture shifts.

### Direction A — GPUI-style “rebuild every frame” authoring

Long-term, the repo aims toward a GPUI-style model where element trees are rebuilt each frame and
cross-frame state is externalized.

Implications:

- Dirty bookkeeping tends to shift from “bit toggles + decrements” toward “frame epochs/stamps”.
- The v1 counter can remain correct and useful, but a v2 implementation may replace counters with an
  epoch-based aggregation while keeping the consumer API unchanged.

### Direction B — Cache-root repair passes as a first-class subsystem

If view-cache boundaries become the primary performance lever, we will likely want a single
coordinated “repair pass” per frame responsible for:

- applying deferred aggregation deltas across cache roots,
- validating cache-root invariants (optional debug mode),
- producing per-cache-root telemetry.

This would reduce ad-hoc “fix-up” logic in individual widgets (scroll, virtual lists, etc.).

### Direction C — Unified “subtree dirty” channels (layout, paint, a11y)

Once the layout channel is proven, we can consider parallel aggregation channels for:

- paint invalidation (“descendant needs repaint but ancestor looks clean”),
- accessibility invalidation (a11y tree changes),
- hit-test caches (pointer input regions).

We should only do this if there is a demonstrated correctness/perf benefit; the v1 workstream should
stay layout-only.

### Direction D — Explicit layout contracts at mechanism boundaries

Many “content grew but extent didn’t” bugs are symptoms of implicit contracts:

- which nodes are allowed to reuse cached sizes,
- which boundaries truncate invalidation,
- what “contained layout” means for scroll/virtualization.

After v1 lands, we should fold learnings into:

- the runtime contract matrix (`docs/runtime-contract-matrix.md`), and/or
- an ADR that defines the minimal “cache reuse correctness” contract for scroll/virtualized
  containers.
