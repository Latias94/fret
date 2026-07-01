---
type: Work Progress
title: U7 runtime resident write-plan decoupling
tags: fret,u7,renderer,geometry-upload,scene-chunks,runtime
timestamp: 2026-07-01
related_plan: ../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Summary

U7 scene chunk payload alignment and resident upload slot tracking are no longer gated by renderer
perf collection.

The previous exact reassembly plan was correct but diagnostics-owned: payloads were built only when
`perf_enabled` was true, payload-plan alignment returned an empty plan when perf was off, and
resident stream slot signatures were updated only while recording perf counters. That would make a
future partial upload path change behavior only when diagnostics were enabled. This slice makes the
runtime planning path unconditional while keeping perf counters optional.

# Implementation

- Removed the old key-only `SceneChunkEncodingState::begin_frame(...)` path.
- `record_scene_chunk_encoding_key_cache_for_frame(...)` now always maintains cached scene chunk
  payloads; `perf_enabled` only controls whether cache stats are copied to `RenderPerfStats`.
- `record_scene_chunk_payload_plan_alignment_for_frame(...)` now returns
  `SceneChunkPayloadPlanAlignment` for runtime use even when perf is disabled.
- Geometry upload now receives the full payload alignment object and updates resident stream slot
  signatures every frame. It records geometry-upload counters only when a perf recorder is present.
- Resident upload fallback diagnostics now read payload alignment stats directly instead of reading
  reassembly counters back out of `RenderPerfStats`.
- Added focused tests proving resident slot state can warm without a perf recorder and that a
  renderer can warm scene chunk payloads plus resident upload slots across perf-off frames.

# Guardrails

- Runtime GPU writes are still full-buffer writes.
- No diagnostics schema or perf key names changed in this slice.
- Resident write decisions are now safe to consume from normal runtime code in the next slice, but
  actual `queue.write_buffer` partial writes have not been enabled yet.

# Verification

Passed:

- `cargo check -p fret-render-wgpu --all-targets`
- `cargo nextest run -p fret-render-wgpu chunk_encoding_key_cache_tracks_hits_misses_and_stale_slots chunk_encoding_key_cache_accounts_for_duplicate_entries_by_slot chunk_encoding_key_cache_context_changes_invalidate_entries chunk_encoding_payload_cache_builds_only_misses_and_evicts_stale_payloads payload_plan_alignment_compares_cached_payloads_to_candidate_segments_in_order payload_plan_alignment_returns_exact_safe_segment_indices resident_upload_diagnostics_are_ring_slot_scoped resident_upload_state_warms_without_perf_recorder resident_upload_diagnostics_report_fallback_reasons resident_upload_diagnostics_report_content_change_partial_write_dry_run resident_upload_diagnostics_use_exact_safe_segment_indices resident_partial_write_dry_run_blocks_incomplete_stream_coverage resident_partial_write_dry_run_counts_changed_ranges scene_chunk_payload_and_resident_upload_state_warm_without_perf scene_chunk_manifest_is_reported_without_busting_scene_encoding_cache --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Next

Implement actual guarded quad instance partial uploads. Start with a `ResidentGeometryUpload*WritePlan`
returned from resident slot tracking, support `Full`, `Skip`, and `Partial(ranges)`, and only use
`Skip` / `Partial` when the stream is resident-compatible and the safe ranges cover the whole quad
instance stream.

# Citations

- [Plan](../../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md)
- [Resident stream coverage gate](2026-07-01-u7-resident-stream-coverage-gate.md)
- `crates/fret-render-wgpu/src/renderer/scene_chunk_encoding_cache.rs`
- `crates/fret-render-wgpu/src/renderer/geometry_upload.rs`
- `crates/fret-render-wgpu/src/renderer/tests.rs`
