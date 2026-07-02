---
type: Work Progress
title: Phase 2 U6 chunk closure native payload
tags: fret,renderer,scene-chunk,phase2,ce-work
timestamp: 2026-07-02
related_plan: docs/plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md
git_branch: feat/ui-framework-phase2-refactor
status: complete
---

# Summary

Phase 2 U6 now has a first chunk-closure implementation slice:

- `SceneChunk` carries core closure metadata for op range, balanced/open/inherited scope state,
  draw-stream summaries, resource references, resource fingerprint, and closure fingerprint.
- `SceneChunkManifestEntry::fingerprint()` includes the chunk closure fingerprint.
- Renderer chunk payload encoding no longer creates a temporary flat `Scene` or calls chunk replay
  in the production payload path.
- Closure-supported quad payloads encode directly from the chunk op slice through the shared render
  scene encoder with an initial scene-origin transform.
- Unsupported/open-scope chunks produce an empty chunk payload instead of a hidden replay payload.

This is intentionally not U8: normal renderer input still uses flat `Scene`, and parity tests still
use flat replay as the oracle. It is also not U7: chunk text resource keys still depend on the
full-blob helper until visible glyph residency closure lands.

# Verification

Passed:

- `cargo check -p fret-core --tests`
- `cargo check -p fret-render-wgpu --tests`
- `cargo nextest run -p fret-core scene_chunk --no-fail-fast`
- `cargo nextest run -p fret-core --no-fail-fast` (108 passed)
- `cargo nextest run -p fret-render-wgpu scene_chunk_encoding_cache --no-fail-fast`
- `cargo nextest run -p fret-render-wgpu scene_chunk_manifest_is_reported_without_busting_scene_encoding_cache unreferenced_text_atlas_churn_does_not_bust_scene_or_chunk_encoding_cache scene_chunk_payload_and_resident_upload_state_warm_without_perf --no-fail-fast`
- `cargo nextest run -p fret-render-wgpu --no-fail-fast` (340 passed)
- `cargo fmt --all`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 /Users/frankorz/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`

Static search in `crates/fret-render-wgpu/src/renderer/scene_chunk_encoding_cache.rs` shows
`Scene::default()` and `replay_translated_into` only in parity tests, not in the production payload
builder.

# Remaining Edge

This slice proves the first native chunk payload class and removes the production replay bridge
there. U6 still has broader parity work if this plan keeps U6 as one unit: text, masks, paths,
materials, effects, relocated side-table/resource-key equivalence, render-plan command parity, and
targeted pixel/golden tests. U7 must replace full-blob text resource keys with chunk-local visible
glyph residency before text chunks can become a supported normal payload class. U8 owns retiring
flat `Scene` from normal renderer input.

# Citations

- [Phase 2 plan](../../../plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md)
- [U6 audit](../subagents/2026-07-02-phase2-u6-chunk-closure-audit.md)
- `crates/fret-core/src/scene/chunk.rs`
- `crates/fret-core/src/scene/manifest.rs`
- `crates/fret-render-wgpu/src/renderer/scene_chunk_encoding_cache.rs`
- `crates/fret-render-wgpu/src/renderer/render_scene/encode/mod.rs`
- `crates/fret-render-wgpu/src/renderer/render_scene/encode/state.rs`
- `docs/adr/0327-frame-pipeline-v2-and-view-boundaries.md`
