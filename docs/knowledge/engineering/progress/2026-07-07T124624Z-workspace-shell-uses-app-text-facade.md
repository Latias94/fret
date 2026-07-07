---
type: "Work Progress"
title: "Workspace shell uses app text facade"
description: "Work Progress for Workspace shell uses app text facade."
timestamp: 2026-07-07T12:46:24Z
tags: ["ui-surface", "examples", "text", "facade", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/workspace-shell-text-facade"
---

# Summary

Moved the workspace shell editor-rail fixed text helpers from raw
`fret_ui_kit::declarative::text` to the app-facing `fret::app::text` facade.

# Details

Changed files:

- `apps/fret-examples/src/workspace_shell_demo/driver.rs`
- `apps/fret-examples/tests/workspace_shell_editor_rail_surface.rs`

Decision:

- Keep `workspace_shell_demo` classified as an advanced shell because it owns the manual driver,
  diagnostics, workspace frame, and command dispatch plumbing.
- Move only the fixed editor-rail helper text onto `AppRenderContext` + `fret::app::text`.
- Leave existing non-text advanced seams untouched.
- Update the editor-rail source test to require app text roles and forbid the old `decl_text`
  helper shape.

# Verification

Passed before commit:

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples workspace_shell_demo_composes_editor_rail_through_workspace_frame_slots --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `git diff --check`

# Next Action

Merge this slice back to `main` and push remote `main`, then continue with the remaining explicit
advanced/KernelApp text seams.

# Citations

- `apps/fret-examples/src/workspace_shell_demo/driver.rs`
- `apps/fret-examples/tests/workspace_shell_editor_rail_surface.rs`
