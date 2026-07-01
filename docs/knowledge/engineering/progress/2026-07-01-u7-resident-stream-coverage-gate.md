---
type: Work Progress
title: U7 resident stream coverage gate
tags: fret,u7,renderer,geometry-upload,scene-chunks,diagnostics
timestamp: 2026-07-01
related_plan: ../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Summary

U7 resident upload diagnostics now reject partial-write dry-run opportunities when retained safe
ranges do not cover the whole GPU stream.

The exact safe segment list proved which retained payload ranges can be trusted, but it did not
prove that every instance in a resident GPU buffer slot is represented by those safe ranges. A real
skip or partial upload would leave stale buffer contents for any uncovered range. This slice adds a
full-stream coverage gate before partial-write dry-run counters can report an executable dirty
range plan.

# Implementation

- Added `resident_stream_signatures_cover_stream(...)` to validate sorted/merged resident ranges
  cover `[0, stream_len)` with no gaps or out-of-bounds ranges.
- Passed each upload stream length into resident stream diagnostics.
- Added `resident_stream_coverage_gaps` to renderer geometry-upload perf snapshots, bootstrap frame
  stats, `fret-diag` perf keys, and the generated frame-stats perf-key registry.
- Changed content-mismatch dry-run accounting so partial-write stream/write/byte estimates are
  recorded only when the safe resident ranges cover the full stream.
- Added tests for adjacent/overlapping coverage, exact safe segment coverage gaps, and blocked
  partial-write dry-run accounting.

# Guardrails

- Runtime uploads are still full-buffer writes.
- Coverage gaps are diagnostic evidence, not a behavior change yet.
- The next runtime slice must first decouple resident write planning from perf-only payload
  alignment; otherwise real upload behavior would change only when renderer perf collection is
  enabled.

# Verification

Passed:

- `cargo fmt --all --check`
- `cargo check -p fret-render-wgpu --all-targets`
- `cargo nextest run -p fret-render-wgpu resident_stream_coverage_requires_complete_stream_ranges resident_upload_diagnostics_report_content_change_partial_write_dry_run resident_upload_diagnostics_use_exact_safe_segment_indices resident_partial_write_dry_run_blocks_incomplete_stream_coverage resident_partial_write_dry_run_counts_changed_ranges --no-fail-fast`
- `cargo check -p fret-bootstrap --lib`
- `cargo check -p fret-bootstrap --lib --features ui-app-driver,diagnostics`
- `cargo nextest run -p fret-bootstrap --lib --features ui-app-driver,diagnostics --no-fail-fast`
- `cargo nextest run -p fret-diag registered_perf_keys_are_unique registered_perf_key_units_match_names registered_perf_key_contract_keeps_stats_and_gate_keys_additive registered_perf_key_inventory_doc_is_in_sync full_registered_perf_key_registry_covers_consumed_debug_stats_fields --no-fail-fast`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Next

Refactor renderer scene chunk payload alignment so the exact reassembly/write plan can be computed
for runtime upload decisions independently of diagnostics. After that, implement guarded quad
instance partial writes with full-upload fallback for uninitialized slots, buffer resize,
layout/range changes, missing payloads, blocked reassembly, and coverage gaps.

# Citations

- [Plan](../../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md)
- [Exact resident write-plan diagnostics](2026-07-01-u7-exact-resident-write-plan.md)
- `crates/fret-render-wgpu/src/renderer/geometry_upload.rs`
- `crates/fret-render-wgpu/src/renderer/types.rs`
