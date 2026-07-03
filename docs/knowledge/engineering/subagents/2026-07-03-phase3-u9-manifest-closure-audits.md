---
type: Subagent Finding
title: Phase 3 U9 manifest closure audits
tags: fret,phase3,u9,subagent,renderer,gpui
timestamp: 2026-07-03
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
subagent_ids:
  - 019f2861-bcfc-7891-af9a-5ebf8c643094
  - 019f2861-bd3e-7c82-9d1f-2e539020411a
---

# Finding

Two read-only audits converged on the same U9 boundary: `SceneChunkManifest` should describe
portable assembly eligibility and dependency closure, not become a second scene or a backend
resource store.

# Fret Audit

The Fret audit found that current `SceneEncoding` has multiple stream tables plus side tables:
quad instances, viewport vertices, text/path paints, glyph/path vertices, uniforms,
`uniform_mask_images`, clip/mask/effect tables, and ordered draws. The only safe append-only
relocation before U9 was resource-free quad. Vertex-color could be promoted if relocation rebased
`viewport_vertices`, `VertexColorDraw.first_vertex`, and `uniform_index`.

The audit recommended keeping text, path, image, SVG, viewport surface, clip, mask, effect,
material, custom-effect, and mixed streams as structured unsupported until resource closure and
side-table relocation are explicit.

# GPUI/Zed Audit

The GPUI/Zed audit found no equivalent to `SceneChunkManifest + side-table relocation`. GPUI keeps
one authoritative per-frame `Scene`; cache hits replay ranges into the next frame, and the platform
backend receives only `draw(&Scene)`. Cross-frame view state, dispatch trees, text layout, atlas
resources, and GPU handles stay outside the scene authority.

# Disposition

U9 follows that guidance:

- `FlatCompat` and `ChunkManifest` remain explicit source choices.
- `FrameAssembler` owns chunk assembly and relocation proof.
- Manifest closure stores only portable order, bounds/origin, stream, resource, and side-table
  requirement facts.
- Unsupported streams stay outside chunk-native launch evidence.
