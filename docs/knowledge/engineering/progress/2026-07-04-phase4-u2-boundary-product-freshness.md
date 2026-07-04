---
type: Work Progress
title: Phase 4 U2 boundary product freshness
tags: fret,phase4,topology,view-boundary,view-cache,cache-freshness
timestamp: 2026-07-04T14:11:17Z
related_plan: ../../../plans/2026-07-04-001-refactor-ui-framework-phase4-topology-epoch-plan.md
git_branch: feat/ui-framework-phase2-refactor
---

# Summary

This Phase 4 U2 sub-slice fixed the boundary product freshness rule that became visible after
stamping boundary frame products with `LiveTopologyEpoch`.

Changes:

- Added a `BoundaryFrameProducts::clear_topology_dependent_products` helper.
- Cleared boundary-owned prepaint outputs, hit-test bounds, semantics subtree, interaction cache,
  scene fragments, and paint cache when a runtime boundary's live node, parent boundary, kind, or
  layout dependency changes.
- Kept the topology epoch as provenance, not a global cache kill switch.
- Added tests proving:
  - reparenting a boundary clears topology-dependent frame products and updates the boundary epoch,
  - unrelated topology epoch bumps do not relabel or clear an unaffected boundary's existing
    products.

# Design Finding

The important boundary is local topology sensitivity, not global topology churn. It is correct to
drop boundary products when the boundary's live binding or parent boundary changes, because cached
interaction records, semantics roots, hit-test bounds, scene fragments, and paint ranges can encode
subtree membership or parent ancestry.

It is not correct to make every boundary product miss when any unrelated topology mutation advances
`LiveTopologyEpoch`. The old product should remain stamped with the epoch it consumed; later reuse
gates can decide whether a product is topology-independent enough to reuse.

# Verification

Passed:

- `cargo check -p fret-ui`
- `cargo nextest run -p fret-ui view_boundary_reparent_clears_topology_dependent_frame_products_and_updates_epoch view_boundary_products_survive_unrelated_topology_epoch_bump --no-fail-fast`
- `cargo nextest run -p fret-ui --no-fail-fast --status-level fail`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Next Action

Continue Phase 4 U2/U3 by guarding snapshot-derived view-cache invalidation walks against stale
dispatch snapshot epochs, then demote `parent_in_layer_forest_via_children` DFS fallback from normal
hot paths once the epoch-bound consumers have coverage.

# Citations

- [Phase 4 topology epoch plan](../../../plans/2026-07-04-001-refactor-ui-framework-phase4-topology-epoch-plan.md)
- [Phase 4 U2 boundary topology epoch](2026-07-04-phase4-u2-boundary-topology-epoch.md)
- Commit before this slice: `bb8757ecff`
