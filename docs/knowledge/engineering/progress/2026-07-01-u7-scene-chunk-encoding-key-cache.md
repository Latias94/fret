---
type: Work Progress
title: U7 scene chunk encoding key cache
tags: fret,u7,scene-chunks,renderer,diagnostics
timestamp: 2026-07-01
related_plan: ../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Summary

U7 now has a renderer-owned retained scene chunk encoding key-cache owner.

This slice is deliberately a key lookup owner only. It does not store encoded payloads, does not
reuse `SceneEncoding`, does not alter whole-scene encoding cache keys, does not change render-plan
dispatch, and does not perform partial GPU writes. Flat `Scene` remains the only render semantics
source.

# Implementation

- Added renderer-private `SceneChunkEncodingState`.
- Added `SceneChunkEncodingContext`, keyed only by renderer-owned context:
  format, viewport, scale factor, render target and image generations, text atlas revision, text
  quality key, material generation/budgets, and custom-effect generation.
- Each manifest entry is keyed by the renderer context plus retained scene chunk entry fingerprint,
  chunk fingerprint, and chunk ops length.
- The owner tracks duplicate entries by slot, reports stale previous entries, and treats context
  changes as misses.
- Renderer perf exposes conservative key-cache counters:
  `scene_chunk_encoding_key_cache_entries`, `scene_chunk_encoding_key_cache_hits`,
  `scene_chunk_encoding_key_cache_misses`,
  `scene_chunk_encoding_key_cache_stale_entries`, and
  `scene_chunk_encoding_key_cache_context_fingerprint`.
- Bootstrap frame stats and the `fret-diag` perf-key registry expose the new counters.

# Subagent Results

Explorer `019f1b77-cabe-7310-981b-79356db9587a` confirmed the GPUI/Zed reference stack supports
retained ownership above renderer identity, but does not provide a directly reusable renderer-side
chunk encode cache. It specifically warned not to leak `ViewId`, component policy, or UI runtime
identity into renderer keys.

Explorer `019f1b77-801f-77e1-99aa-185644ce1e2a` confirmed the current next step should remain
renderer-owned and conservative: key-cache ownership first, CPU encoded payload cache later, and
dirty GPU range uploads only after resident range ownership exists.

# Verification

Passed:

- `cargo check -p fret-render-wgpu --all-targets`
- `cargo check -p fret-bootstrap --lib --tests`
- `cargo check -p fret-diag --all-targets`
- `cargo nextest run -p fret-render-wgpu chunk_encoding_key_cache_tracks_hits_misses_and_stale_slots chunk_encoding_key_cache_accounts_for_duplicate_entries_by_slot chunk_encoding_key_cache_context_changes_invalidate_entries scene_chunk_manifest_is_reported_without_busting_scene_encoding_cache --no-fail-fast`
- `cargo nextest run -p fret-bootstrap --lib --features "launch ui-app-driver diagnostics" ui_diagnostics::service_tests::patch_latest_renderer_perf_sample_updates_latest_snapshot_stats --no-fail-fast`
- `cargo nextest run -p fret-diag registered_perf_keys_are_unique registered_perf_key_units_match_names registered_perf_key_contract_keeps_stats_and_gate_keys_additive full_registered_perf_key_registry_covers_consumed_debug_stats_fields registered_perf_key_inventory_doc_is_in_sync --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Next

The next U7 cut can promote the key owner into a CPU encoded payload cache, but it must keep flat
`Scene` rendering as the output source until cached chunks and fresh chunks can be deterministically
reassembled into the same `SceneEncoding`. Do not report dirty chunk counts, render cache hits, or
partial upload bytes before that point.

# Citations

- [Plan](../../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md)
- [Stream range estimates](2026-07-01-u7-render-plan-stream-range-estimates.md)
- Subagent `019f1b77-cabe-7310-981b-79356db9587a`
- Subagent `019f1b77-801f-77e1-99aa-185644ce1e2a`
- `crates/fret-render-wgpu/src/renderer/scene_chunk_encoding_cache.rs`
- `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs`
- `crates/fret-render-wgpu/src/renderer/types.rs`
