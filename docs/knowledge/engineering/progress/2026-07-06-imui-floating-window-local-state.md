---
type: Work Progress
title: IMUI floating window LocalState surface
timestamp: 2026-07-06T00:00:00Z
git_branch: feat/ui-framework-public-surface-imui-floating-local-state
tags: fret,ui-framework,public-surface,imui,local-state
---

# Summary

The IMUI floating windows proof demo no longer imports raw LocalState model bridges. Floating
window open state, combo selection state, low-level mount reads, and activation writes now stay on
the public LocalState-first surface.

`WindowOptions::with_open(...)` now accepts the narrow `IntoImUiBoolModel` bridge, matching the
boolean control surface while preserving existing `&Model<bool>` callers.

# Decisions

- Keep `WindowOptions` struct-literal compatible; only widen the builder input type.
- Reuse `IntoImUiBoolModel` rather than adding a window-specific bool bridge.
- Use `LocalState::paint(cx).value_or_default()` inside low-level mount content instead of the raw
  `paint_value_in(cx)` bridge.
- Use `host.local_state_txn(|tx| ...)` for activation callbacks that receive `&mut dyn
  UiActionHost`.

# Verification

- `cargo check -p fret-ui-kit --features imui`
- `cargo check -p fret-examples-imui --lib --tests`
- `cargo nextest run -p fret-ui-kit --features imui window_option_builders_compile imui_bool_model_bridge_accepts_existing_model_reference_shapes --no-fail-fast`
- `cargo nextest run -p fret-examples-imui imui_floating_windows_demo_uses_local_state_first_bindings --no-fail-fast`
- `cargo nextest run -p fret root_surface_exposes_explicit_imui_module data_and_local_state_modules_stay_split_instead_of_regrowing_aggregators --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Follow-Up

- `imui_interaction_showcase_demo` is the remaining large IMUI example with raw LocalState bridge
  usage. Migrate it in smaller sections because it still contains many counters, tabs, and helper
  functions.
