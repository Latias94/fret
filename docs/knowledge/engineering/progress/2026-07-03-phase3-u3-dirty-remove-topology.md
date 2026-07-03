---
type: Work Progress
title: Phase 3 U3 dirty/remove topology slice
tags: fret,phase3,u3,u4,retained-parent,dirty-graph,view-cache
timestamp: 2026-07-03
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
---

# Summary

This slice removed normal retained-parent dependency from subtree dirty propagation, semantics
dirty propagation, subtree removal, layout debug/profile ancestry, and view-cache membership
ancestor walks.

Changes:

- `subtree_semantics_dirty_count` propagation now walks child-edge topology.
- `subtree_layout_dirty_count` recompute, rebuild, underflow repair, and ancestor delta propagation
  now walk child-edge topology.
- Layout dirty underflow no longer calls `repair_parent_pointers_from_layer_roots`; it rebuilds
  counts through child-edge ancestors instead.
- `remove_subtree_inner` unlinks removed nodes from the actual child-edge parent and propagates
  dirty deltas through that parent.
- Layout profile/test-id attribution and layout debug descendant checks use child-edge topology.
- `refresh_view_cache_membership_for_ancestor_roots` walks child-edge ancestors instead of
  `ui.node_parent`.

# Verification

Passed:

- `cargo check -p fret-ui`
- `cargo nextest run -p fret-ui subtree_layout_dirty_underflow_uses_child_edges_without_parent_repair semantics_dirty_propagation_uses_child_edges_under_stale_parent_pointers subtree_layout_dirty_underflow_repairs_counts_upwards --no-fail-fast`
- `cargo nextest run -p fret-ui remove_subtree_uses_child_edges_under_stale_parent_pointers subtree_layout_dirty_underflow_repair --no-fail-fast`
- `cargo nextest run -p fret-ui subtree_layout_dirty_underflow_repair barrier_subtree_layout_dirty_aggregation view_cache_invalidation_walk_uses_child_edges_under_stale_parent_pointers view_cache_nested_boundary_ancestors_use_child_edges_under_stale_parent_pointers contained_view_cache_dirty_coverage_uses_child_edges_under_stale_parent_pointers pending_declarative_snapshot_commit_uses_child_edges_under_stale_parent_pointers selectable_text_drag_autoscrolls_scroll_container --no-fail-fast`
- `cargo nextest run -p fret-ui -E 'not test(stack_safety)' --no-fail-fast`
- `cargo fmt --all --check`
- `git diff --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_adr_numbers.py`
- `python3 tools/check_workstream_catalog.py`
- `python3 ~/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`

# Remaining Bridges

Static search over the immediate U3 files now leaves:

- `declarative/mount.rs` normal parent repair calls at the full-window and interaction-root mount
  paths. These remain the U5 deletion target.
- `ui_tree_mutation/remove.rs` retained-parent reads under diagnostics record construction.
- `declarative/mount.rs` diagnostics/test setup matches.

The next architectural step is U4: replace retained subtree membership collection on cache hits
with boundary-owned recorded membership so mount repair can be removed rather than preserved as a
hidden compatibility shim.
