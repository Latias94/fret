---
type: Work Progress
title: Phase 2 U8 explicit render scene source
tags: fret,renderer,scene-chunks,phase2,u8
timestamp: 2026-07-02
related_plan: docs/plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md
git_branch: feat/ui-framework-phase2-refactor
---

# Summary

Phase 2 U8 now removes the ambiguous renderer input bridge where
`RenderSceneParams.scene_chunks: Option<_>` could be mistaken for a normal semantic source.
`RenderSceneParams` carries `RenderSceneSource` instead:

- `Flat { scene, diagnostic_chunks }` keeps launch/default callers on flat-scene semantics while
  preserving retained chunk diagnostics and payload warmup.
- `ResourceFreeQuadChunks { manifest, debug_scene }` is the authoritative chunk-native source for
  the first proven payload class: resource-free quad chunks.

Resource-free quad payloads can be assembled into a frame `SceneEncoding` by relocating quad
instance and uniform indices across cached chunk payloads. Unsupported authoritative chunks fail
instead of silently dropping draw content or falling back through an unprepared debug scene.

# Key Files

- `crates/fret-render-wgpu/src/renderer/mod.rs`
- `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs`
- `crates/fret-render-wgpu/src/renderer/render_scene/encoding_cache.rs`
- `crates/fret-render-wgpu/src/renderer/scene_chunk_encoding_cache.rs`
- `crates/fret-launch/src/runner/desktop/runner/window_redraw_render_scene.rs`
- `crates/fret-launch/src/runner/web/render_loop.rs`
- `crates/fret-render/src/lib.rs`

# Verification

- `cargo check -p fret-render-wgpu --tests`
- `cargo check -p fret-render --tests`
- `cargo check -p fret-launch --tests`
- `cargo nextest run -p fret-render-wgpu diagnostic_scene_chunk_manifest_does_not_override_flat_scene_encoding resource_free_quad_scene_chunk_manifest_uses_chunk_native_scene_encoding_key resource_free_quad_payloads_assemble_frame_encoding_with_relocated_indices scene_chunk_payload_and_resident_upload_state_warm_without_perf --no-fail-fast`
- `cargo nextest run -p fret-render-wgpu --no-fail-fast` (343 passed)

# Follow-Ups

- Prove additional authoritative chunk classes before allowing them through
  `ResourceFreeQuadChunks` or widening the source enum.
- Add targeted pixel parity for resource-free quad authoritative chunks against flat-scene output.
- Keep launch on `Flat::diagnostic_chunks` until UI can produce a full-frame authoritative manifest.
- Delete flat-scene default launch input only after text/SVG/image/path/mask/effect/material chunk
  resource closure and parity gates exist.

# Citations

- [Phase 2 plan](../../../plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md)
- [U8 flat scene source audit](../subagents/2026-07-02-phase2-u8-flat-scene-source-audit.md)
