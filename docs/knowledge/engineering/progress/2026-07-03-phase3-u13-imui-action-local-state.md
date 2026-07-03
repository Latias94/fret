---
type: Work Progress
title: Phase 3 U13 IMUI action local-state migration
tags: fret,phase3,u13,imui,cookbook,source-policy
timestamp: 2026-07-03
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
---

# Summary

Phase 3 U13 eighteenth slice moves `imui_action_basics.rs` from advanced/manual quarantine to the
default cookbook lane by replacing raw runtime/UI seams with explicit app-facing IMUI and command
facades.

# Changes

- Added `fret::imui::AppImUiLocalTextExt` so IMUI text inputs can bind to
  `fret::app::LocalState<String>` without exposing `Model<String>` at the call site.
- Migrated `imui_action_basics.rs` from `fret_runtime::{CommandId, CommandMeta, CommandScope,
  Model}` to explicit `fret::commands` plus `LocalState<String>`.
- Replaced raw `fret_ui::element::ColumnProps` / `imui_raw(...)` usage with an app-facing
  `ui::v_flex(|cx| imui_in(cx, ...))` shape.
- Deleted the mixed GenUI panel from the action example instead of inventing a broad GenUI state
  facade for one cookbook lesson.
- Moved `imui_action_basics.rs` into `DEFAULT_AUTHORING_SURFACES`, tightened source assertions
  against raw runtime/UI/GenUI seams, and updated IMUI teaching docs/gates to describe the new
  `imui_in(...)` + `LocalState` default shape.

# Rationale

The clean default example should teach one idea: typed action dispatch shared by declarative and
IMUI controls. Keeping a GenUI comparison in the same file forced `Model<Value>` and renderer
catalog plumbing into a default lesson. Deleting that mixed panel is the cleaner pre-launch break;
GenUI can keep its own dedicated examples instead of shaping the app facade.

# Verification

Passed:

- `cargo check -p fret-cookbook --features cookbook-imui --example imui_action_basics`
- `cargo check -p fret-cookbook --all-targets`
- `cargo nextest run -p fret-cookbook --lib cookbook_imui_example_keeps_current_facade_teaching_surface migrated_basics_examples_use_the_new_app_surface --no-fail-fast`
- `cargo nextest run -p fret-cookbook --lib cookbook_imui --no-fail-fast`
- `cargo nextest run -p fret --lib root_surface_exposes_explicit_imui_module --no-fail-fast`
- `python3 tools/test_check_surface_policy.py`
- `python3 tools/gate_imui_facade_teaching_source.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_layering.py`
- `python3 tools/check_adr_numbers.py`
- `python3 tools/check_workstream_catalog.py`
- `cargo fmt --all --check`

# Next Action

Continue U13 in three narrow slices:

1. migrate `imui_editor_controls_basics.rs` with editor-control `LocalState<T>` adapters and an
   app-facing color export;
2. remove direct `ColumnProps` from `imui_debug_draw_basics.rs`;
3. add a `fret-plot` plot-specific handle/binding before removing `imui_plot_basics.rs` from
   advanced/manual quarantine.

# Citations

- [Phase 3 plan](../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md)
- [IMUI cookbook facade audits](../subagents/2026-07-03-phase3-u13-imui-cookbook-facade-audit.md)
