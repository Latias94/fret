---
type: "Work Progress"
title: "Datatable demo uses app text facade"
description: "Work Progress for Datatable demo uses app text facade."
timestamp: 2026-07-07T11:37:39Z
tags: ["ui-surface", "examples", "text", "facade", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/datatable-text-facade"
---

# Summary

Moved the datatable demo's fixed header readout and row cell text off raw
`fret_ui_kit::declarative::text` helpers and onto the app-facing `fret::app::text` facade.

# Details

Changed files:

- `apps/fret-examples/src/datatable_demo.rs`
- `apps/fret-examples/tests/datatable_demo_surface.rs`
- `tools/check_surface_policy.py`
- `tools/test_check_surface_policy.py`

Decision:

- Remove the local `datatable_demo_*_text` helpers that exposed `ElementContext`/`AnyElement`.
- Call `text::control_readout` and `text::table_cell` directly from the existing app render lane.
- Shrink the datatable advanced-surface classification by removing the now-unused
  `AnyElement`/`ElementContext` allowed raw seams.

# Verification

Passed before commit:

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples datatable_demo_keeps_fixed_table_text_on_roles --no-fail-fast`
- `cargo nextest run -p fret-examples datatable_demo_uses_local_state_table_output --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `git diff --check`

# Next Action

Merge this slice back to `main` and push remote `main`, then continue with `table_demo.rs` and
`table_stress_demo.rs`, which have the same `control_readout`/`table_cell` text seam shape.

# Citations

- `apps/fret-examples/src/datatable_demo.rs`
- `apps/fret-examples/tests/datatable_demo_surface.rs`
- `tools/check_surface_policy.py`
