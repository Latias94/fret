---
type: Work Progress
title: Phase 3 U4 build-time view-cache membership
tags: fret,phase3,u4,view-cache,gc-liveness,retained-bridge
timestamp: 2026-07-03
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
---

# Summary

This slice removed normal cache-hit retained subtree membership scans and moved view-cache
membership recording to build-time boundary products.

Changes:

- `ElementContext::view_cache` and `view_cache_keep_alive(false)` now record membership from the
  built `AnyElement` subtree before the view-cache scope ends.
- Membership recording includes subtrees that were prebuilt before entering the view-cache closure,
  fixing the cache-enable resize case where the old retained scan used to rediscover those nodes.
- Parent cache membership expands empty nested `ViewCache` children from the child cache root's
  recorded boundary membership, so a parent cache can later reuse without dropping nested live
  descendants.
- `ViewCacheBuildBoundaryStore` now records root membership when a scope begins and records element
  membership while authoring identities are seen.
- Cache-hit replay no longer fails the whole root when a recorded stale member cannot be resolved;
  it filters stale entries, rewrites authoritative membership, and leaves stale node entries for GC.
- The `record_view_cache_reuse_frame` / `view_cache_transitioned_reuse_roots` transition shim and
  retained subtree touch/collect helpers were deleted from normal mount/cache-hit paths.

# Verification

Passed:

- `cargo check -p fret-ui`
- `cargo nextest run -p fret-ui view_cache_keep_alive_revalidates_recorded_membership_before_touching_stale_detached_elements viewport_resize_after_cache_enable_keeps_cache_root_membership_complete viewport_resize_after_cache_enable_keeps_roving_keyed_semantics_and_membership_complete --no-fail-fast`
- `cargo nextest run -p fret-ui view_cache_build_boundary_store_records_element_membership_from_scope_identities view_cache_build_boundary_store_advances_rendered_next_and_clears_frame_local_flags view_cache_build_boundary_store_rebinds_global_membership_to_current_nodes view_cache_subtree_membership_includes_nested_cache_roots keep_alive_view_cache_membership_ignores_stale_nested_cache_roots retained_virtual_list_host_updates_window_without_rerendering_view_cache_root --no-fail-fast`
- `cargo nextest run -p fret-ui view_cache gc_liveness retained_virtual_list --no-fail-fast`
- `cargo nextest run -p fret-ui -E 'not test(stack_safety)' --no-fail-fast`
- `cargo fmt --all --check`
- `git diff --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_adr_numbers.py`
- `python3 tools/check_workstream_catalog.py`
- `python3 ~/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`

# Deletion Gate

Static search over `crates/fret-ui/src` now finds no normal callers or helpers named:

- `collect_declarative_elements_for_existing_subtree`
- `touch_existing_declarative_subtree_seen`
- `mark_existing_declarative_subtree_seen`
- `debug_record_retained_subtree_membership_scan`
- `record_view_cache_reuse_frame`
- `view_cache_transitioned_reuse_roots`

The remaining `retained_subtree_membership_scan_*` matches are historical debug stat fields and
frame resets; U14 owns retiring those compatibility perf keys.

# Remaining Bridges

- `mount.rs` still has two normal calls to `repair_parent_pointers_from_layer_roots`; these are the
  U5 deletion target.
- GC still uses retained storage reachability for cleanup, but cache-hit liveness is now fed by
  boundary-recorded membership and live node revalidation instead of retained subtree scans.
