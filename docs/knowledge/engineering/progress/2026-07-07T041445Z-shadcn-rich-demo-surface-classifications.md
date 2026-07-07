---
type: "Work Progress"
title: "Shadcn rich demo surface classifications"
description: "Work Progress for Shadcn rich demo surface classifications."
timestamp: 2026-07-07T04:14:45Z
tags: ["ui-surface", "shadcn", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
---

# Summary

Classified four shadcn rich behavior demos as `advanced_manual` public-example surfaces:

- `date_picker_demo.rs`
- `form_demo.rs`
- `table_demo.rs`
- `sonner_demo.rs`

Each file owns manual runner/retained-tree or AppUi render-root bridge plumbing plus component
behavior proof responsibilities that are broader than first-contact app authoring.

# Details

Changed files:

- `tools/check_surface_policy.py`
- `tools/test_check_surface_policy.py`

Decision:

- Keep these as maintained behavior proofs until smaller cookbook/default examples own the
  copyable path and app-facing facades cover their runner, overlay, table, toast, and typed child
  needs.
- This avoids teaching `FnDriver`, `UiTree`, raw `fret_ui`, or raw action host plumbing through
  public example inventory.

Verification passed before commit:

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- Raw public example inventory script now reports 7 remaining uncovered files.

# Next Action

Remaining public-example raw-surface gaps:

- `drag_demo.rs`
- `genui_demo.rs`
- `imui_editor_proof_demo.rs`
- `imui_node_graph_demo.rs`
- `markdown_demo.rs`
- `plot_image_demo.rs`
- `tags_demo.rs`

# Citations

- `docs/knowledge/engineering/subagents/2026-07-07-public-example-surface-followup-audit.md`
