---
type: Work Progress
title: Phase 2 U4 durable ViewId lifecycle
tags: fret,ui,view-boundary,viewid,phase2,ce-work
timestamp: 2026-07-02
related_plan: docs/plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md
git_branch: feat/ui-framework-phase2-refactor
status: complete
---

# Summary

Phase 2 U4 now allocates `ViewId`s from `ViewBoundaryStore` instead of deriving them from live
`NodeId` slotmap raw values. `ViewBoundaryStore` uses stable `ViewBoundaryKey` values:

- `ViewBoundaryKey::Element(GlobalElementId)` for declarative boundary roots.
- `ViewBoundaryKey::RuntimeNode(NodeId)` for no-element low-level runtime boundaries.

The live-node-derived ViewId helper and test-only reverse helper were deleted. Tests now compare
dirty/debug views through the boundary store's current `ViewId` lookup rather than assuming a
`ViewId` can be converted back to a `NodeId`.

The store also distinguishes temporary detach from final removal:

- `detach_live_node` preserves the boundary record and clears `live_node`.
- `remove_node` / `remove_live_node` delete the boundary record on final removal.
- structural child detach, barrier detach, mount detach, and layer-root replacement now sync
  boundary liveness for affected subtrees.

# Verification

Passed:

- `cargo check -p fret-ui`
- `cargo check -p fret-ui --tests`
- `cargo check -p fret-ui --features diagnostics`
- `cargo check -p fret-bootstrap --lib --features launch,ui-app-driver,diagnostics`
- Static search in `crates/fret-ui/src` and `crates/fret-core/src` for deleted bridge names:
  `view_id_for_live_boundary_node_v1_quarantine`,
  `live_boundary_node_for_view_id_v1_quarantine`,
  `clear_live_boundary_node_v1_quarantine`,
  `contains_live_boundary_node_v1_quarantine`,
  `ViewId(pub NodeId)`, `BoundaryId(NodeId)`, and raw boundary storage names.
- `cargo nextest run -p fret-ui dirty_view_frontier_coalesces_views_without_node_bridge view_boundary_store_rebinds_element_view_without_treating_node_as_boundary_identity view_boundary_store_rebinding_same_node_to_new_element_allocates_new_view detached_dirty_view_cache_root_is_pruned_before_layout_followups view_cache_mark_nearest_root_needs_rerender_propagates_to_ancestor_roots boundary_frame_products_own_boundary_dirty_prepaint_interaction_scene_and_paint_cache_state set_children_same_children_repairs_parent_pointers_and_reconnects_dirty_descendant_layout set_children_in_mount_new_dirty_detached_parent_skips_redundant_structural_walk set_children_reparents_from_old_barrier_using_barrier_detach_semantics set_root_replacement_clears_detached_base_layer_interaction_state set_root_replacement_preserves_overlay_interaction_state mechanism_harness_layout_dirty_invalidation_matches_oracles --no-fail-fast`
- `cargo nextest run -p fret-ui --no-fail-fast` (1182 passed)
- `cargo nextest run -p fret-bootstrap --lib --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 /Users/frankorz/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`

# Remaining Edge

The remaining U4/U5 bridge is `dirty_live_boundary_nodes_v1_quarantine`, which projects dirty
`ViewId`s to live `NodeId`s because the current layout engine entrypoints still consume node roots.
Deleting it should be a U5 slice that changes layout dirty iteration to consume boundary records or
explicit boundary IDs directly.

Observation fanout is still `NodeId`-based and should remain a separate U4/U5 slice; do not mix it
with renderer/window-owned product migration.

# Citations

- [Phase 2 plan](../../../plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md)
- [Durable ViewId audit](../subagents/2026-07-02-phase2-u4-durable-viewid-audit.md)
- `crates/fret-core/src/ids.rs`
- `crates/fret-ui/src/tree/view_boundary.rs`
- `crates/fret-ui/src/tree/ui_tree_mutation/{core.rs,mount.rs,barrier.rs,remove.rs}`
- `crates/fret-ui/src/tree/layers/impls.rs`
- `crates/fret-ui/src/tree/tests/view_cache.rs`
- `crates/fret-ui/src/tree/tests/layout_dirty_invalidation_harness.rs`
