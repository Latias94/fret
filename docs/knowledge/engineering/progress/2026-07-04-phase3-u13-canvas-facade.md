---
type: Work Progress
title: Phase 3 U13 canvas facade
tags: fret,phase3,u13,canvas,cookbook,surface-policy
timestamp: 2026-07-04
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
git_branch: feat/ui-framework-phase2-refactor
---

# Summary

Phase 3 U13 now has an explicit app-facing canvas lane for the pan/zoom cookbook.

- Added optional `fret/canvas` and `fret::canvas::{...}` as an explicit opt-in module, not part of
  `fret::app::prelude::*`.
- Added `AppCanvasPainter` and `PanZoomCanvas` in `ecosystem/fret/src/view/canvas.rs`, wrapping
  existing `fret-canvas` pan/zoom wiring while hiding raw `CanvasPainter`, raw pointer action host
  callbacks, `CanvasCachePolicy`, and raw `Model<T>` handles from app examples.
- Migrated `canvas_pan_zoom_basics.rs` to `LocalState<T>`, `fret::canvas`, `fret::pointer`, and
  grouped `cx.actions().local/locals_with(...)` handlers.
- Fixed reset behavior so `ResetNode` clears `node_drag` as well as node origin/count.
- Moved `canvas_pan_zoom_basics.rs` from advanced manual quarantine to default clean
  source-policy coverage and added a default-surface ban on direct `fret_canvas::`.

# Verification

- `cargo check -p fret --features canvas --lib`
- `cargo check -p fret-cookbook --features cookbook-canvas --example canvas_pan_zoom_basics`
- `cargo check -p fret-cookbook --all-targets`
- `cargo check -p fret-cookbook --features cookbook-canvas --all-targets`
- `cargo nextest run -p fret-cookbook --lib migrated_basics_examples_use_the_new_app_surface advanced_examples_use_the_explicit_advanced_surface advanced_view_examples_prefer_app_ui_and_ui_aliases selected_cookbook_examples_prefer_handle_first_tracked_reads --no-fail-fast`
- `cargo nextest run -p fret-cookbook --lib --no-fail-fast`
- `cargo nextest run -p fret usage_docs_prefer_grouped_app_ui_actions root_surface_exposes_explicit_style_and_icon_modules root_surface_module_budget_is_curated_and_closed app_prelude_omits_low_level_mechanism_types app_prelude_pub_use_budget_is_curated_and_closed --no-fail-fast`
- `cargo nextest run -p fret --features canvas root_surface_exposes_explicit_style_and_icon_modules root_surface_module_budget_is_curated_and_closed app_prelude_omits_low_level_mechanism_types --no-fail-fast`
- `python3 tools/test_check_surface_policy.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_layering.py`

# Next

Continue U13 with remaining advanced cookbook/example surfaces. `chart_interactions_basics.rs` is
the next public-looking example, but it should stay advanced until chart output/state binding hides
the remaining retained model and chart-canvas seams.

# Citations

- [Phase 3 retained bridge deletion plan](../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md)
- [Canvas facade audit](../subagents/2026-07-04-phase3-u13-canvas-facade-audit.md)
- [Cookbook canvas example](../../../apps/fret-cookbook/examples/canvas_pan_zoom_basics.rs)
- [Fret canvas facade](../../../ecosystem/fret/src/view/canvas.rs)
- [Surface policy gate](../../../tools/check_surface_policy.py)
