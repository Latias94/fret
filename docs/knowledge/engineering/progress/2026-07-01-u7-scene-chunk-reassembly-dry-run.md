---
type: Work Progress
title: U7 scene chunk reassembly dry-run blockers
tags: fret,u7,scene-chunks,renderer,diagnostics
timestamp: 2026-07-01
related_plan: ../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Summary

U7 now classifies retained chunk payloads through a conservative normalized reassembly dry-run.

The dry-run does not reassemble or render cached payloads. It reports whether a matched payload and
flat render-plan segment fall into the first deliberately safe subset: append-only quad payloads
whose shape and upload-stream fingerprint match, with no material state or side-table/effect
rebasing required.

# Implementation

- Extended payload-plan alignment stats with reassembly dry-run counters.
- Counted dry-run candidates only when both a retained payload and flat render-plan candidate
  segment exist.
- Classified the first safe subset as shape match + stream fingerprint match + quad-only payload +
  no clip/mask/effect side tables + no material budget/order state.
- Added blocked-reason counters for shape mismatch, stream fingerprint mismatch, non-quad draws,
  side tables, and material state.
- Propagated the counters through renderer perf snapshots, bootstrap UI frame stats, `fret-diag`,
  and the frame-stats perf-key registry.

# Guardrails

- The dry-run is proof-only. It does not produce a `SceneEncoding`, does not affect render output,
  and does not skip any GPU upload.
- `append_only_matches` is a safe-subset evidence counter, not a render cache hit counter.
- Side-table and material blockers are intentionally conservative. Later slices can split them into
  narrower reasons after rebasing tests exist for uniforms, clips, masks, effect markers, and text
  paint closures.

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

The next U7 cut can introduce a diagnostics-only resident upload owner near `GeometryUploadState`.
It should consume the existing stream fingerprint and dry-run blockers to estimate resident hits,
dirty ranges, and fallback reasons without changing `queue.write_buffer` behavior yet.

# Citations

- [Plan](../../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md)
- [Stream fingerprint diagnostics](2026-07-01-u7-scene-chunk-stream-fingerprint-diagnostics.md)
- Explorer `019f1be0-799d-7ce2-8d8f-2588567c1632`
- Explorer `019f1be0-c5e2-7b23-afdf-3914ad016502`
- `crates/fret-render-wgpu/src/renderer/scene_chunk_encoding_cache.rs`
- `crates/fret-render-wgpu/src/renderer/types.rs`
