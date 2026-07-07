---
type: "Work Progress"
title: "IMUI node graph demo uses app text facade"
description: "Work Progress for IMUI node graph demo uses app text facade."
timestamp: 2026-07-07T09:10:24Z
tags: ["ui-surface", "examples", "imui", "node-graph", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/imui-node-graph-text-facade"
---

# Summary

Moved `imui_node_graph_demo.rs` compatibility title text off raw `ElementContext`, `UiHost`,
`AnyElement`, and `decl_text` signatures onto the app-facing `fret::app::text` and `AppElement`
surface.

# Details

Changed files:

- `apps/fret-examples/src/imui_node_graph_demo.rs`
- `apps/fret-examples/tests/imui_node_graph_demo_surface.rs`
- `tools/check_surface_policy.py`
- `tools/test_check_surface_policy.py`

Decision:

- Keep the demo classified as an advanced/manual compatibility proof because it still owns raw
  `fret_runtime::Model` handles for the retained node-graph bridge.
- Remove `fret_ui`, `AnyElement`, and `ElementContext` from the demo's allowed raw seam list because
  the fixed title helper now uses `AppRenderContext` and the app text facade.

# Verification

Passed before commit:

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples --test imui_node_graph_demo_surface --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- raw seam scan for `imui_node_graph_demo.rs` found no direct `fret_ui::`, `AnyElement`,
  `ElementContext`, `UiHost`, `decl_text`, or `fret_ui_kit::declarative::text` hits.

Note: the Rust test build still emits the pre-existing `fret-chart::visual_map_track_at` dead code
warning.

# Next Action

Merge this slice back to `main` and push remote `main`, then continue with the next remaining
advanced/manual surface whose raw seam is facade shrinkage rather than retained runner or renderer
ownership.

# Citations

- `apps/fret-examples/src/imui_node_graph_demo.rs`
- `apps/fret-examples/tests/imui_node_graph_demo_surface.rs`
- `tools/check_surface_policy.py`
