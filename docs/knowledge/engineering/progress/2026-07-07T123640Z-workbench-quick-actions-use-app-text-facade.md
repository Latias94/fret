---
type: "Work Progress"
title: "Workbench quick actions use app text facade"
description: "Work Progress for Workbench quick actions use app text facade."
timestamp: 2026-07-07T12:36:40Z
tags: ["ui-surface", "examples", "text", "facade", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/workbench-quick-actions-text-facade"
---

# Summary

Moved the IMUI editor workbench quick-action chrome/status helpers from raw
`fret_ui_kit::declarative::text` to the app-facing `fret::app::text` facade.

# Details

Changed files:

- `apps/fret-examples/src/imui_editor_workbench_demo/quick_actions.rs`
- `apps/fret-examples/src/imui_editor_workbench_demo/quick_actions/copy.rs`
- `apps/fret-examples/tests/imui_editor_workbench_golden_path_surface.rs`

Decision:

- Keep quick-action rendering in the existing `AppUi` lane and only change the fixed text helpers.
- Use `AppRenderContext` helper signatures instead of `UiHost`/`ElementContext` raw text seams.
- Add golden-path source assertions that require `fret::app::text` and forbid `decl_text` from the
  quick-action render and copy owners.

# Verification

Passed before commit:

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples imui_editor_workbench_demo_is_the_canonical_editor_workbench_route --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `git diff --check`

# Next Action

Merge this slice back to `main` and push remote `main`, then continue classifying remaining
KernelApp/advanced text seams separately from default app-lane candidates.

# Citations

- `apps/fret-examples/src/imui_editor_workbench_demo/quick_actions.rs`
- `apps/fret-examples/src/imui_editor_workbench_demo/quick_actions/copy.rs`
- `apps/fret-examples/tests/imui_editor_workbench_golden_path_surface.rs`
