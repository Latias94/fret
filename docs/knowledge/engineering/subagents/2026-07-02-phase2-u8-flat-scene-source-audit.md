---
type: Subagent Finding
title: Phase 2 U8 flat scene source audit
tags: fret,renderer,scene-chunks,phase2,u8,subagent
timestamp: 2026-07-02
related_plan: docs/plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md
subagent_id: 019f2465-f702-74f2-bfc1-93dfcfd7a7b9
---

# Finding

The renderer still had real semantic dependencies on flat `Scene`: debug validation, text frame
prepare, SVG prepare, scene encoding cache keys, and CPU scene encoding all consumed it. Render-plan
compile and upload paths were already downstream of `SceneEncoding` / `RenderPlan`.

The audit also found a critical contract hazard: first-party launch callers pass retained
`SceneChunkManifest` values for diagnostics and warmup, not as full-frame authoritative render
sources. Treating any `scene_chunks: Some(_)` as authoritative would have made empty or partial
diagnostic manifests able to override the flat scene.

# Recommendation

Introduce an explicit source enum:

- flat scene plus diagnostic chunks for existing launch/default callers;
- authoritative chunk source only for a proven chunk class.

Start with a resource-free quad authoritative source. Fail unsupported authoritative chunks rather
than silently dropping draw streams. Keep flat scene as debug/parity evidence until broader chunk
closure gates exist.

# Disposition

Implemented in the U8 explicit render scene source slice. The final design uses
`RenderSceneSource::Flat` for launch diagnostic manifests and
`RenderSceneSource::ResourceFreeQuadChunks` for the first chunk-native frame assembly path.

# Citations

- [Phase 2 U8 progress](../progress/2026-07-02-phase2-u8-explicit-render-scene-source.md)
- `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs`
- `crates/fret-launch/src/runner/desktop/runner/window_redraw_render_scene.rs`
- `crates/fret-launch/src/runner/web/render_loop.rs`
