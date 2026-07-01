---
type: Work Progress
title: U8 visible text blob residency prepass
tags: fret,u8,text,glyph-atlas,residency,renderer-cache,visible-prepass
timestamp: 2026-07-01
related_plan: docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Summary

U8 now narrows frame text residency from all `Scene::text_blob_ids()` to the text blobs that can
contribute to the current viewport. Renderer frame prepare runs a visible-text prepass before atlas
prewarm, and scene text resource fingerprints are now based on that visible blob list rather than
the whole scene text side index.

# Decisions

- Rename the text pin cache from scene-specific ownership to text-blob residency ownership.
  `Scene` remains a test compatibility bridge, while runtime frame prepare passes the visible blob
  sequence directly.
- Keep the first prepass at blob granularity. It reproduces transform, opacity, clip, clip-path
  bounds, and current wgpu composite-group work scissor; it deliberately ignores mask/effect/backdrop
  bounds because those are not text draw scissors in encode.
- Use shape glyph bounds for visibility and avoid atlas UV lookup during the prepass. This keeps
  visibility collection independent from the residency work it is deciding.

# Changed Files

- `crates/fret-render-wgpu/src/renderer/render_scene/visible_text.rs`
- `crates/fret-render-wgpu/src/renderer/render_scene/frame_prepare.rs`
- `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs`
- `crates/fret-render-wgpu/src/renderer/render_scene/mod.rs`
- `crates/fret-render-wgpu/src/text/atlas_flow.rs`
- `crates/fret-render-wgpu/src/text/blobs.rs`
- `crates/fret-render-wgpu/src/text/diagnostics.rs`
- `crates/fret-render-wgpu/src/text/pin_state.rs`

# Verification

- `cargo check -p fret-render-wgpu --all-targets`
- `cargo nextest run -p fret-render-wgpu visible_text_blob_prepass_excludes_offscreen_residency visible_text_blob_prepass_respects_clip_and_opacity visible_text_blob_prepass_does_not_treat_effect_bounds_as_clip visible_text_blob_prepass_matches_composite_group_work_scissor prepare_for_scene_pin_cache_removes_replaced_or_missing_blobs prepare_for_scene_reuses_unchanged_ring_bucket_signature prepare_for_scene_diffs_mutated_ring_bucket_incrementally --no-fail-fast`
- `cargo nextest run -p fret-render-wgpu --no-fail-fast`
- `cargo check -p fret-render-wgpu --target wasm32-unknown-unknown --lib`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/perf/audit_perf_baselines.py --matrix docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md --strict`
- `git diff --check`

# Next Action

Continue U8 by moving from visible-blob residency to visible glyph or line residency so long offscreen
suffixes within an otherwise visible text blob stop entering the atlas/resource fingerprint.
