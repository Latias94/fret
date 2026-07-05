---
type: Work Progress
title: Editor notes model owner cleanup
timestamp: 2026-07-06T00:00:00Z
git_branch: feat/ui-framework-public-surface-editor-notes-state
tags: fret,ui-framework,public-surface,editor-notes,raw-model
---

# Summary

`editor_notes_demo` keeps shared `Model<String>` handles because `TextField`, multiline draft
control, and editor rail readouts currently use model-bound editing contracts. This is not a place
for a mechanical `LocalState` rewrite.

This slice removes raw `host.models_mut().update(...)` scatter from editor callback code by routing
all callback writes through demo-local owner helpers:

- `editor_notes_host_update_model(...)`
- `editor_notes_host_set_model(...)`
- `editor_notes_host_set_text(...)`

# Decisions

- Keep model-bound `TextField` state until the editor text-control API has a first-class
  app-facing state binding.
- Keep `models_mut().insert(...)` in asset initialization; the cleanup target is callback write
  plumbing.
- Add a source gate that allows only the owner helper to call `models_mut().update(...)`.

# Verification

- `cargo check -p fret-examples --lib --tests`
- `cargo nextest run -p fret-examples --test editor_notes_editor_rail_surface editor_notes_demo_composes_shell_mounted_rails_through_workspace_frame_slots editor_notes_demo_model_writes_stay_behind_owner_helpers editor_notes_demo_draft_controller_diag_script_clicks_app_owned_commit_and_discard --no-fail-fast`
- `cargo nextest run -p fret-examples --test app_import_surface examples_src_keeps_local_state_raw_bridges_out app_state_demos_use_app_local_state_imports --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Follow-Up

- A future editor-controls contract should decide whether `TextField` accepts `LocalState<String>`
  or a higher-level text document binding. Until then, app demos should hide raw writes behind
  local owner helpers.
