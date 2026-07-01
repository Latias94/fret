---
type: Work Progress
title: U7 scene chunk encoding payload cache
tags: fret,u7,scene-chunks,renderer,diagnostics
timestamp: 2026-07-01
related_plan: ../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Summary

U7 now has a renderer-owned CPU encoded payload cache for retained scene chunk diagnostics.

This cache is populated only while renderer perf collection is enabled. It intentionally does not
feed the main `SceneEncoding`, render-plan compiler, dispatch, or GPU uploads. Flat `Scene` remains
the render output source.

# Implementation

- Extended `SceneChunkEncodingState` to retain `CachedSceneChunkEncoding` payloads by
  `SceneChunkEncodingKey`.
- Payload misses replay each retained `SceneChunkManifestEntry` into a scratch `Scene` using
  `SceneChunk::replay_translated_into(...)`, then reuse the existing renderer scene encoder to
  produce a CPU `SceneEncoding` payload.
- Payload hits reuse retained CPU payloads for diagnostics only.
- Payloads not referenced by the current manifest are evicted.
- Renderer perf exposes:
  `scene_chunk_encoding_payload_cache_hits`,
  `scene_chunk_encoding_payload_cache_misses`,
  `scene_chunk_encoding_payload_chunks_encoded`,
  `scene_chunk_encoding_payload_bytes_estimate`, and
  `scene_chunk_encoding_payload_entries_live`.
- The existing scene encoder visibility is widened only inside `crate::renderer`; no public API is
  exposed.

# Guardrails

- Do not interpret payload cache hits as render cache hits. The cached payloads are not consumed by
  the render path yet.
- Do not report dirty chunk counts or dirty upload bytes from this cache.
- Independent chunk payload encoding may still miss full-scene ordering context such as material
  budget `material_seen` order. Deterministic reassembly must be proven before using payloads for
  output.

# Verification

Passed:

- `cargo check -p fret-render-wgpu --all-targets`
- `cargo check -p fret-bootstrap --lib --tests`
- `cargo check -p fret-diag --all-targets`
- `cargo nextest run -p fret-render-wgpu chunk_encoding_key_cache_tracks_hits_misses_and_stale_slots chunk_encoding_key_cache_accounts_for_duplicate_entries_by_slot chunk_encoding_key_cache_context_changes_invalidate_entries chunk_encoding_payload_cache_builds_only_misses_and_evicts_stale_payloads scene_chunk_manifest_is_reported_without_busting_scene_encoding_cache --no-fail-fast`
- `cargo nextest run -p fret-bootstrap --lib --features "launch ui-app-driver diagnostics" ui_diagnostics::service_tests::patch_latest_renderer_perf_sample_updates_latest_snapshot_stats --no-fail-fast`
- `cargo nextest run -p fret-diag registered_perf_keys_are_unique registered_perf_key_units_match_names registered_perf_key_contract_keeps_stats_and_gate_keys_additive full_registered_perf_key_registry_covers_consumed_debug_stats_fields registered_perf_key_inventory_doc_is_in_sync --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Next

The next U7 cut should compare cached chunk payload metadata against the flat-scene `RenderPlan`
segments and prove deterministic reassembly boundaries before any renderer output consumes cached
payloads.

# Citations

- [Plan](../../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md)
- [Encoding key cache](2026-07-01-u7-scene-chunk-encoding-key-cache.md)
- `crates/fret-render-wgpu/src/renderer/scene_chunk_encoding_cache.rs`
- `crates/fret-render-wgpu/src/renderer/render_scene/encode/mod.rs`
- `crates/fret-render-wgpu/src/renderer/tests.rs`
