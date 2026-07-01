---
type: Work Progress
title: U7 render-plan scene chunk candidate telemetry
tags: fret,renderer,scene,perf,u7
timestamp: 2026-07-01
related_plan: docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
source_session: 019f143b-4f62-7333-a9b1-c3c54cf1409e
---

# Summary

U7's renderer-facing follow-up adds evidence-only retained scene chunk candidate telemetry at the
render-plan segment layer. `RenderPlanSegment` now carries `scene_chunk_candidate` metadata derived
from the flat `SceneEncoding` draw range, ordered-draw shape, resource ids, primitive flags, and
start uniform fingerprint.

The renderer still consumes flat `Scene` input. This slice does not add chunk encode caching, dirty
GPU range uploads, `SceneEncodingState` cache-key changes, or direct `BoundarySceneChunkManifest`
consumption.

# Decision

Use candidate vocabulary until renderer chunk encoding really exists.

- `render_plan_scene_chunk_candidates` counts segment-level candidates for future retained chunk
  encoding.
- `render_plan_scene_chunk_candidate_draws` counts ordered draws covered by those candidates.
- `render_plan_scene_chunk_candidates_stable` / `changed` compare candidate fingerprints against the
  previous render-plan segment report.
- The fingerprint is diagnostics evidence, not a cache key.

# Verified State

Relevant checks passed:

- `cargo check -p fret-render-wgpu --all-targets`
- `cargo check -p fret-bootstrap --lib --tests`
- `cargo check -p fret-diag --all-targets`
- `cargo nextest run -p fret-render-wgpu diff_segment_reports_tracks_shape_changes_and_pass_growth diff_segment_reports_treats_new_shape_candidates_as_changed render_plan_dump_assembly_tracks_segment_passes_and_counts requested_and_emitted_custom_effect_counters_track_all_versions degradation_counters_track_reason_and_kind_totals --no-fail-fast`
- `cargo nextest run -p fret-bootstrap --lib --features "launch ui-app-driver diagnostics" ui_diagnostics::service_tests::patch_latest_renderer_perf_sample_updates_latest_snapshot_stats --no-fail-fast`
- `cargo nextest run -p fret-diag registered_perf_keys_are_unique registered_perf_key_units_match_names registered_perf_key_contract_keeps_stats_and_gate_keys_additive full_registered_perf_key_registry_covers_consumed_debug_stats_fields registered_perf_key_inventory_doc_is_in_sync --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Caveat

`cargo nextest run -p fret-bootstrap patch_latest_renderer_perf_sample_updates_latest_snapshot_stats`
without `--lib --features "launch ui-app-driver diagnostics"` still compiles package examples and
hits the existing `fn_driver_escape_hatch` feature-gating issue. Use the feature-qualified lib test
command above for this diagnostics-service gate.

# Open Threads

The next U7 cut can bridge boundary-owned chunk manifests into renderer input explicitly. Until then,
candidate telemetry should stay separate from real cache hit, dirty chunk, and upload-saved metrics.

# Citations

- [Plan](../../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md)
- [Boundary scene chunk manifest](2026-07-01-u7-boundary-scene-chunk-manifest.md)
