---
type: Subagent Finding
title: Phase 2 U5 observation fanout audit
tags: fret,ui,observation,view-boundary,phase2,subagent
timestamp: 2026-07-02
related_plan: docs/plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md
git_branch: feat/ui-framework-phase2-refactor
subagent_id: 019f23d7-039d-72c2-a13f-d32e2295ad45
status: complete
---

# Finding

The remaining view-boundary bridge after the boundary layout candidate slice was the
view-cache observation collapse path. Widget layout/measure/paint observations were recorded under
`NodeId` and then collapsed after layout or paint into the nearest cache-root `NodeId`.

The clean intermediate target is not to move every window-owned product into a boundary. It is to
record view-cache-owned observations under a view/boundary subscriber when the observation is made,
while preserving node-local records for cleanup and for non-cache-root retained widgets.

# Evidence

The pre-change normal path was:

- `crates/fret-ui/src/tree/observation.rs`: observation indexes keyed by `NodeId`.
- `crates/fret-ui/src/tree/layout/entrypoints.rs` and `crates/fret-ui/src/tree/paint/entry.rs`:
  post-phase collapse calls.
- `crates/fret-ui/src/tree/ui_tree_view_cache.rs`: collapse helpers that projected descendant
  observations to cache roots.
- `crates/fret-ui/src/tree/ui_tree_invalidation_walk/propagate.rs`: model/global fanout consumed
  observation owners and walked retained nodes.

Existing ownership guard tests already cover window-owned products: dispatch snapshots, command
routing, final semantics, modal barriers, hit-test path routing, focus/capture, and window
arbitration.

# Recommendation

Delete the collapse helpers and calls. Add an `ObservationSubscriber` vocabulary with node and
boundary variants, choose the boundary subscriber at record time for view-cache-owned observations,
and keep per-node reverse records so node removal and boundary final removal can clean aggregates.

Do not move dispatch snapshots, command routing, final semantics snapshots, hit-test path routing,
focus/capture state, active layer roots, modal barriers, or tree-wide paint recording as part of
this slice.

# Disposition

Accepted. The implementation now records view-cache-owned layout/paint observations under
`ObservationSubscriber::Boundary(BoundaryId)`, keeps per-node records for cleanup, removes the
post-layout/post-paint cache-root observation collapse bridge, and keeps window/layer-forest products
in their existing owners.
