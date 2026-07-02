---
type: Work Progress
title: Phase 2 U7 chunk-local text resource closure
tags: fret,renderer,text,scene-chunk,phase2,ce-work
timestamp: 2026-07-02
related_plan: docs/plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md
git_branch: feat/ui-framework-phase2-refactor
status: complete
---

# Summary

Phase 2 U7 now has a first text-resource-closure slice:

- Retained chunk text resource keys no longer use the full-blob helper on the normal renderer path.
- `visible_text_residency_for_chunk_entry` reuses the scene visible-text prepass state with the
  chunk entry's scene-origin translation.
- Chunk keys now use `TextSystem::text_resource_snapshot_for_residency` over the chunk-local
  visible glyph residency.
- `text_resource_snapshot_for_blobs` and `text_residency_for_blobs` are hidden behind `#[cfg(test)]`
  after their normal-path caller was deleted.

# Red/Green Evidence

Added `text_chunk_key_ignores_offscreen_suffix_glyph_residency` before implementation. It failed
because the old full-blob key missed the second-frame chunk cache after offscreen suffix glyphs
became resident:

- expected `scene_chunk_encoding_key_cache_hits == 1`
- observed `scene_chunk_encoding_key_cache_hits == 0`

After the implementation, the same test passes.

# Verification

Passed:

- `cargo nextest run -p fret-render-wgpu text_chunk_key_ignores_offscreen_suffix_glyph_residency --no-fail-fast`
- `cargo nextest run -p fret-render-wgpu text_chunk_key_ignores_offscreen_suffix_glyph_residency visible_text_glyph_residency_excludes_offscreen_suffix_glyphs unreferenced_text_atlas_churn_does_not_bust_scene_or_chunk_encoding_cache scene_chunk_payload_and_resident_upload_state_warm_without_perf --no-fail-fast`
- `cargo nextest run -p fret-render-wgpu text_chunk_key_ignores_offscreen_suffix_glyph_residency scene_chunk_encoding_cache --no-fail-fast`
- `cargo nextest run -p fret-render-wgpu --no-fail-fast` (341 passed)
- `cargo check -p fret-render-wgpu --tests`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 /Users/frankorz/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `python3 tools/perf/diag_u8_text_budget_gate.py --help`
- `git diff --check`

Static renderer search, excluding test assertions, has no normal-path
`text_resource_snapshot_for_blobs` calls.

# Remaining Edge

This slice does not make text chunks a supported chunk-native payload class. Current visible
residency is glyph-key based and does not yet carry shaping-run or cluster metadata. Ligatures,
RTL, combining marks, fallback-font changes, decorations, selection, and caret dependencies still
need explicit closure/fallback gates before text payload reassembly or non-quad partial uploads can
use them.

# Citations

- [Phase 2 plan](../../../plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md)
- [U7 audit](../subagents/2026-07-02-phase2-u7-text-closure-audit.md)
- `crates/fret-render-wgpu/src/renderer/render_scene/visible_text.rs`
- `crates/fret-render-wgpu/src/renderer/scene_chunk_encoding_cache.rs`
- `crates/fret-render-wgpu/src/text/diagnostics.rs`
- `crates/fret-render-wgpu/src/text/blobs.rs`
