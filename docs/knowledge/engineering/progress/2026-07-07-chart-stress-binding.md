---
type: "Work Progress"
title: "Chart stress binding cleanup"
description: "Work Progress for the chart stress binding cleanup."
timestamp: 2026-07-07T01:35:00Z
tags: ["fret", "chart", "examples", "stress", "public-surface", "raw-model", "binding"]
git_branch: "refactor/chart-stress-binding"
verified_by: "cargo nextest run -p fret-examples --test basic_chart_demos_surface chart_stress_demo_uses_declarative_canvas_panel --no-fail-fast"
---

# Summary

`chart_stress_demo.rs` now keeps its stress chart behind `ChartCanvasPanelBinding` instead of
storing a raw `Model<ChartEngine>` and manually wiring `ChartCanvasPanelProps`.

# Details

- Added `ChartCanvasPanelBinding::read_engine(...)` and `update_engine(...)` so advanced examples
  can read or mutate the controlled engine without exposing the raw model handle.
- Migrated `chart_stress_demo.rs` to store `ChartCanvasPanelBinding`, render with
  `panel_props()`, observe paint through `observe_engine_paint(...)`, and read stats through
  `read_engine(...)`.
- Strengthened `basic_chart_demos_surface` so the stress demo must keep the binding shape and must
  not regress to raw `Model<ChartEngine>`, direct `app.models_mut().insert(engine)`, or manual
  `ChartCanvasPanelProps::new(spec)` wiring.

# Verification

- `cargo fmt --all --check`
- `cargo check -p fret-chart`
- `cargo check -p fret-examples --lib --tests`
- `cargo nextest run -p fret-chart chart_canvas_binding_creates_props_with_engine_and_output_without_public_raw_handle --no-fail-fast`
- `cargo nextest run -p fret-examples --test basic_chart_demos_surface chart_stress_demo_uses_declarative_canvas_panel --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_layering.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_consumption_profiles.py`
- `python3 ~/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`

# Next Action

Use `ChartCanvasPanelBinding::read_engine(...)` for future chart examples that need diagnostics or
stats from a controlled engine. Keep raw chart panel props for component tests and genuinely shared
output-model contracts.

# Citations

- [binding.rs](../../../../ecosystem/fret-chart/src/binding.rs)
- [chart_stress_demo.rs](../../../../apps/fret-examples/src/chart_stress_demo.rs)
- [basic_chart_demos_surface.rs](../../../../apps/fret-examples/tests/basic_chart_demos_surface.rs)
