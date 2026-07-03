---
type: Work Progress
title: Phase 3 U13 IMUI debug draw host cleanup
tags: fret,phase3,u13,imui,debug-draw,cookbook,source-policy
timestamp: 2026-07-04
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
---

# Summary

Phase 3 U13 twentieth slice removes the remaining low-level layout host leak from
`imui_debug_draw_basics.rs`.

# Changes

- Replaced direct `cx.column(fret_ui::element::ColumnProps::default(), ...)` with an app-facing
  `ui::v_flex(|cx| imui_in(cx, ...))` panel.
- Kept the debug-draw geometry/color vocabulary unchanged because this example is a professional
  drawing facade proof, not a default form/control lesson.
- Tightened cookbook source assertions and the IMUI teaching gate so `ColumnProps` and root
  `imui_raw(...)` do not return to this example.

# Rationale

`imui_debug_draw_basics.rs` was not a true advanced/manual quarantine surface. The raw drawing
commands are the point of the proof; the accidental leak was the low-level layout host needed only
to mount IMUI siblings. Using `imui_in(...)` inside `ui::v_flex(...)` keeps the example on the
app-facing authoring shape without pretending debug drawing is an ordinary component recipe.

# Verification

Passed:

- `cargo check -p fret-cookbook --features cookbook-imui --example imui_debug_draw_basics`
- `cargo check -p fret-cookbook --all-targets`
- `cargo nextest run -p fret-cookbook --lib cookbook_imui_debug_draw_example_keeps_current_facade_teaching_surface --no-fail-fast`
- `cargo nextest run -p fret-cookbook --lib cookbook_imui_debug_draw_example_keeps_current_facade_teaching_surface cookbook_imui migrated_basics_examples_use_the_new_app_surface --no-fail-fast`
- `python3 tools/gate_imui_facade_teaching_source.py`
- `python3 tools/test_check_surface_policy.py`
- `python3 tools/check_surface_policy.py`

# Next Action

Continue U13 by adding a plot-specific handle/binding in `fret-plot` before retiring
`imui_plot_basics.rs` from advanced/manual quarantine. Keep `async_inbox_basics.rs` deferred until
there is an app-facing host/effect action helper for dispatcher wakeups.

# Citations

- [Phase 3 plan](../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md)
- [IMUI cookbook facade audits](../subagents/2026-07-03-phase3-u13-imui-cookbook-facade-audit.md)
