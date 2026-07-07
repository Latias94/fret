---
type: "Work Progress"
title: "Editor notes demo uses app text facade"
description: "Work Progress for Editor notes demo uses app text facade."
timestamp: 2026-07-07T11:02:14Z
tags: ["ui-surface", "examples", "editor", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/editor-notes-text-facade"
---

# Summary

Moved `editor_notes_demo.rs` readout, section, and paragraph helpers off raw
`fret_ui_kit::declarative::text` imports and onto the app-facing `fret::app::text` facade.

# Details

Changed files:

- `apps/fret-examples/src/editor_notes_demo.rs`
- `apps/fret-examples/tests/editor_notes_editor_rail_surface.rs`
- `tools/check_surface_policy.py`

Decision:

- Keep workspace-frame rail composition, asset bindings, theme preset binding, inspector property
  rows, and `TextFieldDraftController` behavior unchanged.
- Convert `editor_notes_readout_text`, `editor_notes_section_text`, and
  `editor_notes_paragraph_text` to `AppRenderContext<'a>`.
- Extend the surface test to require the app text facade and forbid the old `decl_text` constructor
  path for editor rail text.
- Shrink the `editor_notes_demo.rs` surface-policy allowlist by removing the stale
  `ElementContext` raw seam after the helper conversion.

# Verification

Passed before commit:

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples --test editor_notes_editor_rail_surface --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `git diff --check`

Note: the Rust test build still emits the pre-existing `fret-chart::visual_map_track_at` dead code
warning.

# Next Action

Merge this slice back to `main` and push remote `main`, then continue with remaining app text facade
seams that do not require new facade roles.

# Citations

- `apps/fret-examples/src/editor_notes_demo.rs`
- `apps/fret-examples/tests/editor_notes_editor_rail_surface.rs`
- `tools/check_surface_policy.py`
