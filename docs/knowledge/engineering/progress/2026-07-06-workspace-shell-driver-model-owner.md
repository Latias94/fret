---
type: Work Progress
title: Workspace shell driver model owner cleanup
timestamp: 2026-07-06T00:00:00Z
git_branch: feat/ui-framework-public-surface-workspace-shell-state
tags: fret,ui-framework,public-surface,workspace-shell,raw-model
---

# Summary

`workspace_shell_demo` intentionally keeps a shared `Model<T>` graph for shell state. Window layout,
dirty-close prompt state, tabstrip pinning, and file-tree state cross render, command, overlay, and
window-close paths. Treating those as one-off view-local state would blur the workspace owner model.

This slice keeps the shared graph but removes scattered raw write plumbing from the command and
overlay handlers. Driver writes now go through demo-local owner helpers:

- `workspace_shell_update_model(...)`
- `workspace_shell_host_update_model(...)`
- `workspace_shell_set_model(...)`
- `workspace_shell_host_set_model(...)`
- `workspace_shell_update_window_layout(...)`
- `workspace_shell_open_dirty_close_prompt(...)`
- `workspace_shell_clear_dirty_close_prompt(...)`
- `workspace_shell_host_clear_dirty_close_prompt(...)`

# Decisions

- Do not migrate `workspace_shell_demo` to `LocalState`; its shell state is shared app/window state,
  not single-view local state.
- Keep `models_mut().insert(...)` during `build_ui(...)`; model allocation is the owner boundary.
- Keep `models_mut().read(...)` in close-policy evaluation and render selectors; the cleanup target
  is scattered writes, not legitimate shared-state observation.
- Add a source gate that allows only the two generic owner helpers to call
  `models_mut().update(...)`.

# Tightening Follow-Up

Branch `refactor/examples-workspace-shell-owner-tightening` upgrades the first cleanup from generic
free helpers to a named `WorkspaceShellModelOwner`.

- Deleted `workspace_shell_update_model(...)`, `workspace_shell_host_update_model(...)`,
  `workspace_shell_set_model(...)`, and `workspace_shell_host_set_model(...)`.
- Kept semantic helpers for window layout and dirty-close prompt operations; they now delegate to
  `WorkspaceShellModelOwner`.
- Added `WorkspaceShellModelOwner::toggle_tabstrip_two_row_pinned(...)` for the tabstrip command
  path.
- Tightened the source gate so production source forbids direct/generic/update-any and UFCS
  `ModelStore` bypasses, plus the deleted legacy helper names.
- `tools/check_surface_policy.py` now lists `ModelStore` as an explicit allowed raw seam for the
  workspace-shell advanced surface only.
- Added `workspace_shell_model_owner_preserves_prompt_and_toggle_updates` for owner behavior.

# Verification

- `cargo check -p fret-examples --lib --tests`
- `cargo nextest run -p fret-examples --test workspace_shell_driver_state_surface workspace_shell_driver_model_writes_stay_behind_owner_helpers --no-fail-fast`
- `cargo nextest run -p fret-examples --test app_import_surface examples_src_keeps_local_state_raw_bridges_out app_state_demos_use_app_local_state_imports --no-fail-fast`
- `cargo nextest run -p fret-examples --test workspace_shell_state_surface --test workspace_shell_editor_rail_surface --test workspace_shell_pane_proof_surface --test workspace_shell_driver_state_surface --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Follow-Up

- Continue raw-model shrinkage in surfaces that expose reusable component APIs, especially
  plot/chart and custom-effect parameter models. Those likely need component-facing binding
  contracts rather than app-demo-local helper wrappers.
