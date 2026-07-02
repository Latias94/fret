---
type: Subagent Finding
title: Phase 2 U4 boundary store audit
tags: fret,ui,view-boundary,viewid,phase2,subagent
timestamp: 2026-07-02
related_plan: docs/plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md
git_branch: feat/ui-framework-phase2-refactor
subagent_id: 019f234c-c656-73f0-ab47-ed944fed1729
---

# Finding

The first U4 cut removes the implicit `ViewId`/`NodeId` conversion but does not yet make boundary
storage entity-first. `DirtyViewFrontier` stores `ViewId`, but its normal mutating paths still accept
cache-root `NodeId`s through v1 bridge methods. `BoundaryId` is still a raw `NodeId` wrapper, and
`UiTree::view_boundaries` is still keyed by `SecondaryMap<NodeId, ViewBoundaryState>`.

# Evidence

- `crates/fret-core/src/ids.rs` now defines `ViewId(u64)` with `from_raw` / `as_raw`.
- `crates/fret-ui/src/tree/view_boundary.rs` still contains explicit cache-root node bridge helpers.
- `crates/fret-ui/src/tree/mod.rs` still owns `view_boundaries:
  slotmap::SecondaryMap<NodeId, ViewBoundaryState>`.
- Layout dirty propagation still has bridge consumers in
  `crates/fret-ui/src/tree/ui_tree_subtree_layout_dirty.rs` and
  `crates/fret-ui/src/tree/layout/entrypoints.rs`.

# Recommendation

Make the next U4 slice an entity store plus live projection:

- Introduce `ViewBoundaryStore` with independent `BoundaryId` allocation, a `ViewId -> BoundaryId`
  map, and a temporary `NodeId -> BoundaryId` live lookup.
- Move `ViewBoundaryState` to carry `view: ViewId` and `live_node: Option<NodeId>`.
- Change `DirtyViewFrontier` normal APIs to `mark_view`, `clear_view`, and `iter_views`; project to
  live nodes only at current layout bridge boundaries.
- Keep observation-index migration separate unless the store cut exposes a simple deletion path.

# Disposition

Implemented as the next U4 slice. `ViewBoundaryStore` now owns independent `BoundaryId` records plus
live-node and `ViewId` lookup indexes, and `DirtyViewFrontier` normal mutation is view-native. The
remaining follow-up is true durable `ViewId` allocation plus runtime detach/rebind semantics.
