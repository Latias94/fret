---
type: Work Progress
title: U8 prepared shape cache budget
tags: fret,u8,text,glyph-cache,renderer,diagnostics
timestamp: 2026-07-01
related_plan: ../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
subagent_id: 019f1d13-066c-7f42-9f4a-fd85b72981d5
---

# Summary

U8's first slice turns the renderer prepared `TextShape` cache from an unbounded `HashMap` into a
budgeted cache with real eviction and diagnostics.

# Implementation

- Added `fret_render_text::prepared_shape_cache_entries()` with native and wasm defaults, controlled
  by `FRET_TEXT_SHAPE_CACHE_ENTRIES`.
- Wrapped `TextLayoutCacheState` prepared shape storage behind a generation/LRU owner with
  `get_shape`, `insert_shape`, `remove_shape`, `shapes`, and entry-limit diagnostics.
- Added per-frame prepared shape cache eviction accounting to `RendererTextPerfSnapshot`,
  `fret-bootstrap` diagnostics snapshots, and `fret-diag` evidence indexes.
- Kept live `TextBlobId` safety: evicting a shape from the prepared cache does not invalidate a
  `TextBlob` that still owns an `Arc<TextShape>`.
- Updated ADR 0143 alignment to record the prepared shape cache budget as implemented while leaving
  glyph atlas page budgets and atlas-revision scene/chunk invalidation as known gaps.

# Guardrails

- No glyph atlas allocator, shader, draw batching, scene encoding cache key, or public `TextBlobId`
  contract changes in this slice.
- `shape_cache_bytes_estimate_total` still intentionally includes both prepared-cache shapes and live
  blob-owned shapes, so entry-limit gates should use `shape_cache_entries` for the runtime cache
  budget and bytes as heap pressure evidence.

# Verification

Passed:

- `cargo fmt --all --check`
- `cargo check -p fret-render-wgpu --all-targets`
- `cargo check -p fret-bootstrap --lib --features ui-app-driver,diagnostics`
- `cargo check -p fret-diag --all-targets`
- `cargo nextest run -p fret-render-wgpu prepared_shape_cache_respects_entry_budget_and_reports_evictions prepared_shape_cache_hit_refreshes_lru_before_eviction prepared_shape_cache_eviction_keeps_live_blob_shape_usable paint_only_changes_miss_blob_cache_but_hit_shape_cache prepare_for_scene_retries_retained_keys_missing_from_reset_atlas prepare_for_scene_pin_cache_removes_replaced_or_missing_blobs --no-fail-fast`
- `cargo nextest run -p fret-render-text --lib --no-fail-fast`
- `cargo nextest run -p fret-diag registered_perf_keys_are_unique registered_perf_key_units_match_names registered_perf_key_contract_keeps_stats_and_gate_keys_additive registered_perf_key_inventory_doc_is_in_sync full_registered_perf_key_registry_covers_consumed_debug_stats_fields bundle_stats_reports_renderer_prepare_text_subphases --no-fail-fast`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/perf/audit_perf_baselines.py --matrix docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md --strict`
- `git diff --check`

# Next

Continue U8 with glyph residency/cache-budget work. Do not remove `text_atlas_revision` from scene
or chunk cache keys until there is a separate proof that frame-driven glyph residency and resource
generations keep cached text output correct.
