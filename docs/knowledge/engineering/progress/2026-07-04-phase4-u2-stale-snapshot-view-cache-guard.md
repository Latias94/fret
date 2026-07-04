---
type: Work Progress
title: Phase 4 U2 stale dispatch snapshot view-cache guard
tags: fret,phase4,topology,dispatch,view-cache,epoch
timestamp: 2026-07-04T14:20:19Z
related_plan: ../../../plans/2026-07-04-001-refactor-ui-framework-phase4-topology-epoch-plan.md
git_branch: feat/ui-framework-phase2-refactor
---

# Summary

This Phase 4 U2 sub-slice guarded snapshot-derived view-cache invalidation walks against stale
dispatch snapshot topology.

Changes:

- `mark_view_cache_roots_needs_rerender_from_snapshot` now uses a provided dispatch snapshot only
  when its `topology_epoch` matches the current `LiveTopologyEpoch`.
- If the snapshot is stale, the walk falls back to the current child-edge topology.
- Added a test proving a stale snapshot parent chain cannot dirty a detached cache root after the
  leaf has been reparented.

# Design Finding

Dispatch snapshots are valid frame products, not cross-topology ancestry authorities. The correct
fallback is not to fail the invalidation or scan retained `Node.parent`; it is to walk the current
child-edge topology when the snapshot epoch no longer matches.

This keeps the normal snapshot fast path for same-epoch hover invalidation while preventing an old
snapshot from reintroducing stale parent behavior into view-cache rerender marking.

# Verification

Passed:

- `cargo check -p fret-ui`
- `cargo nextest run -p fret-ui view_cache_snapshot_invalidation_ignores_stale_snapshot_topology_epoch --no-fail-fast`
- `cargo nextest run -p fret-ui --no-fail-fast --status-level fail`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Next Action

Move to Phase 4 U3: demote/delete the DFS fallback in
`parent_in_layer_forest_via_children` from normal hot paths. The main remaining deletion risk is
`validated_child_edge_parent_for_reparent`, which still has a retained `Node.parent` fallback that
should be constrained to detached or non-live cases.

# Citations

- [Phase 4 topology epoch plan](../../../plans/2026-07-04-001-refactor-ui-framework-phase4-topology-epoch-plan.md)
- [Phase 4 U2 boundary product freshness](2026-07-04-phase4-u2-boundary-product-freshness.md)
