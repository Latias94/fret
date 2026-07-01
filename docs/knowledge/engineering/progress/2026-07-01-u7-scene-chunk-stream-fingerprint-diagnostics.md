---
type: Work Progress
title: U7 scene chunk stream fingerprint diagnostics
tags: fret,u7,scene-chunks,renderer,diagnostics
timestamp: 2026-07-01
related_plan: ../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Summary

U7 now compares retained chunk payload upload-stream bytes against the corresponding flat-scene
`RenderPlan` candidate segment stream ranges.

This is still diagnostics-only. Cached payloads do not feed renderer output, render cache hit
counts, resident GPU ranges, or partial uploads. The new counters strengthen the previous shape
alignment proof by detecting when a retained payload has the same draw/stream shape but different
stream contents.

# Implementation

- Added a normalized upload-stream fingerprint for cached `SceneEncoding` payloads.
- Compared each payload fingerprint to the matching flat `SceneEncoding` segment range using the
  render-plan candidate stream ranges.
- Exposed `scene_chunk_encoding_payload_plan_stream_fingerprint_matches` and
  `scene_chunk_encoding_payload_plan_stream_fingerprint_mismatches` through renderer perf,
  bootstrap UI frame stats, `fret-diag`, and the frame-stats perf-key registry.
- Added a unit test that proves both a matching stream and a byte-different stream are reported
  correctly.
- Kept the renderer integration fixture as a mismatch case: its manifest translates the chunk while
  the flat scene remains at the original origin, so shape matches but byte fingerprint does not.

# Subagent Results

Explorer `019f1be0-799d-7ce2-8d8f-2588567c1632` confirmed that full deterministic reassembly still
needs more than upload stream parity. Missing pieces include entry-state metadata, side-table
closures, offset rebasing for uniforms/clips/masks/effect markers, material budget/order state,
text atlas state, resource side indexes, and batching-boundary evidence.

Explorer `019f1be0-c5e2-7b23-afdf-3914ad016502` confirmed partial GPU upload should start with a
resident upload owner near `GeometryUploadState`, not render-plan compilation. The owner must be
stream + ring-slot scoped and must record miss reasons such as cold slot, buffer reallocation,
context change, candidate/range/fingerprint mismatch, unsupported upload path, and excessive
fragmentation.

# Guardrails

- Do not treat stream fingerprint matches as render cache hits.
- Do not use cached chunk payloads as renderer output until normalized reassembly dry-runs cover
  side tables and blocked reasons.
- Do not report dirty chunk counts or partial upload bytes until resident stream-range ownership
  exists.
- Full upload remains required for streams outside candidate ownership and for uniforms, clips,
  masks, effect/custom buffers, text atlas, SVG/image/render-target uploads, and unsupported paths.

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

The next U7 cut should add a normalized reassembly dry-run for a deliberately safe subset and
explicit blocked-reason counters for chunks that require entry-state, material/text/resource/effect
state, or unsupported side-table rebasing. Resident GPU range ownership should follow only after
that dry-run distinguishes reusable chunks from blocked chunks.

# Citations

- [Plan](../../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md)
- [Payload-plan alignment](2026-07-01-u7-scene-chunk-payload-plan-alignment.md)
- Explorer `019f1be0-799d-7ce2-8d8f-2588567c1632`
- Explorer `019f1be0-c5e2-7b23-afdf-3914ad016502`
- `crates/fret-render-wgpu/src/renderer/scene_chunk_encoding_cache.rs`
- `crates/fret-render-wgpu/src/renderer/geometry_upload.rs`
- `crates/fret-render-wgpu/src/renderer/render_plan.rs`
