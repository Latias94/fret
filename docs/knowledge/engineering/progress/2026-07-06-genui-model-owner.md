---
type: Work Progress
title: GenUI demo model owner cleanup
timestamp: 2026-07-06T00:00:00Z
git_branch: feat/ui-framework-public-surface-genui-state
tags: fret,ui-framework,public-surface,genui,raw-model
---

# Summary

`genui_demo` remains an advanced/reference surface for GenUI catalog, runtime, executor, validation,
and stream-compiler integration. It intentionally owns a shared `Model<Value>` state graph plus
validation and action-queue models, so this is not a candidate for a mechanical `LocalState`
rewrite.

This slice keeps the shared model graph but routes runtime model access through demo-local owner
helpers:

- `genui_update_model(...)`
- `genui_host_update_model(...)`
- `genui_host_read_model(...)`
- `GenUiState::reset_runtime_models(...)`

# Decisions

- Keep `Model<Value>`, `Model<ValidationStateV1>`, and `Model<GenUiActionQueue>` because GenUI
  executor and render integration consume shared runtime state.
- Keep `LocalState` for app-authored controls such as editor text and toggles.
- Add source gates so raw `models_mut().update(...)` and `models_mut().read(...)` do not regrow
  outside owner helpers.

# Tightening Follow-Up

Branch `refactor/examples-genui-owner-tightening` upgrades the first cleanup from app/host free
helpers to a named `GenUiModelOwner`.

- Deleted `genui_update_model(...)`, `genui_host_update_model(...)`, and
  `genui_host_read_model(...)`.
- Added owner-owned `update(...)` and `read(...)` methods.
- Runtime reset and executor handlers now create a local owner from `app.models_mut()` or
  `host.models_mut()` and route shared GenUI model access through it.
- Tightened the source gate so production source forbids direct/generic read/update, update-any,
  UFCS `ModelStore` bypasses, and the deleted legacy helper names.
- Added `genui_model_owner_preserves_runtime_state_read_write` for owner behavior.

# Verification

- `cargo check -p fret-examples --lib --tests`
- `cargo nextest run -p fret-examples --test genui_demo_surface genui_demo_keeps_tool_text_on_roles genui_demo_uses_explicit_public_surfaces genui_demo_model_writes_stay_behind_owner_helpers --no-fail-fast`
- `cargo nextest run -p fret-examples --test app_import_surface examples_src_keeps_local_state_raw_bridges_out app_state_demos_use_app_local_state_imports --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Follow-Up

- Longer term, design a GenUI runtime-state binding facade if multiple apps need this pattern. For
  now, the demo-local owner helpers are enough to prevent default app examples from teaching raw
  model-store access.
