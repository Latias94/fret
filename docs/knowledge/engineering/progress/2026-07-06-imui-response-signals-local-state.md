---
type: Work Progress
title: IMUI response signals LocalState surface
timestamp: 2026-07-06T00:00:00Z
git_branch: feat/ui-framework-public-surface-imui-raw-cleanup
tags: fret,ui-framework,public-surface,imui,local-state
---

# Summary

The IMUI response signals proof demo now stays on the app-facing LocalState surface for its
ordinary state updates and model bindings. It no longer imports the raw LocalState model bridge or
reopens `.model()` handles for common IMUI controls.

Tab selection now has a narrow `TabBarOptions::selected_model(...)` builder so app code can bind a
`LocalState<Option<Arc<str>>>` handle without making `TabBarOptions` generic or exposing a raw
`Model<Option<Arc<str>>>` handle at the call site.

# Decisions

- Use `app.local_state_txn(|tx| ...)` for immediate-mode event response updates that receive
  `&mut App`.
- Keep `TabBarOptions` struct-literal compatible by adding builder methods instead of changing the
  public field type.
- Reuse the existing `IntoImUiOptionalTextModel` bridge for tab selection because its runtime value
  shape matches combo model selection.
- Keep raw `Model<T>` fields available inside lower-level options structs for now; remove raw
  call-site pressure first, then revisit options internals as a separate API cleanup.

# Verification

- `cargo check -p fret-ui-kit --features imui`
- `cargo check -p fret-examples-imui --lib --tests`
- `cargo nextest run -p fret-ui-kit --features imui tab_bar_options_accepts_narrow_imui_selected_model_bridge model_facade_accepts_narrow_imui_model_bridges --no-fail-fast`
- `cargo nextest run -p fret-examples-imui imui_response_signals_demo_uses_local_state_first_bindings --no-fail-fast`
- `cargo nextest run -p fret root_surface_exposes_explicit_imui_module data_and_local_state_modules_stay_split_instead_of_regrowing_aggregators --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Follow-Up

- Migrate `imui_floating_windows_demo` open/select model call sites after adding the missing narrow
  open-state options builders.
- Migrate `imui_interaction_showcase_demo` in smaller sections; it still has many response counters
  and tab state updates using raw bridge APIs.
