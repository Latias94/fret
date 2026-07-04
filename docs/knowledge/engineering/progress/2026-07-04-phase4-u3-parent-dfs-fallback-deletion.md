---
type: Work Progress
title: Phase 4 U3 parent DFS fallback deletion
tags: fret,phase4,topology,parent-query,dfs-fallback,deletion
timestamp: 2026-07-04T14:27:21Z
related_plan: ../../../plans/2026-07-04-001-refactor-ui-framework-phase4-topology-epoch-plan.md
git_branch: feat/ui-framework-phase2-refactor
---

# Summary

This Phase 4 U3 sub-slice removed the normal-path DFS scan fallback from
`parent_in_layer_forest_via_children`.

Changes:

- `parent_in_layer_forest_via_children` now returns only the validated parent edge owned by
  `LiveTopologyIndex`.
- The function still verifies that the indexed parent is live and that the parent's child list
  contains the node.
- `validated_child_edge_parent_for_reparent` no longer falls through to retained `Node.parent` for
  live children. Retained parent storage is only used for detached/non-live children.

# Design Finding

The typed topology owner is now the live parent oracle. If the index drifts, tests should fail
instead of silently scanning layer roots and repairing the query. This matches the Phase 4 contract:
child edges remain authoritative, but the derived child-parent index is the current-frame product
that normal hot paths consume.

# Verification

Passed:

- `cargo check -p fret-ui`
- `cargo nextest run -p fret-ui same_children_write_keeps_live_topology_epoch_when_edges_are_unchanged reparent_with_stale_retained_parent_advances_live_topology_epoch set_children_reparents_from_old_parent_without_leaving_stale_child_edges set_children_in_mount_reparents_from_old_parent_without_leaving_stale_child_edges set_children_barrier_reparents_from_old_barrier_without_leaving_stale_child_edges view_cache_nearest_root_uses_child_edges_under_stale_parent_pointers view_cache_snapshot_invalidation_ignores_stale_snapshot_topology_epoch --no-fail-fast`
- `cargo nextest run -p fret-ui --no-fail-fast --status-level fail`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Next Action

Continue U3 with a naming/API cleanup pass: rename `parent_in_layer_forest_via_children` to an
index-oriented name, or add a debug-only slow checker if future diagnostics need an assertion
oracle. Do not restore a normal-path DFS scan.

# Citations

- [Phase 4 topology epoch plan](../../../plans/2026-07-04-001-refactor-ui-framework-phase4-topology-epoch-plan.md)
- [Phase 4 U2 stale snapshot guard](2026-07-04-phase4-u2-stale-snapshot-view-cache-guard.md)
