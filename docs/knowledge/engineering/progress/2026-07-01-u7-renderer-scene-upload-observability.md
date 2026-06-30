---
type: Work Progress
title: U7 renderer scene/upload observability
tags: fret,ui,renderer,diagnostics,u7
timestamp: 2026-07-01
related_plan: docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
source_session: 019f143b-4f62-7333-a9b1-c3c54cf1409e
---

# Summary

U7's first implementation slice chose an attribution floor before retained scene chunks or dirty GPU range writes.
The slice exposes geometry upload bytes/write counts per renderer stream and a scene-encoding cache miss-reason histogram through renderer perf snapshots, bootstrap UI diagnostics, `fret-diag` stats rows, `diag perf` JSON rows, repeat summaries, and the frame-stats perf-key registry.

# Decision

Do not start U7 by changing `SceneRecording`, `Scene::swap_storage`, render-plan compilation, `PreviousFramePaintRecording`, or partial upload layout.
Those paths remain the follow-on retained scene chunk / dirty upload migration.

The completed slice is intentionally observability-only:

- Geometry upload streams report bytes and write counts for quad instances, path paints, text paints, viewport vertices, text glyph instances, text vertices, and path vertices.
- Scene encoding cache misses are accumulated by reason bits instead of only exposing the last miss mask.
- Existing flat scene compatibility and full stream writes remain intact.

# Verified State

Relevant checks passed for this slice:

- `cargo check -p fret-render-wgpu --all-targets`
- `cargo check -p fret-diag --all-targets`
- `cargo check -p fret-bootstrap --lib --tests`
- `cargo nextest run -p fret-render-wgpu record_scene_encoding_cache_frame_result_updates_perf_counters --no-fail-fast`
- `cargo nextest run -p fret-diag perf_json_row_exports_top_code_editor_row_scene_fields perf_repeat_run_json_row_exports_top_code_editor_row_scene_fields perf_repeat_summary_json_row_summarizes_code_editor_row_scene_fields --no-fail-fast`
- `cargo nextest run -p fret-diag registered_perf_keys_are_unique registered_perf_key_units_match_names registered_perf_key_contract_keeps_stats_and_gate_keys_additive full_registered_perf_key_registry_covers_consumed_debug_stats_fields registered_perf_key_inventory_doc_is_in_sync --no-fail-fast`
- `cargo nextest run -p fret-bootstrap --lib --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/perf/audit_perf_baselines.py --matrix docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md --strict`
- `git diff --check`

# Open Threads

This slice does not implement retained scene chunk identity, chunk dirty counts, chunk encode hit rates, dirty GPU range uploads, text/glyph cache budgets, or GPU timestamp attribution.
Those remain the U7/U8 migration targets now that the diagnostic payload can explain upload and scene-cache pressure.

# Citations

- [Plan](../../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md)
- [Perf contract matrix](../../../workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md)
- [ADR implementation alignment](../../../adr/IMPLEMENTATION_ALIGNMENT.md)
- Explorer `019f1a3d-5f31-7211-8493-1920832b8c49`
- Explorer `019f1a58-1ce3-74d1-a1ee-3337a1b385f8`
