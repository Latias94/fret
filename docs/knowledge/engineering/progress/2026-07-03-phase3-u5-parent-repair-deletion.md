---
type: Work Progress
title: Phase 3 U5 parent repair deletion
tags: fret,phase3,u5,retained-parent,parent-repair,topology
timestamp: 2026-07-03
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
---

# Summary

This slice deleted the normal all-layer retained parent repair pass from mount.

Changes:

- Full-window and interaction-root mount now record only `parent_pointers_would_repair_from_layer_roots`
  as a shadow oracle; they no longer mutate retained parent pointers.
- `repair_parent_pointers_from_layer_roots` and `debug_record_parent_pointer_repair` were deleted.
- Same-children child writes now use `sync_same_children_parent_edges_and_reconnect_layout`, a
  direct-edge storage sync for the touched parent/children, not a global retained-tree repair.
- Children tests were rewritten from "repair is behavior" to "shadow oracle observes drift without
  mutation, and child-edge/direct-edge sync still drives authoritative layout."

# Verification

Passed:

- `cargo check -p fret-ui`
- `cargo nextest run -p fret-ui set_children_same_children_records_parent_drift_without_global_repair_and_reconnects_layout set_children_in_mount_same_children_syncs_parent_edge_without_global_repair_and_reconnects_layout subtree_layout_dirty_underflow_uses_child_edges_without_parent_repair remove_subtree_uses_child_edges_under_stale_parent_pointers --no-fail-fast`
- `cargo nextest run -p fret-ui children subtree_layout_dirty_underflow_repair view_cache gc_liveness retained_virtual_list --no-fail-fast`
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

# Deletion Gate

Static search over `crates`, `ecosystem`, and `apps` now finds no matches for:

- `repair_parent_pointers_from_layer_roots`
- `debug_record_parent_pointer_repair`
- `repair_same_children_parent_pointers`
- `view_cache_transitioned_reuse_roots`
- `record_view_cache_reuse_frame`

Remaining `parent_pointer_repair_*` fields are historical diagnostics/perf compatibility counters;
they remain zero in normal runtime and are owned by U14.

# Remaining Bridges

- `parent_pointers_would_repair_from_layer_roots` remains as the U2/U5 shadow oracle.
- `Node.parent` still exists as retained storage metadata for direct child-edge writes.
- Later closeout should classify all remaining `node_parent`/`Node.parent` matches as storage,
  debug/test, compatibility diagnostics, or true retained bridges before U14.
