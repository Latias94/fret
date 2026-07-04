---
type: Work Progress
title: Phase 3 U5.5 retained parent query deletion
tags: fret,phase3,u5.5,retained-parent,execution-surface
timestamp: 2026-07-04
related_plan: ../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
---

# Summary

This slice removes the remaining public-looking retained parent query name from `fret-ui`.

Changes:

- The test-only `UiTree::node_parent` storage accessor was renamed to
  `debug_node_parent_storage` and narrowed to `pub(crate)`.
- Runtime and ecosystem static search now has no `node_parent(` matches; current-frame ancestry
  remains expressed through `node_parent_in_layer_tree` / child-edge topology.
- Tests that intentionally inspect retained parent storage now use the explicit debug oracle name,
  so the test surface no longer looks like a normal topology API.
- `tools/check_execution_surface.py` now rejects any future `node_parent(` reintroduction under
  `crates/fret-ui/src`, `ecosystem`, or `apps`.

# Verification

Passed:

- `cargo nextest run -p fret-ui set_children_in_mount_stale_retained_none_parent_does_not_skip_live_ancestor_walk set_children_same_children_records_parent_drift_without_global_repair_and_reconnects_layout subtree_layout_dirty_underflow_uses_child_edges_without_parent_repair touch_pan_scroll_target_resolution_uses_dispatch_snapshot_parent_when_hit_leaf_parent_is_stale --no-fail-fast`
- `cargo check -p fret-ui`
- `cargo fmt --all --check`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `git diff --check`
- `rg -n "\bnode_parent\(" crates/fret-ui/src ecosystem apps -g '*.rs'` returns no matches.

# Deletion Gate

The old retained parent storage query name is now unavailable to production code and fails the
execution-surface gate if it returns. Remaining retained parent access is limited to direct
storage maintenance, shadow diagnostics, and tests that explicitly name the debug storage oracle.

# Next Action

Continue Phase 3 with either:

- U14 diagnostics cleanup for historical parent-repair compatibility counters once the remaining
  bridge inventory is classified, or
- U13 public facade cleanup for the remaining copyable example surfaces from the U13 audit.

# Citations

- [Phase 3 retained bridge deletion plan](../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md)
- [U5 parent repair deletion](2026-07-03-phase3-u5-parent-repair-deletion.md)
