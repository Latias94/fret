---
type: Work Progress
title: Phase 2 U4 ViewId bridge split
tags: fret,ui,viewid,boundary,phase2,ce-work
timestamp: 2026-07-02
related_plan: docs/plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md
git_branch: feat/ui-framework-phase2-refactor
status: partial
---

# Summary

Phase 2 U4 has started with the smallest breaking identity cut: `ViewId` no longer exposes or
implicitly converts from `NodeId`. The runtime now treats `ViewId` as an independent raw
window-scoped token and keeps the old cache-root mapping behind explicit v1 bridge helpers in
`crates/fret-ui/src/tree/view_boundary.rs`.

This is deliberately not the full U4 completion. The remaining U4 contract still needs an
entity-first `ViewBoundaryStore`: `BoundaryId(NodeId)` and `UiTree::view_boundaries:
SecondaryMap<NodeId, ViewBoundaryState>` are still present and should be deleted in the next slice.

# Verification

Passed:

- `cargo check -p fret-ui`
- `cargo check -p fret-ui --features diagnostics`
- `cargo check -p fret-bootstrap --lib --features launch,ui-app-driver,diagnostics`
- `cargo nextest run -p fret-ui dirty_view_frontier_coalesces_views_and_keeps_v1_node_bridge_explicit view_cache_mark_nearest_root_needs_rerender_propagates_to_ancestor_roots detached_dirty_view_cache_root_is_pruned_before_layout_followups mechanism_harness_layout_dirty_invalidation_matches_oracles --no-fail-fast`
- `cargo nextest run -p fret-ui --no-fail-fast` (1180 passed)
- `cargo nextest run -p fret-bootstrap --lib --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 /Users/frankorz/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`

# Remaining U4 Edge

The next slice should introduce a boundary store keyed by independent boundary/view identity and
make live node lookup a projection, not the storage key. Detached boundaries should remain records
with `live_node = None`, and layout candidates must filter them out.

Do not reintroduce `impl From<NodeId> for ViewId`, `impl From<ViewId> for NodeId`, direct
`dirty.view.0`, or `NodeId::from(view)`. If a `NodeId` projection is still required during
migration, use a helper whose name makes the v1 quarantine explicit.

# Citations

- [Phase 2 plan](../../../plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md)
- [U4 boundary-store audit](../subagents/2026-07-02-phase2-u4-boundary-store-audit.md)
- `crates/fret-core/src/ids.rs`
- `crates/fret-ui/src/tree/view_boundary.rs`
- `crates/fret-ui/src/tree/ui_tree_debug/frame.rs`
- `crates/fret-ui/src/tree/ui_tree_debug/query.rs`
- `docs/runtime-contract-matrix.md`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
