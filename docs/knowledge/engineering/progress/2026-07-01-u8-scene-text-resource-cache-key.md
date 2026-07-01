---
type: Work Progress
title: U8 scene text resource cache key
tags: fret,u8,text,glyph-atlas,scene-cache,renderer-cache
timestamp: 2026-07-01
related_plan: docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Summary

U8 now uses the current scene text resource fingerprint as the actual scene and retained scene-chunk
encoding cache key, replacing the earlier global `text_atlas_revision` invalidation input for those
caches. Unreferenced text atlas churn can still be observed through diagnostics, but it no longer
invalidates scene/chunk CPU encodings when the referenced scene glyph resources stay stable.

# Decisions

- Compute the current scene text resource snapshot every render frame, not only when perf diagnostics
  are enabled, because scene and chunk encoding cache keys now depend on its fingerprint.
- Keep the existing `text_atlas_revision` frame-stat fields as diagnostics. They are still useful for
  detecting atlas churn, but they are no longer the scene/chunk cache invalidation authority.
- Keep this as a scene-wide text resource key for now. Per-chunk text resource keys remain a later U8
  optimization once retained chunks prove their text-resource closure independently.

# Changed Files

- `crates/fret-render-wgpu/src/renderer/render_scene/frame_prepare.rs`
- `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs`
- `crates/fret-render-wgpu/src/renderer/render_scene/encoding_cache.rs`
- `crates/fret-render-wgpu/src/renderer/scene_encoding_cache.rs`
- `crates/fret-render-wgpu/src/renderer/scene_chunk_encoding_cache.rs`
- `crates/fret-render-wgpu/src/renderer/scene_encoding_cache_diagnostics.rs`
- `crates/fret-render-wgpu/src/renderer/types.rs`
- `crates/fret-render-wgpu/src/renderer/tests.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/frame_stats.rs`
- `crates/fret-diag/src/perf_keys.rs`
- `docs/workstreams/diag-perf-profiling-infra-v1/perf-key-registry.frame-stats.json`

# Verification

- `cargo fmt --all --check`
- `cargo check -p fret-render-wgpu --all-targets`
- `cargo nextest run -p fret-render-wgpu unreferenced_text_atlas_churn_does_not_bust_scene_or_chunk_encoding_cache record_scene_encoding_cache_frame_result_updates_perf_counters scene_text_resource_snapshot_ignores_unreferenced_atlas_revision_churn text_scene_resource_key_state_counts_atlas_revision_churn_with_stable_resources --no-fail-fast`
- `cargo check -p fret-bootstrap --lib --features ui-app-driver,diagnostics`
- `cargo check -p fret-diag --all-targets`
- `cargo nextest run -p fret-diag registered_perf_keys_are_unique registered_perf_key_units_match_names registered_perf_key_contract_keeps_stats_and_gate_keys_additive registered_perf_key_inventory_doc_is_in_sync full_registered_perf_key_registry_covers_consumed_debug_stats_fields --no-fail-fast`
- `cargo nextest run -p fret-bootstrap --lib --no-fail-fast`
- `cargo check -p fret-render-wgpu --target wasm32-unknown-unknown --lib`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/perf/audit_perf_baselines.py --matrix docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md --strict`
- `git diff --check`

# Next Action

Continue U8 by narrowing text resource invalidation from scene-wide to retained-chunk-local keys, or
remove prepare-time atlas insertion so glyph residency is driven by visible scene text ranges.
