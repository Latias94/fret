---
type: Work Progress
title: Phase 3 U13 IMUI plot binding migration
tags: fret,phase3,u13,cookbook,imui,plot,facade
timestamp: 2026-07-04
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
git_branch: feat/ui-framework-phase2-refactor
verified_by:
  - cargo nextest run -p fret-plot line_plot_binding_creates_props_with_state_and_output_without_public_raw_handles --no-fail-fast
  - cargo nextest run -p fret-cookbook --lib cookbook_imui_plot_example_keeps_optional_plot_adapter_teaching_surface --no-fail-fast
  - cargo nextest run -p fret-cookbook --lib migrated_basics_examples_use_the_new_app_surface --no-fail-fast
  - cargo check -p fret-cookbook --features cookbook-imui-plot --example imui_plot_basics
  - cargo check -p fret-cookbook --all-targets
  - cargo check -p fret-plot --features imui --all-targets
  - cargo check -p fret-examples-imui --all-targets
  - python3 tools/test_check_surface_policy.py
  - python3 tools/check_surface_policy.py
  - python3 tools/gate_imui_facade_teaching_source.py
  - python3 tools/check_consumption_profiles.py
  - python3 tools/check_execution_surface.py
  - python3 tools/check_layering.py
  - cargo fmt --all --check
  - git diff --check
---

# Summary

Phase 3 U13 retired the `imui_plot_basics.rs` advanced/manual quarantine record by adding a
plot-specific app-facing binding in `fret-plot`.

- Added `fret_plot::LinePlotPanelBinding`, which owns the raw `Model<LinePlotModel>`,
  `Model<PlotState>`, and `Model<PlotOutput>` internally.
- Added `LinePlotPanelBinding::new(host, model)` over `fret_runtime::ModelHost` so app code can
  pass `&mut App` without naming `ModelStore` or calling `app.models_mut().insert(...)`.
- Added `LinePlotPanelBinding::panel_props()`, `output_layout(...)`, and `output_paint(...)` as
  narrow plot-specific read/adapter helpers.
- Added `fret_plot::imui::line_plot_panel_binding(...)` so the IMUI cookbook lesson can render the
  existing declarative panel without constructing `LinePlotPanelProps::new(raw_model)` itself.
- Migrated `apps/fret-cookbook/examples/imui_plot_basics.rs` off `fret_core`, `fret_runtime`,
  `fret_ui`, raw `Model<T>`, `ColumnProps`, and root `imui_raw(...)`.
- Moved `imui_plot_basics.rs` from `ADVANCED_MANUAL_SURFACES` to `DEFAULT_AUTHORING_SURFACES` and
  tightened cookbook source assertions against the deleted raw seams.

# Decision

Do not add a broad app-facade `IntoModel<T>` or a root `fret::plot` dependency proxy for this
slice. `fret-plot` remains an explicit optional ecosystem dependency, and the app-facing helper is
domain-specific: a line plot panel binding, not a general runtime model bridge.

# Verification

The focused plot/cookbook checks, U13 source-policy gates, consumption/execution/layering checks,
formatting, and whitespace checks passed on 2026-07-04. See `verified_by` for the exact commands.

# Next

Continue U13 by auditing the remaining advanced/manual cookbook/example records. Keep
`async_inbox_basics.rs` deferred until an app-facing host/effect action helper exists.

# Citations

- [Phase 3 retained bridge deletion plan](../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md)
- [IMUI cookbook facade audits](../subagents/2026-07-03-phase3-u13-imui-cookbook-facade-audit.md)
