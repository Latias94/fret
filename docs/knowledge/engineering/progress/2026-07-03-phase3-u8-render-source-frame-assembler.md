---
type: Work Progress
title: Phase 3 U8 renderer source split and FrameAssembler
tags: fret,phase3,u8,renderer,frame-assembler,source-selection
timestamp: 2026-07-03
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
related_subagents:
  - ../subagents/2026-07-03-phase3-u8-frame-assembler-audits.md
---

# Summary

This slice splits renderer source selection so flat-scene semantics no longer share a type variant
with diagnostic chunk manifests, and introduces a renderer-internal `FrameAssembler` owner for
chunk payload/cache assembly state.

Changes:

- `RenderSceneParams::source` now takes `RenderSceneSourceSelection`.
- `RenderSceneSource` now tells the truth: `FlatCompat { scene }` or
  `ChunkManifest { manifest, debug_scene }`.
- `select_render_scene_source(scene, manifest, policy)` is the shared source-selection contract
  used by native and web launch. U8 keeps normal launch on `FlatCompat` while preserving the
  manifest as an assembly sidecar for perf/cache evidence.
- `ChunkLaunchSupportMatrix` reports structured support. U8 supports resource-free quad
  authoritative chunk launch and deliberately does not promote vertex-color or mixed streams.
- `FrameAssembler` now owns `SceneChunkEncodingState` and mediates resource-free quad assembly,
  payload cache warmup, and payload-plan alignment.
- Renderer tests, integration helpers, stress apps, and gallery stack-overflow harnesses now use
  `RenderSceneSourceSelection::flat_compat` or explicit `chunk_manifest` construction.
- Runtime contract and closure-map docs now point to the U8 implementation anchors.

# Verification

Passed:

- `cargo check -p fret-render-wgpu --all-targets`
- `cargo check -p fret-render --all-targets`
- `cargo check -p fret-launch --all-targets`
- `cargo check -p fret-ui-gallery --all-targets`
- `cargo check -p fret-clip-mask-stress --all-targets`
- `cargo check -p fret-svg-atlas-stress --all-targets`
- `cargo check -p fret-quad-material-stress --all-targets`
- `cargo nextest run -p fret-render-wgpu render_scene scene_chunk source_selection --no-fail-fast`
  - Result: 27 passed, 327 skipped.
- Static search over `crates`, `ecosystem`, `apps`, and `tools` found no code matches for
  `flat_with_diagnostic_chunks`, `RenderSceneSource::flat(`,
  `RenderSceneSource::resource_free_quad_chunks`, `ResourceFreeQuadChunks`, or
  `RenderSceneSource::Flat`.
- `cargo fmt --all --check`
- `git diff --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_adr_numbers.py`
- `python3 tools/check_workstream_catalog.py`

# Deletion Gate

The U8 deletion gate is satisfied for the source type: no code path can construct the old mixed
flat-with-diagnostic-chunks source. Normal launch still uses `FlatCompat` by explicit policy, which
is intentional U8 behavior and remains the U10 deletion target.

# Next

Proceed to U9:

- Promote `SceneChunkManifest` closure metadata into an assembly contract.
- Add side-table relocation coverage and structured unsupported reasons for authoritative mode.
- Keep launch on the U8 source-selection helper until U10 moves supported fixtures to chunk-native
  launch.
