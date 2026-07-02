---
type: Subagent Finding
title: Phase 2 U5 boundary bridge audit
tags: fret,ui,view-boundary,layout-dirty,phase2,subagent
timestamp: 2026-07-02
related_plan: docs/plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md
git_branch: feat/ui-framework-phase2-refactor
subagent_id: 019f23bf-a8ae-7c20-8c7b-ba3428e560a1
status: complete
---

# Finding

The remaining dirty-boundary-node projection bridge had three normal-path consumers at the start of
the U5 slice:

- `crates/fret-ui/src/tree/layout/entrypoints.rs::prune_detached_layout_followups`
- `crates/fret-ui/src/tree/layout/entrypoints.rs::layout_contained_view_cache_roots_if_needed`
- `crates/fret-ui/src/tree/ui_tree_subtree_layout_dirty.rs::node_subtree_layout_dirty_covered_by_contained_view_cache_roots`

The replacement should keep layout iteration boundary-first and only resolve current live roots
through `ViewBoundaryStore`. A candidate object is cleaner than returning naked `BoundaryId`s because
callers should not repeat the projection logic or accidentally recreate a dirty `NodeId` ownership
API.

# Evidence

Existing tests already guard the U5 ownership boundary:

- Dirty layout candidate behavior: `tree::tests::view_cache::*` and
  `tree::tests::layout_dirty_invalidation_harness::mechanism_harness_layout_dirty_invalidation_matches_oracles`.
- Dispatch snapshot ownership:
  `tree::tests::dispatch_snapshot_cache::dispatch_snapshot_cache_reuses_forest_across_frames_until_structure_changes`.
- Command availability and routing:
  `tree::tests::command_availability::command_availability_uses_dispatch_snapshot_parent_not_retained_parent`,
  `tree::tests::command_dispatch_source_trace::dispatch_command_bubble_uses_dispatch_snapshot_parent_not_retained_parent`,
  and
  `tree::tests::window_command_action_availability_snapshot::action_availability_snapshot_uses_dispatch_snapshot_parent_not_retained_parent`.
- Final semantics and focus barriers:
  `tree::tests::semantics_focus_shortcuts::semantics_snapshot_includes_visible_roots_and_barrier`
  and
  `tree::tests::semantics_focus_shortcuts::semantics_snapshot_exposes_focus_barrier_root_independently_of_pointer_barrier`.
- Hit-test and window arbitration:
  `tree::tests::hit_test::*` modal barrier/path-cache tests and
  `tree::tests::window_input_arbitration_snapshot::*` barrier publication tests.

# Recommendation

Do not add a duplicate Rust behavior test for this narrow bridge deletion. Use existing
dirty-layout/window-ownership tests plus static source search proving the old bridge names no longer
exist in normal paths.

# Disposition

Accepted. The implementation deletes `dirty_live_boundary_nodes_v1_quarantine` and replaces it with
`DirtyBoundaryLayoutCandidate` values resolved through `ViewBoundaryStore`. Contract docs now record
that layout dirty iteration consumes boundary candidates and window-owned products remain
window/layer-forest owned.
