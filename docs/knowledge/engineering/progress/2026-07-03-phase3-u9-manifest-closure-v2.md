---
type: Work Progress
title: Phase 3 U9 manifest closure v2
tags: fret,phase3,u9,renderer,scene-chunks
timestamp: 2026-07-03
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
git_branch: feat/ui-framework-phase2-refactor
---

# Summary

Phase 3 U9 promoted `SceneChunkManifest` from an entry list into a renderer-facing assembly
contract without making it a second scene or a GPU resource store.

# Implemented

- `SceneChunkManifestEntry` now receives an explicit `order_index` when pushed into a manifest.
- `SceneChunkManifest` exposes aggregate draw streams, resource fingerprint, side-table
  requirements, and structured `assembly_unsupported_reasons()`.
- `SceneChunkClosureMetadata` exposes `SceneChunkSideTableRequirements`, separating balanced scope
  closure from relocation safety.
- `ChunkLaunchSupportMatrix` now treats resource-free quad and resource-free vertex-color manifests
  as supported frame classes, and blocks side-table/resource manifests with structured manifest
  unsupported reasons.
- `FrameAssembler` can assemble resource-free quad and vertex-color payloads. Vertex-color
  relocation covers `viewport_vertices`, `VertexColorDraw.first_vertex`, and `uniform_index`.
- `BoundarySceneChunkManifest` and canvas/code-editor scene fragment diagnostics now use
  bounds/origin-sensitive entry fingerprints instead of naked chunk fingerprints or XOR-only chunk
  summaries.

# Boundaries

Manifest closure records portable dependency facts only. It does not store GPU handles, atlas
tiles, bind groups, runtime upload ranges, UI state, or flat-scene debug oracles.

Clip, mask, effect, text, path, image, SVG, viewport-surface, material, custom-effect, and mixed
stream classes remain unsupported for authoritative chunk launch until side-table relocation and
resource closure are proven.

# Verification

- `cargo check -p fret-core --all-targets`
- `cargo check -p fret-render-wgpu --all-targets`
- `cargo check -p fret-render --all-targets`
- `cargo check -p fret-ui --all-targets`
- `cargo check -p fret-code-editor --all-targets`
- `cargo nextest run -p fret-core scene_chunk_manifest scene_chunk_closure --no-fail-fast`
- `cargo nextest run -p fret-render-wgpu source_selection scene_chunk_payload resource_free_vertex_color resource_free_quad --no-fail-fast`
- `cargo nextest run -p fret-ui canvas_scene_chunk_manifest --no-fail-fast`
- `cargo nextest run -p fret-code-editor row_scene_replay_plan_reports_scene_chunk_debug_metadata --no-fail-fast`

# Next

U10 can move normal launch to authoritative `ChunkManifest` only for supported resource-free frame
classes. It should not infer support for side-table streams from balanced scope closure.
