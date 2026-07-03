---
type: Progress
title: Phase 3 U3 dispatch fallback topology cleanup
tags: fret,architecture,phase3,retained-bridges,topology,dispatch
timestamp: 2026-07-03
related_plan: docs/plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
---

# Phase 3 U3 Dispatch Fallback Topology Cleanup

Phase 3 U3 fourth slice removes retained parent fallback from normal dispatch and scroll helper
paths that already prefer dispatch snapshots or current-frame topology.

Implemented topology changes:

- Dispatch event-chain fallback, hover target fallback, trapped-focus fallback, dispatch observer
  fallback, and dispatch invalidation ordering now use child-edge parents instead of retained
  `Node.parent`.
- `scroll_node_into_view` now walks scroll ancestors through child-edge topology, so stale retained
  parents cannot scroll the wrong ancestor.
- Snapshot-backed paths still prefer `UiDispatchSnapshot.parent`; this slice only changes fallback
  behavior when no valid snapshot parent exists.

Regression coverage:

- `scroll_node_into_view_uses_child_edges_under_stale_parent_pointers` verifies stale retained
  parents do not receive scroll-into-view requests.
- Existing stale-parent dispatch/hover tests continue to pass, including command dispatch bubbling,
  action availability snapshot publication, and hover-region updates.
- Full `fret-ui` nextest passed after the migration.

Next action: continue U3 with the remaining P0/P1 query surfaces from the read-only audit:
shortcuts/key-context ancestry, pointer-down default focus ancestor resolution, hit-test path cache,
semantics snapshot parent assignment, widget coordinate mapping, and observation propagation depth.
