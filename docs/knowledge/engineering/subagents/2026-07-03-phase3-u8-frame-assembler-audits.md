---
type: Subagent Finding
title: Phase 3 U8 FrameAssembler and renderer source audits
tags: fret,phase3,u8,renderer,frame-assembler,subagent
timestamp: 2026-07-03
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
subagents:
  - 019f283a-52fe-7291-92e0-fac192bb6940
  - 019f283a-95af-7750-a62d-793da264e1dd
---

# Finding

Two read-only explorers converged on the same U8 boundary:

- U8 should do type-level source separation, not a launch behavior switch.
- Normal native/web launch can remain flat-rendered for now, but launch must call a shared source
  selection helper and stop constructing a mixed flat-plus-diagnostic source.
- Renderer-owned chunk payload/cache assembly should move behind a `FrameAssembler` owner, while
  actual GPU upload and stream-policy expansion remain in existing upload code until U11.
- Zed/GPUI has no one-to-one `FrameAssembler` type. The useful prior art is the separation between
  current authoritative frame products (`rendered_frame`/`next_frame`), retained replay/cache
  products, and renderer source/debug outputs.

# Evidence

Codebase audit anchors:

- `crates/fret-render-wgpu/src/renderer/mod.rs`
- `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs`
- `crates/fret-render-wgpu/src/renderer/scene_chunk_encoding_cache.rs`
- `crates/fret-launch/src/runner/desktop/runner/window_redraw_render_scene.rs`
- `crates/fret-launch/src/runner/web/render_loop.rs`

Reference audit anchors:

- `repo-ref/zed/crates/gpui/src/window.rs`
- `repo-ref/zed/crates/gpui/src/scene.rs`
- `repo-ref/zed/crates/gpui/src/view.rs`
- `repo-ref/zed/crates/gpui/src/platform.rs`

# Recommendation

Implement U8 as:

- `RenderSceneSourceSelection` plus explicit `RenderSceneSource::{FlatCompat, ChunkManifest}`.
- `select_render_scene_source(scene, manifest, policy)` shared by native and web launch.
- `ChunkLaunchSupportMatrix` that supports only resource-free quad authoritative chunk launch in
  U8 and reports structured unsupported reasons for everything else.
- `FrameAssembler` as renderer-internal owner for scene chunk payload/cache assembly state.

# Disposition

Implemented in the U8 slice. U9 remains responsible for manifest closure V2 and side-table
relocation. U10 remains responsible for moving normal launch to authoritative chunk manifests.
U11 remains responsible for expanding partial-upload stream policy.
