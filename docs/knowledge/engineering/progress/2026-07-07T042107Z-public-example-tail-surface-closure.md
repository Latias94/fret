---
type: "Work Progress"
title: "Public example tail surface closure"
description: "Work Progress for Public example tail surface closure."
timestamp: 2026-07-07T04:21:07Z
tags: ["ui-surface", "examples", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
---

# Summary

Closed the remaining raw public-example surface inventory by classifying the final seven uncovered
examples under explicit advanced/manual ownership:

- `drag_demo.rs`
- `plot_image_demo.rs`
- `tags_demo.rs`
- `markdown_demo.rs`
- `genui_demo.rs`
- `imui_editor_proof_demo.rs`
- `imui_node_graph_demo.rs`

# Details

Changed files:

- `tools/check_surface_policy.py`
- `tools/test_check_surface_policy.py`

Decision:

- `drag_demo.rs` remains a plot drag-overlay proof with manual runner and retained tree hooks.
- `plot_image_demo.rs` and `tags_demo.rs` keep advanced web/demo-shell launch helpers even though
  their native run path uses `FretApp`.
- `markdown_demo.rs` remains a native markdown/remote-image/rendering proof with raw image/SVG and
  text helper boundaries.
- `genui_demo.rs`, `imui_editor_proof_demo.rs`, and `imui_node_graph_demo.rs` remain advanced
  generator/editor/compatibility proofs.

Verification passed before commit:

- `python3 -m py_compile tools/check_surface_policy.py tools/test_check_surface_policy.py`
- `cargo fmt --all --check`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_layering.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_consumption_profiles.py`
- engineering wiki memory validation for `docs/knowledge/engineering`
- `git diff --check`
- Raw public example inventory script now reports 0 remaining uncovered files.

# Next Action

The public example inventory is now covered by policy. The next useful refactor should shift from
classification to real migration:

- add app-facing wrappers for repeated raw needs (pressable row, scroll handle, typed dynamic child
  sinks, plot web launch helpers), then retire individual advanced/manual records as demos become
  default-clean;
- or delete/replace compatibility-only demos once their cookbook/default equivalents are proven.

# Citations

- `docs/knowledge/engineering/subagents/2026-07-07-public-example-surface-followup-audit.md`
