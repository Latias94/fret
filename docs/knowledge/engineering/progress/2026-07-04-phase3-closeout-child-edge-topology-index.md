---
type: Work Progress
title: Phase 3 closeout child-edge topology index
tags: fret,phase3,closeout,topology,retained-parent,performance
timestamp: 2026-07-04T11:32:32Z
related_plan: ../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
git_branch: feat/ui-framework-phase2-refactor
---

# Summary

This closeout slice fixes the last `fret-ui` regressions exposed while validating Phase 3.

Changes:

- Added derived live child-edge indexes to `UiTree`:
  - `live_layer_nodes` answers live layer-forest membership without scanning all layer roots.
  - `child_parent_index` accelerates child-edge parent queries without restoring `Node.parent` as
    the live topology authority.
- Updated live subtree indexing, layer-root rebuild, child writes, subtree removal, and dirty
  propagation to maintain and consume those derived indexes.
- Kept all fast paths validated against actual `children` edges before returning a parent.
- Changed reparent old-parent detection to use validated child-edge topology first; retained
  `Node.parent` is accepted only when it points at a real current child edge.
- Added stale retained-parent reparent coverage for standard, mount, and barrier child writes.
- Fixed boundary scene-fragment fingerprints to match `SceneChunkManifestEntry` fingerprints, and
  kept empty scene fragments at fingerprint `0` because empty chunks are absent from manifests.

# Design Finding

Deleting retained parent query authority uncovered a real performance contract:

- Child edges are the topology authority.
- `Node.parent` is retained storage metadata/debug evidence.
- A retained tree can still need derived topology indexes; otherwise ancestor walks and removal over
  deep trees fall back to repeated layer-root scans and degrade toward O(n^2).

The right design is not to resurrect retained parent pointers as the source of truth. It is to keep
derived indexes that are built from child edges, validated against actual child lists, and rebuilt
from layer roots when layer topology changes.

# Subagent Review Disposition

Two read-only subagents reviewed the current diff.

Resolved findings:

- `remove_subtree_inner` initially carried traversal parents as live layer-forest parents. Fixed by
  propagating the carried parent only when the current node already has a live parent.
- `index_live_subtree` initially inserted missing nodes into `live_layer_nodes` before checking
  storage. Fixed by inserting only after `nodes.get(node)` succeeds.
- `remove_subtree_inner` cloned child lists only to clean `child_parent_index`. Fixed with a direct
  field borrow.
- `detach_reparented_children_from_old_parents` still used stale retained `Node.parent` as the sole
  old-parent source. Fixed with validated child-edge parent lookup and stale-parent tests.

No remaining blocking findings were reported after the fixes.

# Bridge Ledger

Current source/tool normal-path searches:

- `repair_parent_pointers_from_layer_roots`: removed from current `crates`, `ecosystem`, `apps`,
  and `tools` source.
- `flat_with_diagnostic_chunks`: removed from current normal launch/source searches.
- `text_resource_snapshot_for_blobs`: removed from current normal renderer/text source searches.
- `layout_collapse_layout_observations_time_us` and
  `paint_collapse_observations_time_us`: current perf-key registry output does not publish them;
  remaining matches are `fret-diag` compatibility reads/internal aggregate structs/tests and the
  retired-key list.
- `LocalState::new_in(app.models_mut(), ...)`: removed from default examples; remaining matches are
  facade internals/tests and policy-gate markers.
- `AppUiRawActionNotifyExt`: remains an explicit advanced/raw seam plus tests and policy markers,
  not a default app prelude/example surface.

# Verification

Passed after the final child-edge topology fixes:

- `cargo check -p fret-ui`
- `cargo nextest run -p fret-ui set_children_reparents_from_old_parent_without_leaving_stale_child_edges set_children_in_mount_reparents_from_old_parent_without_leaving_stale_child_edges set_children_barrier_reparents_from_old_barrier_without_leaving_stale_child_edges set_children_reparents_from_old_barrier_using_barrier_detach_semantics remove_and_cleanup_handle_deep_trees_on_small_stacks hit_test_handles_deep_trees_on_small_stacks --no-fail-fast`
- `cargo nextest run -p fret-ui --no-fail-fast`
- `cargo nextest run -p fret --lib --no-fail-fast`
- `cargo check -p fret-cookbook --all-targets`
- `cargo check -p fret-examples-imui --all-targets`
- `cargo nextest run -p fret-diag --no-fail-fast`
- `cargo nextest run -p fret-render-wgpu --no-fail-fast`
- `cargo fmt --all --check`
- `git diff --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_adr_numbers.py`
- `python3 tools/check_workstream_catalog.py`
- `python3 ~/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`

# Next Action

Commit this closeout slice. Afterward, Phase 3 can be treated as closed for the retained bridge
deletion plan. The next fearless refactor plan should move to the next architectural frontier rather
than reopening retained parent repair:

- typed frame topology epoch lifecycle and stale-epoch assertions,
- renderer `FrameAssembler`/chunk-native support expansion by stream class,
- text shaping closure hardening beyond current cluster metadata,
- public advanced/raw seam shrink once manual lanes have app-facing replacements.

# Citations

- [Phase 3 retained bridge deletion plan](../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md)
- [Phase 3 U5 parent repair deletion](2026-07-03-phase3-u5-parent-repair-deletion.md)
- [Phase 3 U14 observation-collapse perf key retirement](2026-07-04-phase3-u14-observation-collapse-perf-key-retirement.md)
