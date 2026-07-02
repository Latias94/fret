---
type: Work Progress
title: Phase 2 U9 VertexColor viewport partial upload
tags: fret,phase2,u9,renderer,partial-upload
timestamp: 2026-07-02
related_plan: docs/plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md
git_branch: feat/ui-framework-phase2-refactor
---

# Phase 2 U9 VertexColor Viewport Partial Upload

## Summary

Phase 2 U9 opens the first non-quad resident geometry partial upload slice, but only for
resource-free `VertexColor` chunks that write `viewport_vertices`.

The slice keeps `Image`, `ViewportSurface`, text, path, mask, material, clip, and effect-dependent
streams on full upload until their resource or side-table closure metadata is explicit.

## Changes

- `SceneChunkDrawStreamSummary` now exposes `is_vertex_color_only()`.
- `SceneChunkClosureMetadata` now exposes `is_resource_free_vertex_color_only()`.
- Chunk payload encoding can build resource-free vertex-color payloads directly from chunk ops.
- Payload-plan reassembly no longer treats `OrderedDraw::VertexColor` as an automatic non-quad
  blocker, but the existing side-table/material blockers still apply.
- Resident geometry upload planning now produces a `viewport_vertices` write plan only for segments
  with `has_vertex_color` and without image, viewport-surface, text, path, or mask flags.
- The actual upload path consumes that plan and writes only dirty viewport vertex ranges after
  warmup; coverage gaps and unsupported flags still fall back to full upload.

## Verification

Focused red/green evidence:

- Before production wiring, `resident_viewport_partial_upload_writes_only_changed_range` failed with
  a full 2-vertex upload (`80` bytes) instead of the expected one-vertex upload (`40` bytes).
- After implementation, the focused U9 test set passed:
  `cargo nextest run -p fret-render-wgpu resident_viewport_partial_upload_writes_only_changed_range resident_viewport_partial_upload_skips_stable_slot_after_warmup resident_viewport_partial_upload_blocks_incomplete_stream_coverage resident_viewport_partial_upload_blocks_image_and_viewport_surface_flags vertex_color_chunk_payload_alignment_allows_viewport_vertex_reassembly image_and_viewport_surface_payloads_remain_blocked_until_resource_closure --no-fail-fast`.

Broader verification passed before commit:

- `cargo check -p fret-core --tests`
- `cargo check -p fret-render-wgpu --tests`
- `cargo nextest run -p fret-render-wgpu --no-fail-fast` (349 passed)
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/perf/audit_perf_baselines.py --matrix docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md --strict`
- `python3 /Users/frankorz/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`

## Next Action

Continue only to the next stream class whose closure owner and fallback proof are explicit.

# Citations

- [Phase 2 plan](../../../plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md)
- [Geometry upload implementation](../../../../crates/fret-render-wgpu/src/renderer/geometry_upload.rs)
- [Scene chunk encoding cache](../../../../crates/fret-render-wgpu/src/renderer/scene_chunk_encoding_cache.rs)
- [Chunk closure metadata](../../../../crates/fret-core/src/scene/chunk.rs)
