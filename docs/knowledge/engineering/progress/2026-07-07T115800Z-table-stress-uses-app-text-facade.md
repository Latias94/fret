---
type: "Work Progress"
title: "Table stress demo uses app text facade"
description: "Work Progress for Table stress demo uses app text facade."
timestamp: 2026-07-07T11:58:00Z
tags: ["ui-surface", "examples", "text", "facade", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/table-stress-text-facade"
---

# Summary

Moved the table stress demo's fixed header readout, header labels, and row cell text off raw
`fret_ui_kit::declarative::text` helpers and onto the app-facing `fret::app::text` facade.

# Details

Changed files:

- `apps/fret-examples/src/table_stress_demo.rs`
- `apps/fret-examples/tests/table_stress_demo_surface.rs`
- `tools/check_surface_policy.py`
- `tools/test_check_surface_policy.py`

Decision:

- Remove the local `table_stress_*_text` helpers that exposed `AnyElement`.
- Call `text::control_readout` and `text::table_cell` directly from the existing table stress render
  lane.
- Shrink the table stress internal-harness classification by removing the now-unused `AnyElement`
  allowed raw seam while keeping `ElementContext` for `TableStressControls::render_snapshot`.

# Verification

Passed before commit:

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples table_stress_demo_keeps_fixed_table_text_on_roles --no-fail-fast`
- `cargo nextest run -p fret-examples table_stress_demo_model_state_stays_behind_controls_binding --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `git diff --check`

# Next Action

Merge this slice back to `main` and push remote `main`, then continue with the remaining examples
that still import `fret_ui_kit::declarative::text` directly.

# Citations

- `apps/fret-examples/src/table_stress_demo.rs`
- `apps/fret-examples/tests/table_stress_demo_surface.rs`
- `tools/check_surface_policy.py`
