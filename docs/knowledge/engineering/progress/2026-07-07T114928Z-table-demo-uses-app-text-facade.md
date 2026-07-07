---
type: "Work Progress"
title: "Table demo uses app text facade"
description: "Work Progress for Table demo uses app text facade."
timestamp: 2026-07-07T11:49:28Z
tags: ["ui-surface", "examples", "text", "facade", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/table-demo-text-facade"
---

# Summary

Moved the table demo's fixed header readout, header menu labels, and row cell text off raw
`fret_ui_kit::declarative::text` helpers and onto the app-facing `fret::app::text` facade.

# Details

Changed files:

- `apps/fret-examples/src/table_demo.rs`
- `apps/fret-examples/tests/table_demo_surface.rs`
- `tools/check_surface_policy.py`
- `tools/test_check_surface_policy.py`

Decision:

- Remove the local `table_demo_*_text` helpers that exposed `ElementContext`/`AnyElement`.
- Call `text::control_readout` and `text::table_cell` directly from the existing table render lane.
- Shrink the table-demo advanced-surface classification by removing the now-unused
  `AnyElement`/`ElementContext` allowed raw seams.

# Verification

Passed before commit:

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples table_demo_keeps_fixed_table_text_on_roles --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `git diff --check`

# Next Action

Merge this slice back to `main` and push remote `main`, then continue with `table_stress_demo.rs`,
which has the same `control_readout`/`table_cell` text seam shape.

# Citations

- `apps/fret-examples/src/table_demo.rs`
- `apps/fret-examples/tests/table_demo_surface.rs`
- `tools/check_surface_policy.py`
