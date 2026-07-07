---
type: "Work Progress"
title: "Editor notes device shell uses app text facade"
description: "Work Progress for Editor notes device shell uses app text facade."
timestamp: 2026-07-07T09:00:14Z
tags: ["ui-surface", "examples", "editor-notes", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/markdown-text-facade"
---

# Summary

Moved `editor_notes_device_shell_demo.rs` mobile header text helpers off raw `ElementContext`,
`UiHost`, `AnyElement`, and `decl_text` signatures onto the app-facing `fret::app::text` and
`AppElement` surface.

# Details

Changed files:

- `apps/fret-examples/src/editor_notes_device_shell_demo.rs`
- `apps/fret-examples/tests/editor_notes_device_shell_surface.rs`
- `tools/check_surface_policy.py`
- `tools/test_check_surface_policy.py`

Decision:

- Use `AppRenderContext` and `AppElement` for the demo-local text helper signatures.
- Keep `theme_snapshot()` on the app-facing `RenderContextAccess` trait import instead of restoring
  `fret_ui_kit::declarative::ElementContextThemeExt`.
- Remove `AnyElement` and `ElementContext` from the editor notes device-shell advanced/manual
  allowed raw seam list.

# Verification

Passed before commit:

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples --test editor_notes_device_shell_surface --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- raw seam scan for `editor_notes_device_shell_demo.rs` found no `AnyElement`, `ElementContext`,
  `UiHost`, `decl_text`, `fret_ui_kit::declarative::text`, or `ElementContextThemeExt` hits.

Note: the Rust test build still emits the pre-existing `fret-chart::visual_map_track_at` dead code
warning.

# Next Action

After this slice lands, merge the completed branch back to `main` and push remote `main` per the
current maintainer workflow.

# Citations

- `apps/fret-examples/src/editor_notes_device_shell_demo.rs`
- `apps/fret-examples/tests/editor_notes_device_shell_surface.rs`
- `tools/check_surface_policy.py`
