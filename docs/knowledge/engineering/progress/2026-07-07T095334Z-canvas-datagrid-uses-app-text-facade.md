---
type: "Work Progress"
title: "Canvas datagrid stress demo uses app text facade"
description: "Work Progress for Canvas datagrid stress demo uses app text facade."
timestamp: 2026-07-07T09:53:34Z
tags: ["ui-surface", "examples", "canvas-datagrid", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/canvas-datagrid-text-facade"
---

# Summary

Moved `canvas_datagrid_stress_demo.rs` header readout text off
`fret_ui_kit::declarative::text` and onto the app-facing `fret::app::text` facade.

# Details

Changed files:

- `apps/fret-examples/src/canvas_datagrid_stress_demo.rs`
- `apps/fret-examples/tests/canvas_datagrid_stress_demo_surface.rs`

Decision:

- Keep the manual runner, renderer perf hooks, shadcn data grid binding, and stress-control model
  ownership unchanged.
- Narrow the local readout helper from a generic raw `ElementContext<'_, H>`/`UiHost` helper to the
  default app `AppRenderContext<'a>` lane, then delegate to `fret::app::text::control_readout`.
- Update the surface test to require the app readout facade and reject the older `decl_text`
  teaching seam.

# Verification

Passed before commit:

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples --test canvas_datagrid_stress_demo_surface --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- direct text helper scan for `canvas_datagrid_stress_demo.rs` found no `decl_text`,
  `fret_ui_kit::declarative::text`, or `text_control_readout` hits.

Note: the Rust test build still emits the pre-existing `fret-chart::visual_map_track_at` dead code
warning.

# Next Action

Merge this slice back to `main` and push remote `main`, then continue with default App/AppUi text
roles already covered by the app facade.

# Citations

- `apps/fret-examples/src/canvas_datagrid_stress_demo.rs`
- `apps/fret-examples/tests/canvas_datagrid_stress_demo_surface.rs`
