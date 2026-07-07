---
type: "Work Progress"
title: "Async playground advanced surface classification"
description: "Work Progress for Async playground advanced surface classification."
timestamp: 2026-07-07T03:51:04Z
tags: ["ui-surface", "query", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
---

# Summary

Classified `apps/fret-examples/src/async_playground_demo.rs` as an `advanced_manual` query
playground. A source read showed it is already on `AppRenderContext` for the outer view but still
owns raw `PressableProps`, `ScrollHandle`, `Vec<AnyElement>` dynamic child collections, and
low-level text helper signatures.

# Details

Changed files:

- `tools/check_surface_policy.py`
- `tools/test_check_surface_policy.py`

Decision:

- Do not mark this file default-clean yet. The copyable query basics path is already represented by
  smaller query examples/cookbook surfaces.
- The retirement condition is the standard advanced/manual condition, but practically this demo
  should move only after app-facing pressable row, scroll handle, and typed dynamic-child wrappers
  exist.

Verification passed before commit:

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- Raw public example inventory script now reports 11 remaining uncovered files.

# Next Action

Continue with either:

- shadcn richer manual-runner demos (`date_picker_demo.rs`, `form_demo.rs`, `table_demo.rs`,
  `sonner_demo.rs`) as a classification group, or
- `markdown_demo.rs` / plot-tail demos as smaller source reads to decide migration vs quarantine.

# Citations

- `docs/knowledge/engineering/subagents/2026-07-07-public-example-surface-followup-audit.md`
