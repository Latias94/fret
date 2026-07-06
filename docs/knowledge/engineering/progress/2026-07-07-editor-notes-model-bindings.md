---
type: "Work Progress"
title: "Editor notes model bindings"
description: "Work Progress for bundling editor-notes asset and theme model state."
timestamp: 2026-07-07T03:40:00Z
tags: ["fret", "examples", "editor-notes", "public-surface", "raw-model", "binding"]
git_branch: "refactor/editor-notes-model-bindings"
verified_by: "cargo nextest run -p fret-examples --test editor_notes_editor_rail_surface editor_notes_demo_model_state_stays_behind_asset_bindings --no-fail-fast"
---

# Summary

`editor_notes_demo.rs` and `editor_notes_device_shell_demo.rs` now keep editor text and theme
models behind demo-local bindings instead of exposing raw model fields on app-facing view state.

# Details

- Added `EditorAssetModels` to own name, notes, notes-outcome, and summary-status models for each
  asset.
- Added `editor_asset_paint_snapshot(...)` so app and device-shell render paths read those models
  through one named snapshot API.
- Added `EditorThemePresetBinding` for the theme preset picker model.
- Updated `render_inspector_panel(...)` to accept the theme binding and route status writes through
  `EditorAssetModels` methods.
- Strengthened source-shape tests so the editor notes demos cannot regress to raw asset model
  fields, raw theme preset fields, or callback-local status model handles.

# Verification

- `cargo fmt --all --check`
- `cargo check -p fret-examples --lib --tests`
- `cargo nextest run -p fret-examples --test editor_notes_editor_rail_surface editor_notes_demo_composes_shell_mounted_rails_through_workspace_frame_slots editor_notes_demo_model_state_stays_behind_asset_bindings --no-fail-fast`
- `cargo nextest run -p fret-examples --test editor_notes_device_shell_surface editor_notes_device_shell_demo_keeps_shell_switch_explicit_and_reuses_inner_editor_content --no-fail-fast`
- `cargo nextest run -p fret-examples --test imui_editor_workbench_golden_path_surface imui_editor_workbench_demo_is_the_canonical_editor_workbench_route --no-fail-fast`
- `cargo nextest run -p fret-examples editor_notes_model_owner_preserves_text_state_updates --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_layering.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_consumption_profiles.py`
- `python3 ~/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`

# Next Action

Keep `TextField` model-bound until the editor control layer has a dedicated app-facing document or
local-state contract. Continue bundling model graphs at the app/example boundary when a demo owns a
coherent local control surface.

# Citations

- [editor_notes_demo.rs](../../../../apps/fret-examples/src/editor_notes_demo.rs)
- [editor_notes_device_shell_demo.rs](../../../../apps/fret-examples/src/editor_notes_device_shell_demo.rs)
- [editor_notes_editor_rail_surface.rs](../../../../apps/fret-examples/tests/editor_notes_editor_rail_surface.rs)
