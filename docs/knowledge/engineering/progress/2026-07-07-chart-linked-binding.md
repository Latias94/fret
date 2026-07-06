---
type: "Work Progress"
title: "Chart linked binding cleanup"
description: "Work Progress for the chart multi-axis linked binding cleanup."
timestamp: 2026-07-07T01:05:00Z
tags: ["fret", "chart", "examples", "public-surface", "raw-model", "binding"]
git_branch: "refactor/chart-linked-output-binding"
verified_by: "cargo nextest run -p fret-examples --test basic_chart_demos_surface chart_multi_axis_demo_uses_declarative_canvas_panel_with_linked_inputs --no-fail-fast"
---

# Summary

`chart_multi_axis_demo.rs` now uses linked chart bindings instead of owning raw chart engine,
output, brush, axis-pointer, and domain-window model wiring directly.

# Details

- Added `ChartCanvasLinkedGroupBinding` to own shared linked-chart state and `LinkedChartGroup`
  ticking behind a chart-specific API.
- Added `ChartCanvasLinkedPanelBinding` to own each linked panel's engine/output model handles and
  build linked `ChartCanvasPanelProps` without exposing raw `Model<T>` handles to app examples.
- Added `ChartCanvasLinkedStateBinding` so diagnostics can read shared domain windows without
  storing a raw model handle.
- Migrated `chart_multi_axis_demo.rs` state, render wiring, diagnostics snapshots, link logging, and
  deterministic auto-zoom updates onto the linked binding surface.
- Strengthened `basic_chart_demos_surface` so the multi-axis demo must keep the linked binding
  shape and must not regress to raw `Model<ChartEngine>`, `Model<ChartCanvasOutput>`,
  `LinkedChartMember`, or manual linked `ChartCanvasPanelProps` wiring.

# Verification

- `cargo fmt --all --check`
- `cargo check -p fret-chart`
- `cargo check -p fret-examples --lib --tests`
- `cargo nextest run -p fret-chart chart_canvas_linked_group_binding_creates_panel_props_without_public_raw_handles --no-fail-fast`
- `cargo nextest run -p fret-examples --test basic_chart_demos_surface chart_multi_axis_demo_uses_declarative_canvas_panel_with_linked_inputs --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_layering.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_consumption_profiles.py`
- `python3 ~/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`

# Next Action

Keep `chart_stress_demo.rs` as the remaining chart perf harness until a stress-specific owner or
stats binding is worth naming. For non-stress chart examples, prefer adding a narrow binding before
allowing raw chart model handles in app-facing source.

# Citations

- [binding.rs](../../../../ecosystem/fret-chart/src/binding.rs)
- [chart_multi_axis_demo.rs](../../../../apps/fret-examples/src/chart_multi_axis_demo.rs)
- [basic_chart_demos_surface.rs](../../../../apps/fret-examples/tests/basic_chart_demos_surface.rs)
