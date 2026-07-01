---
type: Work Progress
title: U7 scene chunk payload-plan alignment diagnostics
tags: fret,u7,scene-chunks,renderer,diagnostics
timestamp: 2026-07-01
related_plan: ../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Summary

U7 can now compare retained scene chunk CPU payload metadata against flat-scene `RenderPlan`
candidate segments.

This is diagnostics-only. Cached chunk payloads still do not feed renderer output, render-plan
dispatch, or GPU uploads. The frame stats prove whether payload shapes and plan segment shapes line
up before later slices attempt deterministic payload reassembly or dirty range writes.

# Implementation

- Added reusable renderer helpers for `RenderPlanSegmentFlags::for_ordered_draws(...)` and
  `RenderPlanSegmentStreamRanges::for_ordered_draws(...)`.
- Added `RenderPlanSegmentStreamShape` so payload metadata can compare stream lengths instead of
  absolute full-scene stream offsets.
- Stored a coarse `SceneChunkPayloadPlanShape` beside each cached CPU chunk payload:
  draw count, diagnostics flags mask, and stream shape.
- Added `SceneChunkEncodingState::record_payload_plan_alignment(...)`, which walks cached manifest
  payload keys and eligible render-plan candidate segments in order.
- Exposed frame stats through renderer perf snapshots, bootstrap UI frame stats, `fret-diag`, and
  the frame-stats perf-key registry:
  `scene_chunk_encoding_payload_plan_candidate_segments`,
  `scene_chunk_encoding_payload_plan_shape_matches`,
  `scene_chunk_encoding_payload_plan_shape_mismatches`,
  `scene_chunk_encoding_payload_entries_without_plan_candidate`, and
  `scene_chunk_encoding_payload_plan_candidates_without_payload`.

# Guardrails

- Do not interpret `payload_plan_shape_matches` as render cache hits. The cached payloads are not
  consumed by the render path.
- Do not report dirty chunk counts or partial upload bytes from these counters.
- The shape comparison is intentionally coarse. It proves candidate ordering and stream footprint
  compatibility, not full material ordering, resource residency, or byte-for-byte encoded stream
  identity.

# Verification

Passed:

- `cargo check -p fret-render-wgpu --all-targets`
- `cargo check -p fret-bootstrap --lib --tests`
- `cargo check -p fret-diag --all-targets`
- `cargo nextest run -p fret-render-wgpu chunk_encoding_key_cache_tracks_hits_misses_and_stale_slots chunk_encoding_key_cache_accounts_for_duplicate_entries_by_slot chunk_encoding_key_cache_context_changes_invalidate_entries chunk_encoding_payload_cache_builds_only_misses_and_evicts_stale_payloads payload_plan_alignment_compares_cached_payloads_to_candidate_segments_in_order scene_chunk_manifest_is_reported_without_busting_scene_encoding_cache --no-fail-fast`
- `cargo nextest run -p fret-bootstrap --lib --features "launch ui-app-driver diagnostics" ui_diagnostics::service_tests::patch_latest_renderer_perf_sample_updates_latest_snapshot_stats --no-fail-fast`
- `cargo nextest run -p fret-diag registered_perf_keys_are_unique registered_perf_key_units_match_names registered_perf_key_contract_keeps_stats_and_gate_keys_additive full_registered_perf_key_registry_covers_consumed_debug_stats_fields registered_perf_key_inventory_doc_is_in_sync --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Next

The next U7 cut should use this alignment evidence to design deterministic payload reassembly and
resident stream-range ownership before introducing true chunk render-cache hits, dirty chunk counts,
or partial GPU upload counters.

# Citations

- [Plan](../../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md)
- [Payload cache](2026-07-01-u7-scene-chunk-encoding-payload-cache.md)
- `crates/fret-render-wgpu/src/renderer/scene_chunk_encoding_cache.rs`
- `crates/fret-render-wgpu/src/renderer/render_plan.rs`
- `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs`
- `crates/fret-render-wgpu/src/renderer/tests.rs`
