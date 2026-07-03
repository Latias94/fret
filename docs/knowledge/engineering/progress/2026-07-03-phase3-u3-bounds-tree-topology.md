---
type: Work Progress
title: Phase 3 U3 bounds-tree topology cleanup
tags: fret,architecture,phase3,retained-bridges,topology,prepaint,hit-test
timestamp: 2026-07-03
related_plan: docs/plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
---

# Phase 3 U3 Bounds-Tree Topology Cleanup

Phase 3 U3 bounds-tree slice removes retained parent ancestry from hit-test bounds-tree clip stack
reconstruction.

Implemented topology changes:

- `InteractionRecord` now carries the frame-time `parent` observed by prepaint traversal.
- `prepaint_interaction_node` passes parent topology through recursion and rewrites the replayed
  cache-root record's parent when a reused cache root is encountered under a current-frame parent.
- `HitTestBoundsTrees::rebuild_for_layer_from_records` no longer receives the retained `SlotMap` and
  no longer reads `Node.parent`; it consumes `InteractionRecord.parent`.

Regression coverage:

- `bounds_tree_clip_stack_uses_recorded_parent_under_stale_parent_pointers` failed under the old
  retained-parent implementation by hitting a child outside a clipping root when all retained child
  parent pointers were stale. It now misses because the clip stack uses recorded prepaint parents.
- Existing bounds-tree, prepaint interaction-cache replay, and virtual-list prepaint tests still
  pass.
- Full `cargo nextest run -p fret-ui --no-fail-fast` passed with 1196 tests.

Verification:

- `cargo nextest run -p fret-ui bounds_tree_clip_stack_uses_recorded_parent_under_stale_parent_pointers --no-fail-fast`
- `cargo nextest run -p fret-ui tree::tests::bounds_tree --no-fail-fast`
- `cargo nextest run -p fret-ui tree::tests::prepaint tree::prepaint::tests prepaint_virtual_list_window_update_harness --no-fail-fast`
- `rg -n "Node\\.parent|\\.parent|nodes|SlotMap|node_parent|and_then\\(\\|n\\| n\\.parent\\)" crates/fret-ui/src/tree/bounds_tree.rs`
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

Next action: finish U3 by classifying remaining retained parent reads. Keep retained storage
mutation, subtree dirty-count aggregation, and repair fallback work separate for U5 unless a path is
still a normal live-topology query.
