---
type: Work Progress
title: U7 exact resident write-plan diagnostics
tags: fret,u7,renderer,geometry-upload,scene-chunks,diagnostics
timestamp: 2026-07-01
related_plan: ../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Summary

U7 resident upload diagnostics now use an exact safe segment list from scene chunk payload
alignment.

The previous dry-run could only infer safe candidates from aggregate counters. This slice returns
the concrete `RenderPlan` segment indices that passed the append-only safe subset and uses those
indices to build resident stream signatures. It also changes resident stream signatures from
stream-union granularity to per-safe-range granularity, so partial-write dry-run write-count and
byte estimates match the ranges a future guarded upload path would write.

# Implementation

- Added `SceneChunkPayloadReassemblyPlan` and `SceneChunkPayloadPlanAlignment` to carry exact safe
  segment indices alongside existing counters.
- `record_scene_chunk_payload_plan_alignment_for_frame` now returns the exact reassembly plan to
  the render upload phase.
- `GeometryUploadState` consumes exact segment indices instead of `append_only_matches` count.
- Resident stream state stores ordered per-range signatures per ring slot, not a single union
  signature.
- Added tests proving a safe second segment is not confused with a blocked first segment, and that
  partial-write dry-run write counts are range counts rather than stream-union counts.

# Guardrails

- Runtime uploads are still full-buffer writes.
- The exact plan is still diagnostics-only and only exists under renderer perf collection.
- Real partial writes still need a guarded execution path that uses these exact ranges and preserves
  full-write fallback for uninitialized slots, buffer resizes, layout/range changes, missing
  payloads, and blocked reassembly candidates.

# Verification

Passed:

- `cargo check -p fret-render-wgpu --all-targets`
- `cargo nextest run -p fret-render-wgpu payload_plan_alignment_compares_cached_payloads_to_candidate_segments_in_order payload_plan_alignment_returns_exact_safe_segment_indices resident_upload_diagnostics_are_ring_slot_scoped resident_upload_diagnostics_report_fallback_reasons resident_upload_diagnostics_report_layout_change_and_buffer_resize resident_upload_diagnostics_report_content_change_partial_write_dry_run resident_upload_diagnostics_use_exact_safe_segment_indices resident_partial_write_dry_run_counts_changed_ranges scene_chunk_manifest_is_reported_without_busting_scene_encoding_cache --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Next

Implement the guarded partial-write execution path for resident-compatible append-only quad
ranges. Start with quad instances only, keep all existing full-write fallbacks, and add a focused
test for byte offset/count calculation before expanding beyond the safe subset.

# Citations

- [Plan](../../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md)
- [Resident partial-write dry-run diagnostics](2026-07-01-u7-resident-partial-write-dry-run.md)
- `crates/fret-render-wgpu/src/renderer/scene_chunk_encoding_cache.rs`
- `crates/fret-render-wgpu/src/renderer/geometry_upload.rs`
