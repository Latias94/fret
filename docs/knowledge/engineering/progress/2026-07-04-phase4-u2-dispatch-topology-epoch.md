---
type: Work Progress
title: Phase 4 U2 dispatch snapshot topology epoch
tags: fret,phase4,topology,dispatch,epoch
timestamp: 2026-07-04T15:42:00Z
related_plan: ../../../plans/2026-07-04-001-refactor-ui-framework-phase4-topology-epoch-plan.md
git_branch: feat/ui-framework-phase2-refactor
---

# Summary

This Phase 4 U2 sub-slice stamped dispatch snapshots and dispatch-derived focus traversal cache keys
with `LiveTopologyEpoch`.

Changes:

- Added `topology_epoch` to `UiDispatchSnapshotCacheKey`.
- Added `topology_epoch` to `UiDispatchSnapshot`.
- Added `topology_epoch` to debug dispatch snapshots and parity reports.
- Added `dispatch_snapshot_topology_epoch` to `WindowFocusTraversalAvailabilityCacheKey`.
- Extended dispatch snapshot cache tests to prove:
  - same-topology same-children writes reuse the cached snapshot,
  - topology-changing child-edge writes produce a newer snapshot epoch,
  - debug frame stats publish the same live topology epoch.

# Design Finding

The dispatch snapshot cache still keeps its explicit generation invalidation. The epoch is an
additional correctness gate, not a replacement yet. This means existing structure invalidation
behavior remains unchanged, while stale-product rejection can now be expressed by comparing the
snapshot's `LiveTopologyEpoch` with the current topology owner.

This is intentionally smaller than full U2. View-boundary frame products and view-cache reuse
decisions still need topology epoch stamping before U3 can delete normal DFS parent fallback paths.

# Verification

Passed:

- `cargo check -p fret-ui`
- `cargo nextest run -p fret-ui dispatch_snapshot_cache_reuses_forest_across_frames_until_structure_changes --no-fail-fast`
- `cargo nextest run -p fret-ui --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Next Action

Continue Phase 4 U2 by stamping view-boundary frame products and view-cache reuse decisions with
`LiveTopologyEpoch`, then add stale-epoch tests that mutate child edges after a boundary/cache
product is built.

# Citations

- [Phase 4 topology epoch plan](../../../plans/2026-07-04-001-refactor-ui-framework-phase4-topology-epoch-plan.md)
- [Phase 4 U1 live topology epoch owner](2026-07-04-phase4-u1-live-topology-epoch.md)
