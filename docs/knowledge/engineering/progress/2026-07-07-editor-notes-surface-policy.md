---
type: "Work Progress"
title: "Editor notes surface policy"
description: "Work Progress for gating editor notes app model/theme state behind EditorAssetModels, EditorNotesModelOwner, and EditorThemePresetBinding."
timestamp: 2026-07-07T01:27:54Z
tags: ["fret", "editor-notes", "examples", "advanced-surface", "source-policy", "raw-model"]
git_branch: "refactor/editor-notes-surface-policy"
verified_by: "PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py"
---

# Summary

`editor_notes_demo.rs` and `editor_notes_device_shell_demo.rs` are now included in public example
scanning and classified as advanced/manual examples. The main editor notes demo now has a global
source-policy gate requiring app model and theme state to stay behind `EditorAssetModels`,
`EditorNotesModelOwner`, and `EditorThemePresetBinding`.

# Details

- Added both editor notes demos to `PUBLIC_EXAMPLE_SCAN_ROOTS`.
- Classified `editor_notes_demo.rs` as the owner of editor app model bindings, shell-mounted rails,
  and theme preset wiring.
- Classified `editor_notes_device_shell_demo.rs` separately because it reuses the main demo's
  asset/theme bindings while owning adaptive shell composition.
- Added an editor notes binding-boundary subcheck to `tools/check_surface_policy.py`.
- Required compact production-source markers for asset model allocation/accessors, owner-mediated
  notes/summary writes, theme preset binding, editor theme picker usage, and paint snapshots.
- Rejected direct production-source `models_mut().update(...)`, `update_any(...)`, UFCS
  `ModelStore::update(...)`, legacy public model fields, legacy theme model fields, old
  `*_model` access patterns, and old free host helper functions.

# Verification

- Red proof before implementation:
  `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise tools.test_check_surface_policy.SurfacePolicyTests.test_editor_notes_direct_model_writes_are_rejected tools.test_check_surface_policy.SurfacePolicyTests.test_editor_notes_binding_surface_is_allowed`
  failed because the demos were not scanned/classified and no binding-boundary violations were
  reported.
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise tools.test_check_surface_policy.SurfacePolicyTests.test_editor_notes_direct_model_writes_are_rejected tools.test_check_surface_policy.SurfacePolicyTests.test_editor_notes_binding_surface_is_allowed`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`

# Next Action

Continue with smaller public-example surface gaps, especially demos that are still absent from
`PUBLIC_EXAMPLE_SCAN_ROOTS` despite owning raw model, launch, viewport, or retained tree seams.

# Citations

- [check_surface_policy.py](../../../../tools/check_surface_policy.py)
- [test_check_surface_policy.py](../../../../tools/test_check_surface_policy.py)
- [editor_notes_demo.rs](../../../../apps/fret-examples/src/editor_notes_demo.rs)
- [editor_notes_device_shell_demo.rs](../../../../apps/fret-examples/src/editor_notes_device_shell_demo.rs)
