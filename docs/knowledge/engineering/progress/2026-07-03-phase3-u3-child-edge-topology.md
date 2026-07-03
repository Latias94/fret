---
type: Progress
title: Phase 3 U3 child-edge topology for boundary and cache roots
tags: fret,architecture,phase3,retained-bridges,topology
timestamp: 2026-07-03
related_plan: docs/plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
---

# Phase 3 U3 Child-Edge Topology for Boundary and Cache Roots

Phase 3 U3 first slice moves high-risk boundary and view-cache ancestor queries away from retained
`Node.parent` pointers and onto the layer forest child-edge topology.

Implemented topology changes:

- `UiTree::parent_in_layer_forest_via_children` resolves a node parent by walking current layer
  roots and `children` edges instead of trusting retained parent pointers.
- `nearest_view_cache_root` first requires layer-forest reachability and then ascends via
  child-edge topology.
- `mark_nearest_view_cache_root_needs_rerender` propagates ancestor cache-root dirty state via
  child-edge topology, so stale retained parents no longer receive rerender pressure.
- `nearest_parent_view_boundary_node` now derives the parent boundary from child edges, keeping
  boundary parent products tied to the current layer forest.

Regression coverage:

- `view_cache_nearest_root_uses_child_edges_under_stale_parent_pointers` verifies nearest and
  ancestor cache-root dirty propagation ignore a corrupted retained parent.
- `view_boundary_parent_uses_child_edges_under_stale_parent_pointers` verifies boundary parent
  state follows the actual child-edge parent after layout.
- Full `fret-ui` nextest passed after the migration.

Next action: continue U3 by migrating layout and viewport topology consumers, especially contained
view-cache candidate pruning, bounds fallback, scroll ancestor follow-up, and viewport owner/bounds
queries. U5 should still wait until U2 pressure counters are zero across representative suites.
