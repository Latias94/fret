---
type: Work Progress
title: Phase 3 U13 chart facade
tags: fret,phase3,u13,chart,cookbook,surface-policy
timestamp: 2026-07-04
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
git_branch: feat/ui-framework-phase2-refactor
---

# Summary

Phase 3 U13 now has an explicit app-facing chart lane for the chart interactions cookbook.

- Added optional `fret/chart` and `fret::chart::{...}` as an explicit opt-in module, not part of
  `fret::app::prelude::*`.
- Added `ChartCanvas` in `ecosystem/fret/src/view/chart.rs`, wrapping existing `fret-chart`
  declarative panel wiring while hiding raw `ChartCanvasPanelProps`, chart output `Model<T>`, and
  raw `ViewCacheProps` from default app examples.
- Migrated `chart_interactions_basics.rs` from `ui_app_with_hooks(...)` + `UiAppDriver`
  command callbacks to `FretApp` + `View` + `LocalState<T>` + typed `cx.actions()` handlers.
- Moved direct chart dependencies from `fret-cookbook` into the optional `fret/chart` facade, so
  the cookbook feature now enables `fret/chart` instead of direct `fret-chart`/`delinea`.
- Moved `chart_interactions_basics.rs` from advanced manual quarantine to default clean
  source-policy coverage, added a default-surface ban on direct `fret_chart::`, and tightened the
  existing default-surface policy to reject direct `fret_app::`.
- Fixed the resulting default-surface leak in `theme_switching_basics.rs` by using
  `cx.request_animation_frame()` instead of raw `fret_app::Effect`.

# Verification

- `cargo check -p fret --features chart --lib`
- `cargo check -p fret-cookbook --features cookbook-chart --example chart_interactions_basics`
- `cargo check -p fret-cookbook --all-targets`
- `cargo check -p fret-cookbook --features cookbook-chart --all-targets`
- `cargo nextest run -p fret-cookbook --lib --no-fail-fast`
- `cargo nextest run -p fret --features chart usage_docs_prefer_grouped_app_ui_actions root_surface_exposes_explicit_style_and_icon_modules root_surface_module_budget_is_curated_and_closed app_prelude_omits_low_level_mechanism_types --no-fail-fast`
- `python3 tools/test_check_surface_policy.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_layering.py`
- `cargo fmt --all --check`
- `git diff --check`

# Next

Continue U13 with remaining non-cookbook and advanced surfaces. The strongest next commit slice is
to split `apps/fret-examples/src/simple_todo_demo.rs` so copyable app-view code can become
default-clean while runner glue remains an internal harness.

# Citations

- [Phase 3 retained bridge deletion plan](../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md)
- [Remaining surface audit](../subagents/2026-07-04-phase3-u13-remaining-surface-audit.md)
- [Cookbook chart example](../../../apps/fret-cookbook/examples/chart_interactions_basics.rs)
- [Fret chart facade](../../../ecosystem/fret/src/view/chart.rs)
- [Surface policy gate](../../../tools/check_surface_policy.py)
