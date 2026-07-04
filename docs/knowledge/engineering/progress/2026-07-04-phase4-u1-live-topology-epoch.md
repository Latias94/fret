---
type: Work Progress
title: Phase 4 U1 live topology epoch owner
tags: fret,phase4,topology,epoch,view-cache,dispatch
timestamp: 2026-07-04T15:06:00Z
related_plan: ../../../plans/2026-07-04-001-refactor-ui-framework-phase4-topology-epoch-plan.md
git_branch: feat/ui-framework-phase2-refactor
---

# Summary

Phase 4 U1 introduced a typed owner for the live topology indexes that Phase 3 left as raw
`UiTree` fields.

Changes:

- Added `LiveTopologyEpoch` and `LiveTopologyIndex`.
- Moved live layer-forest membership and child-parent edge storage out of `UiTree` raw fields.
- Routed live subtree indexing, detach cleanup, layer-root rebuild, child-edge replacement, parent
  lookup, and subtree removal through `LiveTopologyIndex`.
- Kept child edges as the topology authority and retained `Node.parent` as storage/debug metadata.
- Added `live_topology_epoch` to debug frame stats and the diagnostics perf-key registry.
- Added epoch contract tests for same-children no-op writes, stale-parent reparent, deep subtree
  removal, and base-root rebuild.

# Design Finding

The correct next step after Phase 3 is not more parent-pointer repair. Fret needs a typed topology
product that can be consumed by dispatch snapshots, view-boundary products, and view-cache decisions.

This slice deliberately keeps the DFS parent fallback for now. The owner and epoch make the fallback
observable and give U2 a version to stamp on consumers. U3 should delete the normal fallback path
only after dispatch and boundary products reject stale topology epochs.

Epoch semantics from this slice:

- Live membership changes advance the epoch.
- Live child-parent edge changes advance the epoch.
- Stable same-children writes do not advance the epoch when indexed edges are already correct.
- Same-children writes that repair an indexed edge drift do advance the epoch because the topology
  owner changed state.

# Subagent Review Disposition

Three read-only subagents informed the Phase 4 ordering:

- Topology/view-cache audit: recommended `LiveTopologyIndex + LiveTopologyEpoch` first, then stamp
  dispatch snapshots, view-boundary frame products, and view-cache reuse decisions.
- Renderer audit: recommended deferring broader `FrameAssembler`/chunk-native stream expansion until
  topology/frame epochs can prove stale product rejection.
- Raw-surface audit: recommended only small raw/prelude documentation hygiene before topology and
  renderer work; remaining raw seams are intentional advanced/manual surfaces.

# Verification

Passed:

- `cargo check -p fret-ui`
- `cargo check -p fret-bootstrap`
- `cargo nextest run -p fret-ui same_children_write_keeps_live_topology_epoch_when_edges_are_unchanged reparent_with_stale_retained_parent_advances_live_topology_epoch removing_deep_subtree_advances_live_topology_epoch_and_clears_live_membership base_root_update_rebuilds_live_topology_epoch_and_live_element_index --no-fail-fast`
- `cargo nextest run -p fret-ui set_children_reparents_from_old_parent_without_leaving_stale_child_edges add_child_reparents_from_old_parent_without_leaving_stale_child_edges set_children_in_mount_reparents_from_old_parent_without_leaving_stale_child_edges set_children_barrier_reparents_from_old_barrier_without_leaving_stale_child_edges --no-fail-fast`
- `cargo nextest run -p fret-ui --no-fail-fast`
- `cargo nextest run -p fret-diag registered_perf_keys_are_unique registered_perf_key_units_match_names registered_perf_key_contract_keeps_stats_and_gate_keys_additive registered_perf_key_inventory_doc_is_in_sync full_registered_perf_key_registry_covers_consumed_debug_stats_fields --no-fail-fast`
- `cargo nextest run -p fret-bootstrap --lib`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Next Action

Continue with Phase 4 U2:

- Stamp cached dispatch snapshots with `LiveTopologyEpoch`.
- Stamp view-boundary frame products or view-cache reuse decisions with `LiveTopologyEpoch`.
- Add stale-epoch tests that mutate child edges after a snapshot/product is built and prove reuse is
  rejected or rebuilt.
- Only after U2 is green, start U3 to demote/delete the DFS parent fallback from normal hot paths.

# Citations

- [Phase 4 topology epoch plan](../../../plans/2026-07-04-001-refactor-ui-framework-phase4-topology-epoch-plan.md)
- [Phase 3 closeout child-edge topology index](2026-07-04-phase3-closeout-child-edge-topology-index.md)
