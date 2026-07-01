---
type: Work Progress
title: U7 guarded quad instance partial upload
tags: fret,u7,renderer,geometry-upload,scene-chunks,runtime
timestamp: 2026-07-01
related_plan: ../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Summary

U7 now has the first real guarded dirty GPU upload path: resident-compatible quad instance streams can
skip exact reuploads or write only changed safe ranges.

The previous slice made resident write planning available without perf collection, but runtime still
wrote the full quad instance buffer. This slice consumes the resident plan for quad instances only and
keeps conservative fallbacks for all unsafe cases.

# Implementation

- Added a resident geometry upload write plan with `Full`, `Skip`, and `Partial(ranges)` decisions.
- Quad instance uploads now skip exact resident hits and issue per-range `queue.write_buffer` calls
  for resident-compatible content changes.
- Partial writes are allowed only when safe resident ranges cover the entire stream. Coverage gaps,
  shape/layout mismatches, uninitialized slots, invalid ranges, empty reassembly plans, and missing
  eligible candidates fall back to full uploads.
- Resident slots are invalidated when a frame has no eligible resident proof for a non-empty stream,
  preventing stale exact-hit decisions after a fallback frame.
- Layout fingerprints now track stream range shape while content fingerprints track byte changes, so
  stable ranges with changed bytes can become partial writes instead of layout fallbacks.
- Non-quad streams still run the diagnostics/planning path but continue to use full uploads.

# Guardrails

- This is intentionally quad-only. Path, text, glyph, and vertex streams need separate safety checks
  before real partial writes.
- The flat `Scene` renderer input remains the semantic source of truth.
- No diagnostics schema or perf key names changed in this slice.

# Verification

Passed:

- `git diff --check`
- `cargo fmt --all --check`
- `cargo check -p fret-render-wgpu --all-targets`
- `cargo nextest run -p fret-render-wgpu pod_upload_range_reports_byte_offsets resident_upload_diagnostics_are_ring_slot_scoped resident_upload_state_warms_without_perf_recorder resident_upload_state_invalidates_slot_when_reassembly_plan_is_empty resident_upload_diagnostics_report_fallback_reasons resident_upload_diagnostics_report_content_change_partial_write_dry_run resident_upload_diagnostics_use_exact_safe_segment_indices resident_partial_write_dry_run_blocks_incomplete_stream_coverage resident_partial_write_dry_run_counts_changed_ranges resident_quad_partial_upload_writes_only_changed_range scene_chunk_payload_and_resident_upload_state_warm_without_perf --no-fail-fast`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/perf/audit_perf_baselines.py --matrix docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md --strict`

# Next

Close out U7 by deciding whether the remaining non-quad streams need guarded partial writes now or
whether the next higher-value slice is U8 text/glyph cache budgets. A narrow U7 closeout should audit
the renderer payload contract fields and record explicit retained-vs-deferred evidence before moving
to U8.
