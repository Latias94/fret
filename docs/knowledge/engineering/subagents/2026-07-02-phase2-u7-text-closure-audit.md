---
type: Subagent Finding
title: Phase 2 U7 text resource closure audit
tags: fret,renderer,text,scene-chunk,phase2,subagent
timestamp: 2026-07-02
related_plan: docs/plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md
git_branch: feat/ui-framework-phase2-refactor
subagent_id: 019f244b-86fa-7e82-b662-f88e7d2c1850
status: complete
---

# Finding

U7's first safe slice is to remove the normal retained-chunk cache-key dependency on
`TextSystem::text_resource_snapshot_for_blobs`, not to claim full shaping-aware text closure.

Before this slice, the normal renderer chunk key path still derived each retained chunk's text
resource key from the chunk's full `TextBlobId` side index. Normal frame prepare already used
visible glyph residency, so the remaining mismatch was retained chunk cache invalidation.

# Evidence

- `text_resource_snapshot_for_blobs` was defined in
  `crates/fret-render-wgpu/src/text/diagnostics.rs`.
- Its normal renderer/cache caller was
  `crates/fret-render-wgpu/src/renderer/scene_chunk_encoding_cache.rs`.
- Reusable closure pieces already existed:
  `TextFrameResidency`, `TextSystem::push_glyph_residency_for_blob`,
  `TextSystem::text_resource_snapshot_for_residency`, and the scene-level
  `visible_text_residency_for_scene` prepass.
- The known gap is shaping-aware chunk safety: current visible residency tracks selected glyph
  keys and rects, but it does not yet retain glyph-to-cluster/run metadata for ligatures, RTL,
  combining marks, fallback-font edges, selection, caret, or decoration dependencies.

# Recommendation

Add a renderer-private chunk-entry visible text residency helper that reuses `VisibleTextState`,
initializes it with `SceneChunkManifestEntry::scene_origin()`, and uses
`text_resource_snapshot_for_residency` for retained chunk text keys. Keep
`text_resource_snapshot_for_blobs` test-only/debug-only and leave text chunk payload reassembly
unsupported until the shaping-aware closure gaps are closed.

# Disposition

Accepted for the first U7 slice. The implementation adds
`visible_text_residency_for_chunk_entry`, migrates retained chunk text keys to visible residency,
and keeps full-blob helpers out of normal renderer chunk/resource paths.
