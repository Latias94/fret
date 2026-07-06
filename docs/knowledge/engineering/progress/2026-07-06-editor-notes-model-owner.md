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

# Tightening Follow-Up

Branch `refactor/examples-editor-notes-owner-tightening` upgrades the first cleanup from free
functions to a named `EditorNotesModelOwner`.

- Deleted `editor_notes_host_update_model(...)`, `editor_notes_host_set_model(...)`, and
  `editor_notes_host_set_text(...)`.
- Added the semantic owner method `set_text(...)`.
- Action handlers now create a local owner from `host.models_mut()` and route text-status updates
  through `owner.set_text(...)`.
- Tightened the source gate so production source forbids direct/generic/update-any and UFCS
  `ModelStore` bypasses, plus the deleted legacy helper names.
- Added `editor_notes_model_owner_preserves_text_state_updates` for owner behavior.

# Binding Follow-Up

Branch `refactor/editor-notes-model-bindings` keeps the model-bound editor control contract but
moves app-level model choreography behind named bindings.

- Added `EditorAssetModels` so each asset owns name, notes, outcome, and summary-status models
  behind one binding.
- Added `editor_asset_paint_snapshot(...)` for the app and device-shell render paths.
- Added `EditorThemePresetBinding` so the editor theme preset model is not stored as a raw view
  field.
- `render_inspector_panel(...)` now receives the theme binding and writes note outcome / summary
  state through `EditorAssetModels` methods instead of cloning raw status model handles in
  callbacks.

# Verification

- `cargo check -p fret-examples --lib --tests`
- `cargo nextest run -p fret-examples --test editor_notes_editor_rail_surface editor_notes_demo_composes_shell_mounted_rails_through_workspace_frame_slots editor_notes_demo_model_state_stays_behind_asset_bindings editor_notes_demo_draft_controller_diag_script_clicks_app_owned_commit_and_discard --no-fail-fast`
- `cargo nextest run -p fret-examples --test editor_notes_device_shell_surface editor_notes_device_shell_demo_keeps_shell_switch_explicit_and_reuses_inner_editor_content --no-fail-fast`
- `cargo nextest run -p fret-examples --test imui_editor_workbench_golden_path_surface imui_editor_workbench_demo_is_the_canonical_editor_workbench_route --no-fail-fast`
- `cargo nextest run -p fret-examples --test app_import_surface examples_src_keeps_local_state_raw_bridges_out app_state_demos_use_app_local_state_imports --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Follow-Up

- A future editor-controls contract should decide whether `TextField` accepts `LocalState<String>`
  or a higher-level text document binding. Until then, app demos should hide raw model allocation,
  reads, and writes behind local asset/theme bindings.
