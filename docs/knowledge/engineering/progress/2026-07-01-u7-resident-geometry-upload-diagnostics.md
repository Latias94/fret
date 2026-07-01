---
type: Work Progress
title: U7 resident geometry upload diagnostics
tags: fret,u7,renderer,geometry-upload,diagnostics
timestamp: 2026-07-01
related_plan: ../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Summary

U7 now has a diagnostics-only resident upload owner inside `GeometryUploadState`.

The owner records upload-stream residency by ring slot for the conservative retained-chunk safe
subset. It reports resident stream candidates, slot hits, misses, dirty-range byte estimates, and
full-upload fallback reasons while preserving the existing full `queue.write_buffer` behavior.

# Implementation

- `StorageRingBuffer` and `RingBuffer` now expose the current upload slot and report whether
  `ensure_capacity` rebuilt buffers.
- `GeometryUploadState` owns `ResidentGeometryUploadState`, scoped to the actual geometry upload
  path and ring slots.
- Resident diagnostics consume `RenderPlan` candidate stream ranges plus prior payload reassembly
  counters from the same frame.
- Fallback reasons distinguish no render-plan candidate, missing retained payload, reassembly
  blockers, uninitialized ring slots, buffer resize invalidation, and stream layout/signature
  changes.
- New counters flow through `GeometryUploadPerfSnapshot`, bootstrap UI frame stats, `fret-diag`,
  and the frame-stats perf-key registry.

# Guardrails

- The new resident hit/miss counters are diagnostics only. They do not skip writes, change bound
  buffers, count render cache hits, or imply retained payloads are renderer output.
- Dirty-range bytes are estimates for candidate streams, not actual partial upload bytes.
- Current resident signatures prove ring-slot ownership and range/fingerprint stability. They
  should be split into layout/range fingerprints and content fingerprints before enabling real
  partial writes.

# Verification

Passed:

- `cargo check -p fret-render-wgpu --all-targets`
- `cargo check -p fret-bootstrap --lib --tests`
- `cargo check -p fret-diag --all-targets`
- `cargo run -p fretboard -- diag stats --perf-keys-json > docs/workstreams/diag-perf-profiling-infra-v1/perf-key-registry.frame-stats.json`
- `cargo nextest run -p fret-render-wgpu resident_upload_diagnostics_are_ring_slot_scoped resident_upload_diagnostics_report_fallback_reasons resident_upload_diagnostics_report_layout_change_and_buffer_resize scene_chunk_manifest_is_reported_without_busting_scene_encoding_cache --no-fail-fast`
- `cargo nextest run -p fret-bootstrap --lib --features "launch ui-app-driver diagnostics" ui_diagnostics::service_tests::patch_latest_renderer_perf_sample_updates_latest_snapshot_stats --no-fail-fast`
- `cargo nextest run -p fret-diag registered_perf_keys_are_unique registered_perf_key_units_match_names registered_perf_key_contract_keeps_stats_and_gate_keys_additive full_registered_perf_key_registry_covers_consumed_debug_stats_fields registered_perf_key_inventory_doc_is_in_sync --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Next

Before real dirty GPU writes, split the resident stream signature into a stable layout/range
signature and a stream-content fingerprint, then add a guarded partial-write dry-run that reports
actual write offsets/counts without changing upload behavior. Only after that should the safe
append-only quad subset switch from full writes to partial writes.

# Citations

- [Plan](../../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md)
- [Reassembly dry-run blockers](2026-07-01-u7-scene-chunk-reassembly-dry-run.md)
- `crates/fret-render-wgpu/src/renderer/geometry_upload.rs`
- `crates/fret-render-wgpu/src/renderer/types.rs`
