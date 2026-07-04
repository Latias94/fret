---
type: Work Progress
title: Phase 4 U2 view boundary topology epoch
tags: fret,phase4,topology,view-boundary,view-cache,epoch
timestamp: 2026-07-04T13:43:15Z
related_plan: ../../../plans/2026-07-04-001-refactor-ui-framework-phase4-topology-epoch-plan.md
git_branch: feat/ui-framework-phase2-refactor
---

# Summary

This Phase 4 U2 sub-slice stamped view-boundary frame products with the
`LiveTopologyEpoch` they consumed.

Changes:

- Added topology epoch provenance to `BoundaryFrameProducts`.
- Threaded `LiveTopologyEpoch` through `ViewBoundaryStore::ensure_live`,
  `ensure_boundary_for_key`, `ViewBoundaryState::new_runtime`, and
  `ViewBoundaryState::refresh_runtime`.
- Exposed the stamped epoch in `UiDebugBoundaryStats`.
- Added a stale retained-parent reparent test proving boundary parent ancestry and boundary frame
  product epoch update after a topology-changing child-edge mutation.

# Design Finding

This is a provenance and observability step, not a broad global cache invalidation rule. A boundary
state now records the current live topology epoch when `UiTree` ensures the live boundary, and debug
stats can report that epoch. The change intentionally does not reject every cross-epoch view-cache
product yet, because a global epoch mismatch can be too coarse: unrelated topology mutations should
not necessarily invalidate every boundary's paint/prepaint products.

The next U2 slice should decide the narrower reuse gate: boundary ancestry and topology-dependent
products must reject stale topology, while topology-independent cached products should keep their
existing invalidation contracts.

# Verification

Passed:

- `cargo check -p fret-ui`
- `cargo nextest run -p fret-ui --no-fail-fast --status-level fail`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Next Action

Continue Phase 4 U2 by adding the narrow view-cache reuse stale-epoch gate, then move to U3 only
after dispatch and boundary tests prove stale topology products are rejected without the DFS parent
fallback.

# Citations

- [Phase 4 topology epoch plan](../../../plans/2026-07-04-001-refactor-ui-framework-phase4-topology-epoch-plan.md)
- [Phase 4 U1 live topology epoch owner](2026-07-04-phase4-u1-live-topology-epoch.md)
- [Phase 4 U2 dispatch snapshot topology epoch](2026-07-04-phase4-u2-dispatch-topology-epoch.md)
