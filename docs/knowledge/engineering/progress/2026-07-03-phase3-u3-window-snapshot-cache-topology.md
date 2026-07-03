---
type: Work Progress
title: Phase 3 U3 window snapshot and cache topology slice
tags: fret,phase3,u3,retained-parent,view-cache,topology
timestamp: 2026-07-03
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
subagent_id: 019f2769-0c37-7052-a68e-fbefd92a6e99
---

# Summary

This U3 slice moved another group of normal live-topology queries away from retained
`Node.parent` and toward layer-forest child-edge topology:

- Pending declarative window snapshot attachment now uses layer-tree attachment instead of
  `node_layer || node_parent`.
- Selectable text drag auto-scroll ancestor lookup uses child-edge ancestors.
- View-cache invalidation walks and nested cache-root ancestor propagation use child-edge parents.
- Layout dirty suppression and descendant checks use child-edge parents.
- Snapshot fallback cache root walks use child-edge parents.
- Element root bounds cache distinguishes current-frame viewport roots from retained viewport-root
  bounds and avoids reusing stale retained bounds when the owner was laid out in the current frame.

The slice also added stale-parent tests that intentionally corrupt retained parent metadata while
the declarative child graph remains valid.

# Subagent Audit

Read-only explorer `019f2769-0c37-7052-a68e-fbefd92a6e99` confirmed the direction is correct but
U3 is not complete. The remaining high-risk retained bridges are:

- Normal mount still calls `parent_pointers_would_repair_from_layer_roots` and
  `repair_parent_pointers_from_layer_roots`.
- `ui_tree_subtree_layout_dirty.rs` still has retained-parent reads for dirty aggregation,
  underflow recovery, and repair paths.
- `refresh_view_cache_membership_for_ancestor_roots` in declarative mount still uses
  `ui.node_parent(node)` and belongs to U4.
- `parent_in_layer_forest_via_children` is a transitional helper; the final U3 target is a frozen
  frame or boundary topology product with an explicit epoch lifecycle.

# Verification

Passed:

- `cargo check -p fret-ui`
- Focused U3 tests:
  `pending_declarative_snapshot_commit_uses_child_edges_under_stale_parent_pointers`,
  `render_layer_interaction_root_parent_attach_commits_window_snapshot_after_root_attachment`,
  `view_cache_invalidation_walk_uses_child_edges_under_stale_parent_pointers`,
  `view_cache_nested_boundary_ancestors_use_child_edges_under_stale_parent_pointers`,
  `contained_view_cache_dirty_coverage_uses_child_edges_under_stale_parent_pointers`,
  `layout_dirty_suppression_uses_child_edges_under_stale_parent_pointers`,
  `selectable_text_drag_autoscrolls_scroll_container`,
  `element_root_bounds_cache_rebuilds_on_view_cache_hit_after_viewport_move`
- Focused viewport-root bounds tests:
  `element_root_bounds_cache_rebuilds_on_view_cache_hit_after_viewport_move`,
  `element_root_bounds_cache_uses_child_edges_under_stale_parent_pointers`,
  `element_root_bounds_cache_rebuilds_when_element_moves_between_viewport_roots`,
  `element_root_bounds_cache_prunes_when_owner_relayouts_without_viewport_root`
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

Not fully proven locally:

- Full `cargo nextest run -p fret-ui --no-fail-fast` was interrupted after the non-stack suite had
  passed because the two `stack_safety` deep-tree tests were still running for several minutes.
  Treat this as a local long-running gate caveat, not as a code failure.

# Next Action

Continue U3 by classifying and migrating the remaining retained-parent reads in
`ui_tree_subtree_layout_dirty.rs`, `declarative/mount.rs`, layout callbacks, and removal cleanup.
Then proceed to U4 by replacing cache-hit membership refresh scans with boundary-owned recorded
membership.
