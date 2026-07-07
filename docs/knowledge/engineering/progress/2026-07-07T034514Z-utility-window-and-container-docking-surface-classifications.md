---
type: "Work Progress"
title: "Utility window and container docking surface classifications"
description: "Work Progress for Utility window and container docking surface classifications."
timestamp: 2026-07-07T03:45:14Z
tags: ["ui-surface", "window", "docking", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
---

# Summary

Classified three remaining public-example raw surfaces:

- `launcher_utility_window_demo.rs` as `advanced_manual`
- `launcher_utility_window_materials_demo.rs` as `advanced_manual`
- `container_queries_docking_demo.rs` as `internal_harness`

The utility-window demos intentionally own manual `UiAppDriver` hooks, window style effects,
drag/resize or material requests, and raw diagnostics. The container-query docking demo owns manual
`FnDriver`, `UiTree`, diagnostics/script driving, docking runtime, and accessibility bridge hooks.

# Details

Changed files:

- `tools/check_surface_policy.py`
- `tools/test_check_surface_policy.py`

Verification passed before commit:

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- Raw public example inventory script now reports 12 remaining uncovered files.

# Next Action

Remaining likely classifications/migrations:

- `async_playground_demo.rs`: inspect for possible default-facade text helper migration before
  classifying.
- shadcn richer control demos (`date_picker_demo.rs`, `form_demo.rs`, `table_demo.rs`,
  `sonner_demo.rs`): likely advanced/manual until a public overlay/manual-runner facade exists.
- IMUI/GenUI/markdown/plot-tail demos need separate reads because some may be deletable or
  migratable.

# Citations

- `docs/knowledge/engineering/subagents/2026-07-07-public-example-surface-followup-audit.md`
