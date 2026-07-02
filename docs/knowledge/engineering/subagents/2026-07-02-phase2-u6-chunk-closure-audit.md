---
type: Subagent Finding
title: Phase 2 U6 chunk closure audit
tags: fret,renderer,scene-chunk,phase2,subagent
timestamp: 2026-07-02
related_plan: docs/plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md
git_branch: feat/ui-framework-phase2-refactor
subagent_id: 019f2419-77ef-73f1-a934-fe7b898cfa7f
status: complete
---

# Finding

U6 was partially satisfied before this slice. The repo already had scene chunk input plumbing,
chunk encoding key/cache diagnostics, render-plan stream ranges, payload-plan alignment, and a
guarded quad resident upload proof. It did not have renderer-facing core closure metadata, and
chunk payload encoding still built a temporary flat `Scene` by replaying each chunk.

# Evidence

Existing partial coverage:

- `SceneChunk` and `SceneChunkManifestEntry` carried retained ops, text blob ids, local bounds,
  scene origin, and fingerprints.
- UI paint and native/web runners already passed chunk manifests into the renderer.
- `SceneChunkEncodingState` already keyed payloads by render context and per-entry text resource
  key.
- Render plans already exposed candidate segments and stream ranges.
- Payload-plan alignment already compared cached payload shape and stream fingerprints to flat
  render-plan segments.
- Quad resident upload already had a narrow safe-segment proof.

Remaining bridge evidence:

- `encode_scene_chunk_entry_payload` created a temporary `Scene`, called
  `replay_translated_into`, and encoded that flat scene.
- The normal renderer path still used `RenderSceneParams.scene` as the semantic source.
- Text chunk keys still used `text_resource_snapshot_for_blobs`.
- There was no dedicated external `crates/fret-render-wgpu/tests/scene_chunk_parity.rs`; most proof
  lived in renderer internals.

# Recommendation

Take a first U6 implementation slice that deletes the production replay bridge only for the safe
subset:

- add lightweight core closure metadata to `SceneChunk`,
- encode quad-only supported payloads directly from chunk ops with scene-origin translation,
- keep unsupported chunks explicit rather than replaying them,
- do not delete `RenderSceneParams.scene`, normal render planning from flat scene, or full-blob text
  resource helpers in U6.

# Disposition

Accepted for the first U6 slice. The implementation adds `SceneChunkClosureMetadata`, uses a shared
op-slice encoder for chunk payloads, encodes closure-supported quad payloads without temporary flat
scene replay, and keeps flat replay only in tests as a parity oracle.
