---
type: Work Progress
title: Components gallery driver model owner cleanup
timestamp: 2026-07-06T00:00:00Z
git_branch: feat/ui-framework-public-surface-components-gallery-state
tags: fret,ui-framework,public-surface,components-gallery,raw-model
---

# Summary

`components_gallery` keeps its explicit shared `Model<T>` graph because it is a mixed retained
render / app-theme / driver-event proof surface, not a simple `View + AppUi` teaching example.

This slice removes the noisy raw `app.models_mut().update(...)` scatter from driver/event call
sites by routing writes through demo-local owner helpers:

- `components_gallery_update_model(...)`
- `components_gallery_set_model(...)`
- `components_gallery_set_last_action(...)`
- `components_gallery_open_command_palette(...)`
- `components_gallery_close_transient_surfaces(...)`

The only remaining direct `models_mut().update(...)` call in `components_gallery.rs` is inside the
generic owner helper. Initial `models_mut().insert(...)` calls remain because this demo owns a
shared model graph.

# Decisions

- Do not convert `components_gallery` state to `LocalState`; prior owner-split audits classify this
  file as retained render + app/theme sync + driver/event owner.
- Keep raw model allocation in `build_ui(...)`; the cleanup target is scattered write plumbing in
  command/hot-reload/event/tree-key handlers.
- Add a source gate that fails if raw `models_mut().update(...)` regrows outside the owner helper.

# Tightening Follow-Up

Branch `refactor/examples-components-gallery-owner-tightening` upgrades the first cleanup from
generic free helpers to a named `ComponentsGalleryModelOwner`.

- Deleted `components_gallery_update_model(...)` and `components_gallery_set_model(...)`.
- Kept semantic helpers for last-action updates, command-palette open, and transient-surface close;
  they now delegate to `ComponentsGalleryModelOwner`.
- Tree keyboard handling, progress commands, and font-reset commands now route generic writes
  through `ComponentsGalleryModelOwner::update(...)` / `set(...)`.
- Tightened the source gate so production source forbids direct/generic/update-any and UFCS
  `ModelStore` bypasses, plus the deleted legacy helper names.
- `tools/check_surface_policy.py` now lists `ModelStore` as an explicit allowed raw seam for the
  components-gallery advanced surface only.
- Added `components_gallery_model_owner_preserves_generic_updates` for owner behavior.

# Verification

- `cargo check -p fret-examples --lib --tests`
- `cargo nextest run -p fret-examples --test components_gallery_surface components_gallery_driver_writes_stay_behind_owner_helpers components_gallery_table_torture_uses_text_roles components_gallery_chrome_and_controls_use_text_roles components_gallery_overlay_text_uses_text_roles --no-fail-fast`
- `cargo nextest run -p fret-examples --test app_import_surface examples_src_keeps_local_state_raw_bridges_out --no-fail-fast`
- `cargo nextest run -p fret-examples --test app_import_surface app_state_demos_use_app_local_state_imports --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Follow-Up

- Continue raw-model shrinkage by choosing another app-facing owner class, likely
  `workspace_shell_demo/*`, before touching plot/chart/custom-effect mechanisms.
- If `components_gallery` needs deeper cleanup later, split it by owner first rather than adding
  more broad helpers to the current file.
