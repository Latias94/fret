---
type: Work Progress
title: Virtual list stress model owner cleanup
timestamp: 2026-07-06T00:00:00Z
git_branch: feat/ui-framework-public-surface-virtual-list-state
tags: fret,ui-framework,public-surface,virtual-list,raw-model
---

# Summary

`virtual_list_stress_demo` keeps a tiny shared `Model<T>` graph because it is a performance and
diagnostics stress surface. The state is observed by render and mutated by driver events, so this is
not a first-contact `LocalState` teaching example.

This slice keeps the shared state but removes raw write scatter from keyboard handling:

- `virtual_list_stress_update_model(...)`
- `virtual_list_stress_toggle_model(...)`
- `virtual_list_stress_bump_revision(...)`

# Decisions

- Do not migrate the demo to `LocalState`; it is a driver/runtime stress harness.
- Keep `models_mut().insert(...)` in `build_ui(...)` for the owned shared state.
- Add a source gate that allows only the owner helper to call `models_mut().update(...)`.

# Verification

- `cargo check -p fret-examples --lib --tests`
- `cargo nextest run -p fret-examples --test virtual_list_stress_demo_surface virtual_list_stress_demo_keeps_fixed_row_text_on_roles virtual_list_stress_demo_model_writes_stay_behind_owner_helpers --no-fail-fast`
- `cargo nextest run -p fret-examples --test app_import_surface examples_src_keeps_local_state_raw_bridges_out app_state_demos_use_app_local_state_imports --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Follow-Up

- Continue with app-facing examples that can be cleaned locally. Avoid mechanical rewrites for
  plot/chart/custom-effect demos until their component binding contracts are designed.
