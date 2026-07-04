---
type: Subagent Finding
title: Phase 3 U13 remaining surface audit
tags: fret,phase3,u13,cookbook,examples,subagent
timestamp: 2026-07-04
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
subagent_id: 019f2b17-acee-7270-8553-a12fdeb9a2d3,019f2b18-07c6-7200-a1a7-ff442d96db08
git_branch: feat/ui-framework-phase2-refactor
---

# Finding

Read-only explorers `019f2b17-acee-7270-8553-a12fdeb9a2d3` and
`019f2b18-07c6-7200-a1a7-ff442d96db08` audited the remaining U13 source-policy surfaces.

# Evidence

- `chart_interactions_basics.rs` was the best cookbook migration candidate: it looked like a
  normal tutorial surface but still exposed `advanced::prelude`, raw `Model<ChartEngine>`,
  `ChartCanvasPanelProps`, `ViewCacheProps`, and `UiAppDriver` command callbacks.
- `gizmo_basics.rs` is the next plausible cookbook migration candidate, but it needs more facade
  work first: pointer `on_wheel`, canvas/vector path helpers, and local-state command migration.
- `embedded_viewport_basics.rs`, `external_texture_import_basics.rs`,
  `utility_window_materials_windows.rs`, `docking_basics.rs`, and `customv1_basics.rs` still own
  real advanced interop seams and should not be reclassified without new public facades.
- Non-cookbook examples still have useful cleanup opportunities:
  `apps/fret-examples/src/simple_todo_demo.rs` can split copyable view code from runner glue;
  `todo_demo.rs` can separate semantics harness code from app-facing helpers; `plot_demo.rs`
  overlaps with newer declarative plot demos and should be reclassified, merged, or deleted.
- Source policy was too permissive for default surfaces that imported `fret_app::`; tightening that
  rule exposed and fixed the `theme_switching_basics.rs` raw `Effect` leak.

# Recommendation

After the chart facade slice, prioritize a non-cookbook cleanup commit that splits
`simple_todo_demo.rs` into default-clean app-view code plus an internal harness for runner glue.
Then revisit `gizmo_basics.rs` only after the needed wheel/vector-path/canvas helpers are scoped.

# Disposition

Accepted for chart: the chart cookbook was migrated to `fret::chart` and default-clean coverage.
The remaining recommendations are queued as U13 follow-up work.

# Citations

- [Cookbook chart example](../../../apps/fret-cookbook/examples/chart_interactions_basics.rs)
- [Simple todo demo](../../../apps/fret-examples/src/simple_todo_demo.rs)
- [Surface policy gate](../../../tools/check_surface_policy.py)
