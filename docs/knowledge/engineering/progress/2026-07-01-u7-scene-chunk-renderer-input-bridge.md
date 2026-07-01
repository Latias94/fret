---
type: Work Progress
title: U7 scene chunk renderer input bridge
tags: fret,u7,scene-chunks,renderer,diagnostics
timestamp: 2026-07-01
related_plan: ../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Summary

U7 now has an explicit portable bridge from UI boundary-owned retained scene chunks to renderer
input diagnostics.

The bridge deliberately uses `fret_core::SceneChunkManifest` plus
`RenderSceneParams::scene_chunks: Option<&SceneChunkManifest>`. It does not put retained chunk
identity into `SceneRecording`, does not make `fret-render-wgpu` depend on `fret-ui`, and does not
change scene encoding cache keys, geometry upload ranges, or render output semantics.

# Implementation

- `fret-core::scene` owns `SceneChunkManifest` and `SceneChunkManifestEntry`, alongside
  `SceneChunk`.
- `BoundarySceneChunkManifest` can append into the portable core manifest.
- `UiTree` publishes a frame-local `last_paint_scene_chunk_manifest` after painting visible roots.
  The manifest is cleared/replaced every paint pass and filters out stale, non-visible boundaries.
- `fret-launch` stores a `SceneChunkManifest` next to each runner-owned `Scene` and passes it to
  `RenderSceneParams` on desktop and web.
- `WinitAppDriver` / `FnDriverHooks` gained an additive `scene_chunk_manifest` hook. The bootstrap
  UI app driver returns `state.ui.scene_chunk_manifest()`.
- `fret-render-wgpu` records evidence-only `scene_chunk_input_chunks`,
  `scene_chunk_input_ops`, and `scene_chunk_input_fingerprint` perf counters. These are input
  visibility counters, not reuse/hit/dirty-upload counters.
- Bootstrap diagnostics and `fret-diag` perf-key registry expose the renderer input counters.

# Verification

Passed:

- `cargo check -p fret-core --all-targets`
- `cargo check -p fret-ui --all-targets`
- `cargo check -p fret-render-wgpu --all-targets`
- `cargo check -p fret-render --all-targets`
- `cargo check -p fret-launch --all-targets`
- `cargo check -p fret-bootstrap --lib --tests`
- `cargo check -p fret-diag --all-targets`
- `cargo check -p fret-clip-mask-stress -p fret-svg-atlas-stress -p fret-ui-gallery -p fret-quad-material-stress --all-targets`
- `cargo nextest run -p fret-core scene_chunk_manifest_skips_empty_chunks_and_reports_ops_and_fingerprint --no-fail-fast`
- `cargo nextest run -p fret-ui canvas_prepaint_can_prepare_text_scene_fragment_before_paint canvas_scene_chunk_manifest_is_frame_local_and_clears_when_no_chunk_is_painted --no-fail-fast`
- `cargo nextest run -p fret-render-wgpu scene_chunk_manifest_is_reported_without_busting_scene_encoding_cache --no-fail-fast`
- `cargo nextest run -p fret-launch fn_driver_forwards_scene_chunk_manifest_hook --no-fail-fast`
- `cargo nextest run -p fret-bootstrap --lib --features "launch ui-app-driver diagnostics" ui_diagnostics::service_tests::patch_latest_renderer_perf_sample_updates_latest_snapshot_stats --no-fail-fast`
- `cargo nextest run -p fret-diag registered_perf_keys_are_unique registered_perf_key_units_match_names registered_perf_key_contract_keeps_stats_and_gate_keys_additive full_registered_perf_key_registry_covers_consumed_debug_stats_fields registered_perf_key_inventory_doc_is_in_sync --no-fail-fast`
- `cargo nextest run -p fret-render --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Next

Next U7 work should design actual renderer chunk encode/cache ownership against this input bridge.
Do not rename the new `scene_chunk_input_*` counters to hit/reuse/dirty/upload metrics until a real
cache and dirty upload implementation exists.

# Citations

- [Plan](../../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md)
- Subagent `019f1b08-2a9f-72d1-958e-539ec31c5e9b`
- `crates/fret-core/src/scene/manifest.rs`
- `crates/fret-ui/src/tree/paint/entry.rs`
- `crates/fret-launch/src/runner/desktop/runner/window_redraw.rs`
- `crates/fret-launch/src/runner/web/render_loop.rs`
- `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs`
