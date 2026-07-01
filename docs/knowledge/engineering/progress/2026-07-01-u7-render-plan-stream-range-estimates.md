---
type: Work Progress
title: U7 render-plan stream range estimates
tags: fret,u7,scene-chunks,render-plan,diagnostics
timestamp: 2026-07-01
related_plan: ../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Summary

U7 now has renderer-local evidence for how much encoded geometry payload a future retained scene
chunk could cover.

This slice deliberately stays in diagnostics. It does not change `SceneEncodingState` cache keys,
does not execute chunk encode reuse, and does not perform partial GPU writes. Flat `Scene` remains
the render truth source.

# Implementation

- `fret_core::SceneChunkManifestEntry::fingerprint()` now mixes chunk content with `local_bounds`
  and `scene_origin`; `SceneChunkManifest::fingerprint()` is order-sensitive and no longer uses XOR.
- `RenderPlanSegment` now carries `RenderPlanSegmentStreamRanges`, populated by the render-plan
  compiler from encoded `OrderedDraw` ranges.
- The estimated candidate upload size is derived from encoded geometry/paint stream ranges:
  quad instances, path paints, text paints, viewport vertices, text glyph instances, text vertices,
  and path vertices.
- Renderer perf exposes two new planning-only counters:
  `render_plan_scene_chunk_candidate_upload_bytes_estimate` and
  `render_plan_scene_chunk_candidate_stream_ranges_changed`.
- Bootstrap frame stats and the `fret-diag` perf-key registry expose the new counters.

# Subagent Result

Explorer `019f1b47-016d-7653-ac4a-f084ceaaba99` confirmed the safe next cut was renderer-local
encoded stream range evidence, not direct use of `SceneChunkManifest` for cache keys or dirty GPU
uploads. It also flagged the old manifest fingerprint as unsafe because it ignored order, repeats,
local bounds, and scene origin.

# Verification

Passed:

- `cargo check -p fret-core --all-targets`
- `cargo check -p fret-render-wgpu --all-targets`
- `cargo check -p fret-bootstrap --lib --tests`
- `cargo check -p fret-diag --all-targets`
- `cargo nextest run -p fret-core scene_chunk_manifest_skips_empty_chunks_and_reports_ops_and_fingerprint --no-fail-fast`
- `cargo nextest run -p fret-render-wgpu compile_for_scene_segments_report_encoded_stream_ranges_and_upload_estimate diff_segment_reports_tracks_shape_changes_and_pass_growth diff_segment_reports_treats_new_shape_candidates_as_changed scene_chunk_manifest_is_reported_without_busting_scene_encoding_cache --no-fail-fast`
- `cargo nextest run -p fret-bootstrap --lib --features "launch ui-app-driver diagnostics" ui_diagnostics::service_tests::patch_latest_renderer_perf_sample_updates_latest_snapshot_stats --no-fail-fast`
- `cargo nextest run -p fret-diag registered_perf_keys_are_unique registered_perf_key_units_match_names registered_perf_key_contract_keeps_stats_and_gate_keys_additive full_registered_perf_key_registry_covers_consumed_debug_stats_fields registered_perf_key_inventory_doc_is_in_sync --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Operational Note

The first bootstrap focused test failed because `/System/Volumes/Data` had only 131 MiB free and
Cargo could not write incremental query cache data. The generated cache directory
`target/debug/incremental` was removed, freeing about 87 GiB. No source files were removed.

# Next

The next U7 cut should design a real renderer chunk encode/cache owner. It must include per-chunk
resource-generation context before reporting cache hits, dirty chunk counts, or dirty upload bytes.

# Citations

- [Plan](../../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md)
- [Renderer input bridge](2026-07-01-u7-scene-chunk-renderer-input-bridge.md)
- Subagent `019f1b47-016d-7653-ac4a-f084ceaaba99`
- `crates/fret-core/src/scene/manifest.rs`
- `crates/fret-render-wgpu/src/renderer/render_plan.rs`
- `crates/fret-render-wgpu/src/renderer/render_plan_compiler/context.rs`
- `crates/fret-render-wgpu/src/renderer/render_plan_reporting.rs`
