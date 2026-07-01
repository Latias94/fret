---
type: Work Progress
title: U8 scene chunk text resource cache key
tags: fret,u8,text,glyph-atlas,scene-chunks,renderer-cache
timestamp: 2026-07-01
related_plan: docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Summary

U8 now narrows retained scene-chunk encoding cache invalidation from the whole-scene text resource
fingerprint to an entry-local text resource key derived from each chunk's `TextBlobId` side index.
Scene encoding still uses the whole-scene key, but retained chunk CPU payloads no longer miss merely
because another retained chunk's referenced glyph resources changed.

# Decisions

- Keep `SceneChunkManifest` as a portable `fret-core` contract. The renderer derives entry-local text
  keys from `entry.chunk().text_blob_ids()` and `TextSystem`, rather than storing renderer text
  resource fingerprints in the manifest.
- Treat chunks with no text blobs as key `0`. Empty text chunks must not inherit atlas churn or atlas
  reset generation.
- Keep visible-range glyph residency out of this slice. Explorers confirmed prepare-time atlas
  insertion and whole-scene glyph prewarm still need a larger U8 cut.

# Changed Files

- `crates/fret-render-wgpu/src/text/diagnostics.rs`
- `crates/fret-render-wgpu/src/text/tests.rs`
- `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs`
- `crates/fret-render-wgpu/src/renderer/scene_chunk_encoding_cache.rs`

# Verification

- `cargo check -p fret-render-wgpu --all-targets`
- `cargo nextest run -p fret-render-wgpu empty_text_resource_snapshot_ignores_atlas_churn_and_reset chunk_encoding_payload_cache_uses_entry_local_text_resource_keys unreferenced_text_atlas_churn_does_not_bust_scene_or_chunk_encoding_cache scene_text_resource_snapshot_ignores_unreferenced_atlas_revision_churn --no-fail-fast`
- `cargo nextest run -p fret-render-wgpu chunk_encoding_key_cache_tracks_hits_misses_and_stale_slots chunk_encoding_key_cache_accounts_for_duplicate_entries_by_slot chunk_encoding_key_cache_context_changes_invalidate_entries chunk_encoding_payload_cache_builds_only_misses_and_evicts_stale_payloads chunk_encoding_payload_cache_uses_entry_local_text_resource_keys payload_plan_alignment_compares_cached_payloads_to_candidate_segments_in_order payload_plan_alignment_returns_exact_safe_segment_indices --no-fail-fast`
- `cargo check -p fret-render-wgpu --target wasm32-unknown-unknown --lib`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/perf/audit_perf_baselines.py --matrix docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md --strict`
- `git diff --check`

# Next Action

Continue U8 by cutting off prepare-time atlas insertion, then replace whole-scene text prewarm with a
visible glyph residency prepass.
