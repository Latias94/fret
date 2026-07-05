---
type: Work Progress
title: IMUI interaction showcase LocalState surface
timestamp: 2026-07-06T00:00:00Z
git_branch: feat/ui-framework-public-surface-imui-showcase-local-state
tags: fret,ui-framework,public-surface,imui,local-state
---

# Summary

The IMUI interaction showcase no longer imports raw LocalState bridge APIs. All visible showcase
state writes now use `app.local_state_txn(|tx| ...)`, ordinary reads use `layout_value(...)`, and
IMUI controls/options bind directly to `LocalState` handles.

This completes the current IMUI example cleanup lane: the hello, shadcn adapter, response signals,
floating windows, and interaction showcase examples now keep raw LocalState bridge APIs out of the
teaching/reference call sites.

# Decisions

- Convert helper functions first (`push_showcase_event`, `record_showcase_response`, and
  `set_bool_if_changed`) so event/timeline semantics remain centralized while dropping
  `models()`/`models_mut()` plumbing.
- Use the existing IMUI bool/text/float/optional-text bridge traits for controls.
- Use `kit::TabBarOptions::default().selected_model(&selected_tab)` for tab state.
- Keep this as a call-site cleanup only; lower-level IMUI internals may still store `Model<T>` where
  that is their mechanism contract.

# Verification

- `cargo check -p fret-examples-imui --lib --tests`
- `cargo nextest run -p fret-examples-imui imui_interaction_showcase_demo_uses_local_state_first_bindings showcase_responsive_layout_prefers_two_columns_at_default_window showcase_responsive_layout_stacks_on_narrow_viewports --no-fail-fast`
- `cargo nextest run -p fret root_surface_exposes_explicit_imui_module data_and_local_state_modules_stay_split_instead_of_regrowing_aggregators --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Follow-Up

- Run a broader `rg` over first-party app examples for remaining `advanced::raw` imports that are
  not IMUI-specific and decide whether they are valid advanced/reference seams or should receive
  public app-facing helpers.
