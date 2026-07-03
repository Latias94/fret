---
type: Work Progress
title: Phase 3 U13 IMUI editor local-state migration
tags: fret,phase3,u13,imui,editor,cookbook,source-policy
timestamp: 2026-07-03
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
---

# Summary

Phase 3 U13 nineteenth slice moves `imui_editor_controls_basics.rs` from advanced/manual
quarantine to the default cookbook lane by adding narrow app-facing `LocalState<T>` constructors for
editor controls.

# Changes

- Added `fret::style::Color` as an explicit style noun while keeping it out of
  `fret::app::prelude::*`.
- Converted `fret::imui::editor::controls` from a pure external re-export into a re-export module
  with `LocalState` adapter traits:
  - `NumericInputLocalStateExt`
  - `DragValueLocalStateExt`
  - `ColorEditLocalStateExt`
  - `MiniSearchBoxLocalStateExt`
  - `TextAssistFieldLocalStateExt`
- Migrated `imui_editor_controls_basics.rs` from raw `Model<T>` fields and
  `app.models_mut().insert(...)` to `LocalState<T>` plus `app.local_state(...)`.
- Replaced root `imui_raw(...)` use with an app-facing `ui::v_flex(|cx| imui_in(cx, ...))` panel.
- Moved `imui_editor_controls_basics.rs` into `DEFAULT_AUTHORING_SURFACES` and tightened source
  assertions so raw `fret_core`, `fret_runtime`, `Model<...>`, and old editor constructors do not
  return.

# Rationale

The editor controls themselves still correctly own raw `Model<T>` internally. The default app lane
should not expose that storage model to cookbook readers. A narrow adapter layer in `fret` preserves
the lower crate boundary and avoids a broad `IntoModel<T>` abstraction.

# Verification

Passed:

- `cargo check -p fret-cookbook --features cookbook-imui --example imui_editor_controls_basics`
- `cargo check -p fret-cookbook --all-targets`
- `cargo nextest run -p fret-cookbook --lib cookbook_imui_editor_example_keeps_public_editor_facade_teaching_surface migrated_basics_examples_use_the_new_app_surface --no-fail-fast`
- `cargo nextest run -p fret-cookbook --lib cookbook_imui migrated_basics_examples_use_the_new_app_surface --no-fail-fast`
- `cargo nextest run -p fret --lib root_surface_exposes_explicit_imui_module root_surface_exposes_explicit_style_and_icon_modules app_and_style_modules_expose_explicit_secondary_app_nouns app_prelude_omits_low_level_mechanism_types --no-fail-fast`
- `python3 tools/test_check_surface_policy.py`
- `python3 tools/gate_imui_facade_teaching_source.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_layering.py`

# Next Action

Continue U13 by removing direct `fret_ui::element::ColumnProps` from
`imui_debug_draw_basics.rs`, then add a plot-specific handle/binding in `fret-plot` before moving
`imui_plot_basics.rs` out of advanced/manual quarantine.

# Citations

- [Phase 3 plan](../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md)
- [IMUI cookbook facade audits](../subagents/2026-07-03-phase3-u13-imui-cookbook-facade-audit.md)
