---
type: Work Progress
title: U8 editor line text identity
tags: fret,u8,text,code-editor,line-identity,row-cache,scene-chunks
timestamp: 2026-07-01
related_plan: docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Summary

U8 now has a concrete editor-side line text identity contract. Renderer `TextBlobId` remains an
opaque renderer resource handle, while editor/app layers are responsible for submitting stable
line/paragraph-sized text slices. The renderer text tests now lock the mechanism: an edited line
chunk receives a new blob/resource key, while unchanged chunks keep stable `TextBlobId`,
`SceneChunk` fingerprint, and text resource fingerprint.

`fret-code-editor` now delta-updates row text cache entries for safe single-line edits. It removes
the edited display rows, shifts unaffected cached row ranges by the byte delta, keeps their
`Arc<str>` allocations, and syncs the row text cache epoch so the next row read does not force a
whole-cache reset.

# Decisions

- Keep document semantic identity out of `TextBlobId`; use editor/app line or paragraph identity as
  the higher-level contract.
- First implementation slice targets row text cache, not row scene replay cache. Retained scene
  fragments also carry geometry and content byte ranges, so scene replay delta preservation needs a
  separate slice that updates retained metadata safely.
- Restrict row text cache delta preservation to single-line edits with stable display-map
  conditions and no folds/inlays. Other cases keep the conservative full-reset path.

# Changed Files

- `crates/fret-render-wgpu/src/text/tests.rs`
- `ecosystem/fret-code-editor/src/editor/input/edit.rs`
- `ecosystem/fret-code-editor/src/editor/paint/mod.rs`
- `ecosystem/fret-code-editor/src/editor/paint/text.rs`
- `ecosystem/fret-code-editor/src/editor/tests/row_text_cache.rs`

# Subagent Findings

- Explorer `019f1e9d-56b7-7e63-9721-86205f0576e3` found Fret can only partially express local
  editor text invalidation today: core/text APIs prepare whole opaque blobs, while
  `fret-code-editor` already has row text and row scene caches that still reset on global buffer
  revisions.
- Explorer `019f1e9d-d989-7601-8d29-7fb48d08f8aa` recommended the Zed/GPUI-aligned split: editor
  owns line/paragraph identity and display-row invalidation; renderer `TextBlobId` stays a resource
  handle.

# Verification

- `cargo nextest run -p fret-render-wgpu local_line_chunk_edit_preserves_unchanged_text_chunk_identity --no-fail-fast`
- `cargo nextest run -p fret-code-editor single_line_edit_preserves_unaffected_row_text_cache_entries --no-fail-fast`
- `cargo check -p fret-code-editor --all-targets`
- `cargo check -p fret-render-wgpu --all-targets`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Next Action

Continue U8 with a separate row scene replay-cache delta slice. That slice must update retained
fragment content/geometry byte ranges before preserving scene chunks across local edits.
