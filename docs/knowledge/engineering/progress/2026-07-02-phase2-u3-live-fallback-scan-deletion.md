---
type: Work Progress
title: Phase 2 U3 live fallback scan deletion
tags: fret,ui,identity,phase2,ce-work
timestamp: 2026-07-02
related_plan: docs/plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md
git_branch: feat/ui-framework-phase2-refactor
status: complete
---

# Summary

Phase 2 U3 removed the normal-path scan fallback from live element-to-node resolution and the
semantics relation map's lazy `WindowFrame.instances` construction.
`UiTree::resolve_live_attached_node_for_element*` now resolves through the seeded live node check
and the authoritative `ElementNodeIndex`; retained detached reuse remains a separate named path via
`resolve_reusable_node_for_element_seeded`.

Deleted runtime and diagnostics surface:

- `UiTree::live_nodes_for_element`, including its `WindowFrame.instances` and `self.nodes.iter()`
  live lookup scans.
- The `self.nodes.iter()` fallback inside `resolve_live_attached_node_for_element_seeded`.
- Identity fallback scan frame stats, bootstrap diagnostic fields, and `fret-diag` perf keys.
- `ElementIdMapCache` and `element_id_map_for_window`, which previously built semantics relation
  lookup maps by scanning `WindowFrame.instances`.

Scroll target invalidation now resolves the target through the live index. The same-frame stale
scroll-target regression test was updated to maintain `Node.element` / index state explicitly, so
the test proves stale indexed handles lose to a live attached indexed handle instead of relying on
`WindowFrame` as an identity authority.

Semantics relation resolution now uses `UiTree::live_element_id_map()`, derived from
`ElementNodeIndex`. `SemanticsCx::resolve_declarative_element()` treats a supplied map as
authoritative, so duplicate or missing live indexed elements do not fall back to
`ElementRuntime::live_node_for_element`.

# Verification

Passed:

- `cargo check -p fret-ui`
- `cargo check -p fret-bootstrap`
- Focused U3 identity nextest coverage for seeded stale resolution, retained seed reuse, duplicate
  live ids, detached indexed handles, and scroll target stale/live resolution.
- Focused semantics relation coverage:
  `mechanism_harness_semantics_relations_match_oracles`,
  `semantics_snapshot_reuses_clean_subtrees_between_dirty_refreshes`, and
  `semantics_subtree_reuse_product_is_owned_by_view_boundary_state`.
- `cargo nextest run -p fret-ui --no-fail-fast` (1180 passed)
- `cargo nextest run -p fret-bootstrap --lib --no-fail-fast`
- Focused `fret-diag` perf-key registry contract tests.
- `cargo fmt --all`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `git diff --check`

# Remaining U3 Edge

No normal-path U3 live identity scan bridge remains. Future identity failures should fix index
maintenance or U4 boundary identity ownership, not restore `live_nodes_for_element`, fallback scan
counters, or `element_id_map_for_window`.

# Citations

- [Phase 2 plan](../../../plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md)
- `crates/fret-ui/src/tree/layout/state.rs`
- `crates/fret-ui/src/tree/tests/identity_stress.rs`
- `crates/fret-ui/src/tree/tests/models.rs`
- `crates/fret-ui/src/tree/tests/scroll_invalidation.rs`
- `crates/fret-ui/src/tree/ui_tree_semantics.rs`
- `crates/fret-ui/src/widget.rs`
- `crates/fret-ui/src/declarative/frame.rs`
