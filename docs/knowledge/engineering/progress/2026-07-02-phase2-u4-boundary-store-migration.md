---
type: Work Progress
title: Phase 2 U4 boundary store migration
tags: fret,ui,view-boundary,boundaryid,viewid,phase2,ce-work
timestamp: 2026-07-02
related_plan: docs/plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md
git_branch: feat/ui-framework-phase2-refactor
status: partial
---

# Summary

Phase 2 U4 now stores boundary records through `ViewBoundaryStore` instead of
`SecondaryMap<NodeId, ViewBoundaryState>`. `BoundaryId` is an independent `SlotMap` key, and
`ViewBoundaryState` carries `view: ViewId`, `live_node: Option<NodeId>`, and parent
`BoundaryId` metadata. Live-node access is now a projection through the store, not the owner key.

`DirtyViewFrontier` also has view-native normal APIs (`mark_view`, `clear_view`, `iter_views`).
Layout candidates still need live `NodeId`s in v1, but that projection is centralized behind
`UiTree::dirty_live_boundary_nodes_v1_quarantine()` and `ViewBoundaryStore::live_node_for_view`.

# Verification

Passed so far:

- `cargo check -p fret-ui`
- `cargo check -p fret-ui --tests`
- `cargo check -p fret-bootstrap --lib --features launch,ui-app-driver,diagnostics`
- `cargo nextest run -p fret-ui view_boundary_store_rebinds_element_view_without_treating_node_as_boundary_identity dirty_view_frontier_coalesces_views_without_node_bridge boundary_frame_products_own_boundary_dirty_prepaint_interaction_scene_and_paint_cache_state view_cache_runs_contained_relayout_for_invalidated_boundaries detached_dirty_view_cache_root_is_pruned_before_layout_followups view_cache_mark_nearest_root_needs_rerender_propagates_to_ancestor_roots view_cache_observation_collapse_skips_already_rooted_observations view_cache_observation_collapse_uplifts_observations_to_nearest_root_and_invalidates_ancestor_roots prepaint_output_is_owned_by_view_boundary_state_and_removed_with_node paint_cache_entry_is_boundary_owned_for_view_cache_roots semantics_subtree_reuse_product_is_owned_by_view_boundary_state canvas_scene_fragment_is_boundary_owned_and_keyed_by_prepaint_key --no-fail-fast`
- `cargo nextest run -p fret-ui --no-fail-fast` (1181 passed)
- `cargo nextest run -p fret-bootstrap --lib --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 /Users/frankorz/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`

# Remaining U4 Edge

Superseded follow-up: the next U4 durable ViewId slice now allocates `ViewId`s from stable
`ViewBoundaryKey` values, deletes the live-`NodeId` derived ViewId helper, and distinguishes
temporary detach (`live_node = None`) from final boundary removal. The remaining bridge is the
layout-only `dirty_live_boundary_nodes_v1_quarantine` projection.

# Citations

- [Phase 2 plan](../../../plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md)
- [U4 boundary-store audit](../subagents/2026-07-02-phase2-u4-boundary-store-audit.md)
- `crates/fret-ui/src/tree/view_boundary.rs`
- `crates/fret-ui/src/tree/mod.rs`
- `crates/fret-ui/src/tree/layout/entrypoints.rs`
- `crates/fret-ui/src/tree/ui_tree_subtree_layout_dirty.rs`
- `crates/fret-ui/src/tree/ui_tree_semantics.rs`
- `crates/fret-ui/src/tree/bounds_tree.rs`
