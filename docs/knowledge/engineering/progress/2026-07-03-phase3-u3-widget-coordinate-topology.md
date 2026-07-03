---
type: Work Progress
title: Phase 3 U3 widget coordinate topology cleanup
tags: fret,architecture,phase3,retained-bridges,topology,transforms,input
timestamp: 2026-07-03
related_plan: docs/plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
---

# Phase 3 U3 Widget Coordinate Topology Cleanup

Phase 3 U3 widget-coordinate slice removes retained parent ancestry from window-to-node coordinate
mapping.

Implemented topology changes:

- `UiTree::map_window_point_and_vector_to_node_layout_space` now builds the target-to-root transform
  chain through layer-forest child-edge parents.
- Event coordinate helpers that use this mapping no longer depend on retained `Node.parent` being
  correct.

Regression coverage:

- `map_window_point_to_node_layout_space_uses_child_edges_under_stale_parent_pointers` failed on the
  retained-parent implementation by applying the stale sibling's child transform. It now applies the
  actual child-edge parent's transform.
- Existing transform tests for pointer event coordinates, wheel vectors, visual bounds, nested
  transforms, and declarative render/visual transforms still pass.
- Full `cargo nextest run -p fret-ui --no-fail-fast` passed with 1194 tests.

Verification:

- `cargo nextest run -p fret-ui map_window_point_to_node_layout_space_uses_child_edges_under_stale_parent_pointers --no-fail-fast`
- `cargo nextest run -p fret-ui tree::tests::transforms declarative::tests::core::render_transform_affects_hit_testing declarative::tests::core::visual_transform_does_not_affect_hit_testing --no-fail-fast`
- `rg -n "nodes\\.get\\(id\\).*parent|\\.and_then\\(\\|n\\| n\\.parent\\)|node_parent\\(" crates/fret-ui/src/tree/ui_tree_widget.rs`
- `cargo check -p fret-ui`
- `cargo fmt --all --check`
- `git diff --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `cargo nextest run -p fret-ui --no-fail-fast`

Next action: continue U3 with propagation depth ordering, bounds-tree prepaint parent
reconstruction, and remaining invalidation or debug-only retained parent classifications.
