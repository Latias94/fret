---
type: Work Progress
title: U7 boundary scene chunk diagnostics
tags: fret,ui,scene,diagnostics,u7
timestamp: 2026-07-01
related_plan: docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
source_session: 019f143b-4f62-7333-a9b1-c3c54cf1409e
---

# Summary

U7's third implementation slice exposes retained scene chunk count and fingerprint metadata through
boundary-owned scene fragment diagnostics. The runtime still paints through the flat `Scene`
compatibility bridge, but diagnostics can now tell whether a boundary fragment contains retained
chunks and whether the chunk identity stayed stable across frames.

# Decision

Keep this slice diagnostic-only:

- `BoundarySceneFragmentDebug` now reports entry count, chunk count, and a diagnostic fingerprint.
- `BoundaryFrameProducts::scene_fragment` stores those counts in typed-output metadata.
- `CanvasSceneFragment<T>` forwards its `SceneChunk` count/fingerprint while the payload still owns
  entry count.
- Code-editor `RowSceneReplayPlan` reports row-scene chunk count and an aggregate fingerprint that
  includes row identity for non-empty retained chunks.
- `debug.boundaries[]` and `UiBoundaryDiagnosticsV1` expose `scene_fragment_chunks` and
  `scene_fragment_fingerprint`.

This does not add renderer chunk encoding, render-plan reuse, or dirty GPU range uploads yet.

# Verified State

Relevant checks passed:

- `cargo check -p fret-ui --all-targets`
- `cargo check -p fret-code-editor --all-targets`
- `cargo nextest run -p fret-ui canvas_scene_fragment_is_boundary_owned_and_keyed_by_prepaint_key canvas_prepaint_can_prepare_text_scene_fragment_before_paint --no-fail-fast`
- `cargo nextest run -p fret-code-editor row_scene_replay_plan_reports_scene_chunk_debug_metadata row_scene_replay_plan_rejects_stale_frame_and_skipped_rows --no-fail-fast`
- `cargo nextest run -p fret-bootstrap --lib --no-fail-fast`
- `cargo nextest run -p fret-diag bundle_stats_preserves_cache_root_boundary_summary --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/perf/audit_perf_baselines.py --matrix docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md --strict`
- `git diff --check`

Note: `cargo check -p fret-bootstrap --all-targets` still fails on the existing
`fn_driver_escape_hatch` example unless launch-related features/deps are enabled. This slice used
`cargo nextest run -p fret-bootstrap --lib --no-fail-fast` for the touched diagnostics code.

# Open Threads

Next U7 work should decide how chunk identity reaches renderer encoding:

- either add `SceneRecording` sidecar chunk ranges or let boundary products publish chunk lists,
- account for ambient transform, clip, mask, and effect stack before independently encoding chunks,
- map chunk fingerprints plus resource generations to scene encoding / render-plan reuse,
- only then replace full stream uploads with chunk-derived dirty GPU range writes.

# Citations

- [Plan](../../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md)
- [Scene chunk compatibility bridge](2026-07-01-u7-scene-chunk-compatibility-bridge.md)
- [Frame pipeline scene fragment carrier](../../../workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/M3C_BOUNDARY_SCENE_FRAGMENT_CARRIER_SLICE_2026-05-14.md)
