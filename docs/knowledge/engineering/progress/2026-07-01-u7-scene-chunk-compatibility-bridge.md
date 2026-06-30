---
type: Work Progress
title: U7 scene chunk compatibility bridge
tags: fret,ui,scene,renderer,u7
timestamp: 2026-07-01
related_plan: docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
source_session: 019f143b-4f62-7333-a9b1-c3c54cf1409e
---

# Summary

U7's second implementation slice adds a portable retained scene chunk carrier without changing renderer behavior.
`SceneChunk` lives in `fret-core::scene` and carries retained ops, the matching text-blob side index, and a chunk fingerprint.
It can replay into the existing flat `SceneRecording` path, including translated replay, so current renderers continue to consume flat `Scene`.

# Decision

Start with a core compatibility bridge instead of dirty GPU range upload.
Dirty upload needs stable chunk identity, chunk-to-encoded-stream ranges, and a buffer strategy that can preserve clean ranges; the current renderer still encodes a flat scene and uploads full streams.

This slice intentionally does not:

- add chunk sidecar ranges to `SceneRecording`,
- change `RenderSceneParams`,
- reinterpret `RenderPlanSegment` as input chunk identity,
- change `GeometryUploadState` upload strategy,
- add renderer chunk encode cache semantics.

# Verified State

Relevant checks passed:

- `cargo check -p fret-core --all-targets`
- `cargo check -p fret-ui --all-targets`
- `cargo check -p fret-code-editor --all-targets`
- `cargo check -p fret-code-editor --features syntax-rust --all-targets`
- `cargo nextest run -p fret-core scene_chunk_replay_matches_flat_replay_and_keeps_text_blob_index scene_chunk_translated_replay_wraps_ops_and_keeps_chunk_fingerprint_stable --no-fail-fast`
- `cargo nextest run -p fret-ui canvas_prepaint_can_prepare_text_scene_fragment_before_paint canvas_scene_fragment_is_boundary_owned_and_keyed_by_prepaint_key --no-fail-fast`
- `cargo nextest run -p fret-code-editor retained_row_scene_origin_preserves_bounds_offset row_scene_replay_plan_rejects_stale_frame_and_skipped_rows paint_frame_cache_min_entries_tracks_visible_window_union --no-fail-fast`
- `cargo nextest run -p fret-code-editor --features syntax-rust prepaint_row_scene_replay_plan_moves_row_text_work_out_of_paint prepaint_row_scene_replay_plan_reuses_stable_window_plan prepaint_row_scene_replay_plan_reuses_cached_non_preedit_rows_during_preedit prepaint_row_scene_replay_plan_handles_plain_cached_rows prepaint_row_scene_replay_plan_uses_cached_syntax_replay_context --no-fail-fast`
- `cargo fmt --all --check`

# Open Threads

Next U7 slices should add explicit chunk identity / metadata before renderer reuse:

- decide whether `SceneRecording` gets sidecar chunk ranges or whether boundary products publish chunk lists separately,
- include ambient transform/clip/mask/effect stack facts before independently encoding chunks,
- map chunk fingerprints to scene encoding/render-plan reuse,
- only then implement chunk-derived dirty GPU upload ranges.

# Citations

- [Plan](../../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md)
- [ADR implementation alignment](../../../adr/IMPLEMENTATION_ALIGNMENT.md)
- [Perf contract matrix](../../../workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md)
- Explorer `019f1a82-3bd0-7c41-99cd-2815e36df64c`
