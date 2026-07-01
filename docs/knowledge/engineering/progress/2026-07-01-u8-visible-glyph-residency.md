---
type: Work Progress
title: U8 visible glyph residency
tags: fret,u8,text,glyph-atlas,residency,renderer-cache,visible-glyphs
timestamp: 2026-07-01
related_plan: docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Summary

U8 now narrows text residency inside visible text blobs. The renderer visible-text prepass builds a
`TextFrameResidency` from glyph rectangles that intersect the current transform, opacity, viewport,
clip stack, and wgpu composite-group work scissor. Frame atlas prewarm and scene text resource
fingerprints consume that same residency, so offscreen suffix glyphs in a long visible blob no longer
enter the glyph atlas or the frame text resource key.

# Decisions

- Introduce `TextFrameResidency` as the renderer/text contract for frame-local glyph residency. It
  records blob id, selected glyph keys, and a glyph-subset fingerprint, while keeping `GlyphKey`
  private to the text module.
- Key the pin cache by `TextBlobId + selected glyph fingerprint` instead of only `TextBlobId`, so
  scrolling within one long text blob updates the retained pin set.
- Keep `text_resource_snapshot_for_blobs` as the full-blob compatibility helper. Runtime frame
  prepare now uses `text_resource_snapshot_for_residency`.

# Changed Files

- `crates/fret-render-wgpu/src/renderer/render_scene/frame_prepare.rs`
- `crates/fret-render-wgpu/src/renderer/render_scene/visible_text.rs`
- `crates/fret-render-wgpu/src/text/atlas_flow.rs`
- `crates/fret-render-wgpu/src/text/blobs.rs`
- `crates/fret-render-wgpu/src/text/diagnostics.rs`
- `crates/fret-render-wgpu/src/text/mod.rs`
- `crates/fret-render-wgpu/src/text/pin_state.rs`

# Verification

- `cargo check -p fret-render-wgpu --all-targets`
- `cargo nextest run -p fret-render-wgpu visible_text_blob_prepass_excludes_offscreen_residency visible_text_glyph_residency_excludes_offscreen_suffix_glyphs visible_text_blob_prepass_respects_clip_and_opacity visible_text_blob_prepass_does_not_treat_effect_bounds_as_clip visible_text_blob_prepass_matches_composite_group_work_scissor prepare_for_scene_pin_cache_removes_replaced_or_missing_blobs prepare_for_scene_reuses_unchanged_ring_bucket_signature prepare_for_scene_diffs_mutated_ring_bucket_incrementally --no-fail-fast`
- `cargo nextest run -p fret-render-wgpu --no-fail-fast`
- `cargo check -p fret-render-wgpu --target wasm32-unknown-unknown --lib`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/perf/audit_perf_baselines.py --matrix docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md --strict`
- `git diff --check`

# Next Action

Continue U8 by aligning editor-like text surfaces around line or paragraph blob identity and by
adding/refreshing text-heavy diagnostics that prove local edits avoid whole-buffer text invalidation.
