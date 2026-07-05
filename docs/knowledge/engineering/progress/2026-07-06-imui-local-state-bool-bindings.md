---
type: Work Progress
title: IMUI LocalState bool bindings
timestamp: 2026-07-06T00:00:00Z
git_branch: feat/ui-framework-public-surface-imui-local-state-bindings
tags: fret,ui-framework,public-surface,imui,local-state
---

# Summary

The public IMUI boolean model surface now accepts app-local `LocalState<bool>` handles through a
narrow `IntoImUiBoolModel` bridge instead of forcing first-party examples to import raw
`LocalState` model bridges.

The smallest IMUI hello demo now increments and reads view-local state through app-facing
`local_state_txn(...)`, `paint_value(...)`, and `checkbox_model("Enabled", &enabled_state)`.

# Decisions

- Keep the bridge narrow and IMUI-specific. Do not introduce a crate-wide `IntoModel<T>` story.
- Keep the underlying boolean control implementation on `Model<bool>` for now. Convert only at the
  public facade boundary.
- Gate the `fret` LocalState adapter behind the `imui` feature so the default `fret` facade does
  not depend on the optional IMUI module.
- Preserve compatibility for old `&Model<bool>` callers, including `&mut Model<bool>` call sites
  that previously coerced to a shared borrow.

# Verification

- `cargo check -p fret-ui-kit --features imui`
- `cargo check -p fret-examples-imui --lib --tests`
- `cargo check -p fret`
- `cargo check -p fret --features imui`
- `cargo nextest run -p fret-ui-kit --features imui imui_bool_model_bridge_accepts_existing_model_reference_shapes boolean_model_facade_accepts_narrow_imui_model_bridge --no-fail-fast`
- `cargo nextest run -p fret-examples-imui imui_hello_demo_uses_local_state_first_bindings --no-fail-fast`
- `cargo nextest run -p fret data_and_local_state_modules_stay_split_instead_of_regrowing_aggregators --no-fail-fast`
- `cargo nextest run -p fret --features imui root_surface_exposes_explicit_imui_module --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Review Notes

- Read-only correctness and project-standards subagents found no actionable issues.
- Read-only API-contract review found the missing `&mut Model<bool>` compatibility case; the final
  bridge and compile test now cover it.

# Follow-Up

- Remaining IMUI examples still use raw `LocalState` bridge calls for text, slider, combo, and
  bespoke resize state. Continue by adding equally narrow IMUI value/text model bridges before
  migrating those examples.
