---
type: Work Progress
title: Phase 2 U5 boundary layout candidates
tags: fret,ui,view-boundary,layout-dirty,phase2,ce-work
timestamp: 2026-07-02
related_plan: docs/plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md
git_branch: feat/ui-framework-phase2-refactor
status: complete
---

# Summary

Phase 2 U5 deletes the remaining `dirty_live_boundary_nodes_v1_quarantine` layout projection bridge.
Layout dirty consumers now call `UiTree::dirty_boundary_layout_candidates`, which walks the
`DirtyViewFrontier` by `ViewId`, resolves through `ViewBoundaryStore`, and returns
`DirtyBoundaryLayoutCandidate` values that carry the owning `BoundaryId` plus the current live
layout root.

The normal layout paths no longer expose a dirty-view-to-`NodeId` bridge API:

- `prune_detached_layout_followups` consumes candidate roots instead of dirty live boundary nodes.
- `layout_contained_view_cache_roots_if_needed` schedules contained relayouts from boundary
  candidates and keeps a debug assertion that the candidate root still matches the boundary record.
- `node_subtree_layout_dirty_covered_by_contained_view_cache_roots` uses the same candidate API.

No new ownership was moved into a boundary. Dispatch snapshots, command availability/routing, final
semantics snapshots, hit-test path routing, focus/capture state, active layer roots, modal barriers,
and paint ordering remain window/layer-forest owned per ADR 0327.

# Verification

Passed:

- `cargo check -p fret-ui --tests`
- `cargo check -p fret-ui --features diagnostics`
- `cargo check -p fret-bootstrap --lib --features launch,ui-app-driver,diagnostics`
- `cargo nextest run -p fret-ui --no-fail-fast` (1182 passed)
- `cargo nextest run -p fret-bootstrap --lib --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- Static search in `crates/fret-ui/src`, `crates/fret-core/src`, and contract docs for
  `dirty_live_boundary_nodes_v1_quarantine`, `iter_boundary_nodes_v1`,
  `mark_boundary_node_v1`, `clear_boundary_node_v1`,
  `view_id_for_live_boundary_node_v1_quarantine`, and
  `live_boundary_node_for_view_id_v1_quarantine`.
- `git diff --check`

# Remaining Edge

Superseded by the follow-on observation subscriber slice. Observation invalidation for
view-cache-owned layout/paint paths now records boundary subscribers directly instead of using the
v1 cache-root collapse pass.

# Citations

- [Phase 2 plan](../../../plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md)
- [U5 boundary bridge audit](../subagents/2026-07-02-phase2-u5-boundary-bridge-audit.md)
- [U5 observation boundary subscribers](2026-07-02-phase2-u5-observation-boundary-subscribers.md)
- `crates/fret-ui/src/tree/view_boundary.rs`
- `crates/fret-ui/src/tree/layout/entrypoints.rs`
- `crates/fret-ui/src/tree/ui_tree_subtree_layout_dirty.rs`
- `docs/adr/0165-dirty-views-and-notify-gpui-aligned.md`
- `docs/adr/0327-frame-pipeline-v2-and-view-boundaries.md`
