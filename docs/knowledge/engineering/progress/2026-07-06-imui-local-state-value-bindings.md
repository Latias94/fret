---
type: Work Progress
title: IMUI LocalState value bindings
timestamp: 2026-07-06T00:00:00Z
git_branch: feat/ui-framework-public-surface-imui-value-local-state-bindings
tags: fret,ui-framework,public-surface,imui,local-state
---

# Summary

The public IMUI text, slider, and combo model surface now accepts app-local `LocalState` handles
through narrow IMUI-specific bridge traits. First-party IMUI examples no longer need to import raw
`LocalState` model bridge APIs for common string, float, optional-string, or boolean controls.

The shadcn adapter demo now uses app-facing `local_state_txn(...)` writes and passes `LocalState`
handles directly into IMUI model controls.

# Decisions

- Keep each bridge type-specific: `IntoImUiTextModel`, `IntoImUiFloatModel`, and
  `IntoImUiOptionalTextModel`.
- Do not introduce a generic app-wide `IntoModel<T>` abstraction while the framework is still
  separating public app surfaces from advanced/raw mechanisms.
- Preserve compatibility for existing `Model<T>`, `&Model<T>`, and `&mut Model<T>` callers.
- Keep low-level IMUI control implementations on `Model<T>` and convert only at public facade
  boundaries.
- Gate the `fret` LocalState adapters behind the `imui` feature.

# Verification

- `cargo check -p fret-ui-kit --features imui`
- `cargo check -p fret-examples-imui --lib --tests`
- `cargo check -p fret`
- `cargo check -p fret --features imui`
- `cargo nextest run -p fret-ui-kit --features imui imui_bool_model_bridge_accepts_existing_model_reference_shapes imui_text_model_bridge_accepts_existing_model_reference_shapes imui_float_model_bridge_accepts_existing_model_reference_shapes imui_optional_text_model_bridge_accepts_existing_model_reference_shapes model_facade_accepts_narrow_imui_model_bridges --no-fail-fast`
- `cargo nextest run -p fret-examples-imui imui_shadcn_adapter_demo_owns_resizable_table_width_state --no-fail-fast`
- `cargo nextest run -p fret data_and_local_state_modules_stay_split_instead_of_regrowing_aggregators --no-fail-fast`
- `cargo nextest run -p fret --features imui root_surface_exposes_explicit_imui_module --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Review Notes

- Read-only correctness, API-contract, and project-standards review agents found no actionable
  issues in the slice.
- API-contract review also ran `cargo check -p fret --no-default-features --lib` successfully.
- API-contract review found an existing `cargo check -p fret --no-default-features --tests` gap in
  test feature assumptions unrelated to this IMUI LocalState binding slice.

# Follow-Up

- Continue deleting raw bridge usage from remaining IMUI examples once the corresponding public
  app-facing surface exists.
- Audit whether the existing no-default-features test helpers should be feature-gated or moved
  behind focused test crates.
