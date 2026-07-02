---
type: Work Progress
title: Phase 2 U5 observation boundary subscribers
tags: fret,ui,observation,view-boundary,phase2,ce-work
timestamp: 2026-07-02
related_plan: docs/plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md
git_branch: feat/ui-framework-phase2-refactor
status: complete
---

# Summary

Phase 2 U5 deletes the post-layout/post-paint cache-root observation collapse bridge. Layout,
measure, and paint now choose an `ObservationSubscriber` while recording observations:

- view-cache-owned observations aggregate under `ObservationSubscriber::Boundary(BoundaryId)`,
- non-cache-root observations remain `ObservationSubscriber::Node(NodeId)`,
- per-node records remain the reverse index for ordinary node cleanup and boundary final-removal
  cleanup.

The aggregate `by_model` / `by_global` indexes now fan out by subscriber rather than raw `NodeId`.
Propagation resolves a boundary subscriber to its current live node through `ViewBoundaryStore`
before walking invalidation. Detached boundaries therefore do not produce a stale layout root, and
final boundary removal clears subscriber aggregates.

The slice also removes current runtime/exported frame stats and trace-exported perf keys for the
deleted collapse spans. `fret-diag` still keeps registered stats compatibility for historical
bundles that contain the old fields.

# Verification

Passed:

- `cargo check -p fret-ui --tests`
- `cargo check -p fret-ui --features diagnostics`
- `cargo check -p fret-bootstrap --lib --features launch,ui-app-driver,diagnostics`
- `cargo check -p fret-diag`
- `cargo nextest run -p fret-ui tree::tests::observation --no-fail-fast`
- `cargo nextest run -p fret-ui observation model_change global_change view_cache_observation stale_detached_node_entry --no-fail-fast`
- `cargo nextest run -p fret-ui dispatch_snapshot_cache command_availability semantics_snapshot modal_barrier --no-fail-fast`
- `cargo nextest run -p fret-ui --no-fail-fast` (1184 passed)
- `cargo nextest run -p fret-bootstrap --lib --no-fail-fast` (7 passed)
- `cargo nextest run -p fret-diag trace_exported_perf_keys_are_unique trace_exported_perf_key_units_match_names trace_exported_perf_key_registry_contains_core_timeline_keys registered_perf_keys_are_unique registered_perf_key_units_match_names registered_perf_key_contract_keeps_stats_and_gate_keys_additive registered_perf_key_inventory_doc_is_in_sync full_registered_perf_key_registry_covers_consumed_debug_stats_fields --no-fail-fast`
- `cargo fmt --all`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 /Users/frankorz/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`
- Static source search confirms the deleted collapse helper and trace span names no longer exist in
  `crates/fret-ui/src` or current bootstrap frame-stat exports.

# Remaining Edge

This slice does not create a public `cx.notify(view_id)` API and does not migrate all
ElementRuntime/declarative observation storage to a public ViewId subscriber surface. Those are the
next runtime/API design steps. The deleted bridge must not be restored as a post-phase collapse
pass.

# Citations

- [Phase 2 plan](../../../plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md)
- [Observation fanout audit](../subagents/2026-07-02-phase2-u5-observation-fanout-audit.md)
- `crates/fret-ui/src/tree/observation.rs`
- `crates/fret-ui/src/tree/ui_tree_invalidation_walk/propagate.rs`
- `crates/fret-ui/src/tree/ui_tree_mutation/remove.rs`
- `crates/fret-ui/src/tree/ui_tree_view_cache.rs`
- `docs/adr/0165-dirty-views-and-notify-gpui-aligned.md`
- `docs/adr/0327-frame-pipeline-v2-and-view-boundaries.md`
