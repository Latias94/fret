---
type: Work Progress
title: U7 resident partial-write dry-run diagnostics
tags: fret,u7,renderer,geometry-upload,partial-write,diagnostics
timestamp: 2026-07-01
related_plan: ../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Summary

U7 now separates resident geometry upload layout/range signatures from stream-content
fingerprints.

When a resident stream has the same ring slot and layout/range but different content, the renderer
records partial-write dry-run counters: candidate stream count, estimated write count, and estimated
bytes. Full `queue.write_buffer` uploads are still preserved.

# Implementation

- `ResidentGeometryUploadStreamSignature` now stores layout and content fingerprints separately.
- Content fingerprints are computed from the actual typed geometry upload slices for each
  candidate stream range.
- The resident diagnostics owner distinguishes full resident hits, content mismatches,
  stream layout/range changes, uninitialized slots, and resize invalidations.
- Added partial-write dry-run counters for streams, write-count estimate, and byte estimate.
- Propagated the counters through `GeometryUploadPerfSnapshot`, bootstrap UI frame stats,
  `fret-diag`, and the frame-stats perf-key registry.

# Guardrails

- Partial-write counters are still dry-run evidence only. No upload offsets are written and no
  `queue.write_buffer` call is skipped or narrowed.
- Content mismatches are treated as full-upload fallbacks in runtime behavior while also reporting
  the dry-run partial-write opportunity.
- Real partial writes should be enabled only behind the append-only safe subset and only after
  tests prove buffer offsets/counts match the current full-upload rendering output.

# Verification

Passed:

- `cargo check -p fret-render-wgpu --all-targets`
- `cargo check -p fret-bootstrap --lib --tests`
- `cargo check -p fret-diag --all-targets`
- `cargo run -p fretboard -- diag stats --perf-keys-json > docs/workstreams/diag-perf-profiling-infra-v1/perf-key-registry.frame-stats.json`
- `cargo nextest run -p fret-render-wgpu resident_upload_diagnostics_are_ring_slot_scoped resident_upload_diagnostics_report_fallback_reasons resident_upload_diagnostics_report_layout_change_and_buffer_resize resident_upload_diagnostics_report_content_change_partial_write_dry_run scene_chunk_manifest_is_reported_without_busting_scene_encoding_cache --no-fail-fast`
- `cargo nextest run -p fret-bootstrap --lib --features "launch ui-app-driver diagnostics" ui_diagnostics::service_tests::patch_latest_renderer_perf_sample_updates_latest_snapshot_stats --no-fail-fast`
- `cargo nextest run -p fret-diag registered_perf_keys_are_unique registered_perf_key_units_match_names registered_perf_key_contract_keeps_stats_and_gate_keys_additive full_registered_perf_key_registry_covers_consumed_debug_stats_fields registered_perf_key_inventory_doc_is_in_sync --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Next

The next U7 slice can add a guarded partial-write execution path for the append-only quad safe
subset. It should preserve full-write fallback for all blocked reasons and include a focused test
that validates byte offsets/counts before any visual/perf claim is made.

# Citations

- [Plan](../../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md)
- [Resident geometry upload diagnostics](2026-07-01-u7-resident-geometry-upload-diagnostics.md)
- `crates/fret-render-wgpu/src/renderer/geometry_upload.rs`
- `crates/fret-render-wgpu/src/renderer/types.rs`
