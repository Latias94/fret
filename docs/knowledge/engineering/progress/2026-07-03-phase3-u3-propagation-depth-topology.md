---
type: Work Progress
title: Phase 3 U3 propagation depth topology cleanup
tags: fret,architecture,phase3,retained-bridges,topology,invalidation
timestamp: 2026-07-03
related_plan: docs/plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
---

# Phase 3 U3 Propagation Depth Topology Cleanup

Phase 3 U3 propagation-depth slice removes retained parent ancestry from observation invalidation
ordering.

Implemented topology changes:

- `propagation_depth_for` now walks `UiTree::parent_in_layer_forest_via_children` instead of
  retained `Node.parent`.
- Model/global/local observation propagation keeps its ancestor-first ordering when retained parent
  pointers are stale.

Regression coverage:

- `propagation_depth_uses_child_edges_under_stale_parent_pointers` failed on the retained-parent
  implementation with depth `0` after the leaf's retained parent was cleared. It now reports depth
  `2` from the authoritative child-edge tree.
- Existing model observation invalidation tests still pass, including the shared-ancestor dedup
  stats test.
- Full `cargo nextest run -p fret-ui --no-fail-fast` passed with 1195 tests.

Verification:

- `cargo nextest run -p fret-ui propagation_depth_uses_child_edges_under_stale_parent_pointers --no-fail-fast`
- `cargo nextest run -p fret-ui model_change_invalidates_observers model_change_invalidates_bound_text_input debug_invalidation_walks_record_model_change_root model_change_invalidation_dedup_stops_at_shared_ancestors --no-fail-fast`
- `cargo check -p fret-ui`
- `cargo fmt --all --check`
- `git diff --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_adr_numbers.py`
- `python3 tools/check_workstream_catalog.py`
- `python3 ~/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `cargo nextest run -p fret-ui --no-fail-fast`

Next action: continue U3 with bounds-tree prepaint parent reconstruction, then classify remaining
normal-path retained parent reads versus U5 dirty-count or debug-only work.
