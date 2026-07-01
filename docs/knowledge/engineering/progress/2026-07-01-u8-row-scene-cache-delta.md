---
type: Work Progress
title: U8 row scene cache delta preservation
tags: fret,u8,text,code-editor,row-scene-cache,scene-chunks
timestamp: 2026-07-01
related_plan: docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Summary

`fret-code-editor` now delta-updates retained row scene cache entries for safe single-line edits.
The edit path uses the same stable display-map conditions as row text/row geometry cache shifting:
single-line old/new edit, unchanged wrap configuration, stable edited row start, and no folds/inlays.

The retained scene migration is intentionally conservative. It preserves only plain, non-rich row
scene entries with no preedit, folds, row spans, or mismatched content/geometry byte ranges. The
edited display rows are removed, unaffected rows are shifted by the display-row delta, content and
geometry byte ranges are shifted by the edit byte delta, retained `SceneChunk` payloads and hosted
resources are kept stable, and syntax window replay-plan caches are invalidated because their keys
include revision/display epochs.

# Decisions

- Preserve only the plain row-scene path in this slice. Syntax/rich/preedit/fold/inlay rows still
  rebuild, because their correctness depends on broader semantic spans and display composition.
- Rebuild the row scene LRU queue from surviving entries instead of carrying stale duplicate queue
  records across row shifts.
- Keep retained chunk payload identity stable by cloning the retained fragment metadata and
  replacing only the `RowContentSnapshot` and `RowGeom` ranges.

# Changed Files

- `ecosystem/fret-code-editor/src/editor/input/edit.rs`
- `ecosystem/fret-code-editor/src/editor/paint/mod.rs`
- `ecosystem/fret-code-editor/src/editor/paint/scene.rs`
- `ecosystem/fret-code-editor/src/editor/tests/row_text_cache.rs`

# Verification

- `cargo nextest run -p fret-code-editor single_line_edit_preserves_unaffected_row_scene_cache_entries --no-fail-fast`
- `cargo nextest run -p fret-code-editor single_line_edit_preserves_unaffected_row_text_cache_entries single_line_edit_preserves_unaffected_row_scene_cache_entries --no-fail-fast`
- `cargo nextest run -p fret-code-editor --features syntax-rust single_line_edit_preserves_unaffected_row_scene_cache_entries prepaint_row_scene_replay_plan_reuses_stable_window_plan --no-fail-fast`
- `cargo check -p fret-code-editor --all-targets`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Next Action

Continue U8 by adding text-heavy diagnostic gates that make editor row text/row scene cache hit
rates and retained scene replay behavior visible in repeatable perf outputs.
