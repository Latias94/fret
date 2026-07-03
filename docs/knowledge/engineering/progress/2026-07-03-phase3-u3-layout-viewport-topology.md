---
type: Progress
title: Phase 3 U3 layout and viewport topology migration
tags: fret,architecture,phase3,retained-bridges,topology,layout
timestamp: 2026-07-03
related_plan: docs/plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
---

# Phase 3 U3 Layout and Viewport Topology Migration

Phase 3 U3 second slice removes retained parent authority from layout-time contained relayout and
viewport-root queries.

Implemented topology changes:

- Contained view-cache relayout candidate pruning now ascends through layer-forest child edges.
- Contained view-cache and pending barrier relayout bounds fallback now resolves the parent
  layout-engine child rect through child-edge topology.
- Contained view-cache and pending barrier scroll follow-up scheduling now finds the nearest
  scrollable ancestor through child-edge topology.
- Viewport root bounds lookup and viewport root registration owner lookup now use child-edge
  topology instead of retained `Node.parent`.

Regression coverage:

- `contained_view_cache_relayout_uses_child_edges_for_candidate_pruning_and_scroll_followup`
  verifies stale retained parents cannot change contained candidate pruning or scroll follow-up
  scheduling.
- `element_root_bounds_cache_uses_child_edges_under_stale_parent_pointers` verifies nearest
  viewport root bounds follow the actual child-edge parent chain.
- Full `fret-ui` nextest passed after the migration.

Next action: continue U3 by auditing the remaining non-debug normal-path retained parent queries
in semantics, invalidation walks, hit testing, shortcuts, and command/focus fallback paths. Keep
mutation code and explicit retained storage maintenance out of scope until U5.
