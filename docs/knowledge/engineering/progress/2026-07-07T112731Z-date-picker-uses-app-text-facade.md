---
type: "Work Progress"
title: "Date picker demo uses app text facade"
description: "Work Progress for Date picker demo uses app text facade."
timestamp: 2026-07-07T11:27:31Z
tags: ["ui-surface", "examples", "text", "facade", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/date-picker-text-facade"
---

# Summary

Moved the date picker demo's fixed chrome readout, switch labels, and prose off raw
`fret_ui_kit::declarative::text` helpers and onto the app-facing `fret::app::text` facade.

# Details

Changed files:

- `apps/fret-examples/src/date_picker_demo.rs`
- `apps/fret-examples/tests/date_picker_demo_surface.rs`
- `tools/check_surface_policy.py`
- `tools/test_check_surface_policy.py`

Decision:

- Remove the local `date_picker_*_text` helpers that exposed `ElementContext`/`AnyElement`.
- Call `text::control_readout`, `text::control_label`, and `text::paragraph` directly from the
  existing app render lane.
- Shrink the date picker advanced-surface classification by removing the now-unused
  `AnyElement`/`ElementContext` allowed raw seams.

# Verification

Passed before commit:

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples date_picker_demo_keeps_fixed_chrome_text_on_roles --no-fail-fast`
- `cargo nextest run -p fret-examples app_state_demos_use_app_local_state_imports --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `git diff --check`

# Next Action

Merge this slice back to `main` and push remote `main`, then continue with the table/datatable demos
that are now unblocked by `fret::app::text::table_cell`.

# Citations

- `apps/fret-examples/src/date_picker_demo.rs`
- `apps/fret-examples/tests/date_picker_demo_surface.rs`
- `tools/check_surface_policy.py`
