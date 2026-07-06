---
type: Work Progress
title: Plot binding state API for advanced overlays
timestamp: 2026-07-06T00:00:00Z
git_branch: feat/plot-advanced-state-binding
tags: fret,ui-framework,public-surface,plot,binding,state,raw-model
---

# Summary

`LinePlotPanelBinding` and the other family bindings now support advanced caller-owned plot state
without making app examples store raw runtime model handles:

- `new_with_state(host, model, state)` initializes a binding with caller-provided `PlotState`;
- `read_state_untracked(host, ...)` supports event-time diagnostics and coordination reads;
- `update_state(host, ...)` supports overlay updates and drag/linking follow-ups;
- `linked_member()` creates the advanced coordinator member from the binding's existing state/output
  pair without teaching examples to hand-wire `LinkedPlotMember { state, output }`.

`tags_demo` now demonstrates initial overlay state through `LinePlotPanelBinding::new_with_state`.
`plot_image_demo` now demonstrates dynamic overlay mutation through
`LinePlotPanelBinding::update_state`. `drag_demo` now demonstrates event-time output observation plus
state feedback through `LinePlotPanelBinding::output_untracked(...)` and `update_state(...)`.
`linked_cursor_demo` now demonstrates binding-backed plot coordination through
`LinkedPlotGroup::push_binding(...)`.

# Decision

Do not add raw getters such as `state_model()`, `output_model()`, or `model()` to the app-facing
binding surface. The binding's job is to hide runtime model choreography while still allowing
component-specific state operations. `LinePlotPanelProps::state(...)`, `output(...)`, and
`from_models(...)` remain the advanced/component-author escape hatches.

# Verification

- `cargo nextest run -p fret-plot line_plot_binding_accepts_initial_state_without_public_raw_handles line_plot_binding_updates_state_without_exposing_state_model_handle line_plot_binding_creates_linked_member_without_manual_raw_model_wiring --no-fail-fast`
- `cargo check -p fret-examples --lib --tests`
- `cargo nextest run -p fret-examples --test basic_plot_demos_surface tags_demo_uses_default_declarative_line_plot_panel --no-fail-fast`
- `cargo nextest run -p fret-examples --test basic_plot_demos_surface plot_image_demo_uses_default_declarative_line_plot_panel --no-fail-fast`
- `cargo nextest run -p fret-examples --test basic_plot_demos_surface drag_demo_uses_manual_harness_declarative_line_plot_panel --no-fail-fast`
- `cargo nextest run -p fret-examples --test basic_plot_demos_surface linked_cursor_demo_uses_manual_harness_declarative_top_line_plot_panel --no-fail-fast`

# Next

Consider making `LinkedPlotMember` fields private after all in-tree advanced callers use
`push_binding(...)` or another explicitly named coordinator bridge.
