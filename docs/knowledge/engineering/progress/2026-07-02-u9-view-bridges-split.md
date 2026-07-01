---
type: Work Progress
title: U9 view bridge impls split
tags: fret,u9,facade,bridges,modularity
timestamp: 2026-07-02
related_plan: docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Summary

The external bridge impls for `AppUi` moved from `ecosystem/fret/src/view.rs` into
`ecosystem/fret/src/view/bridges.rs`: `ElementContextAccess`,
`ElementCommandGatingExt`, and `ElementContextThemeExt`.

`view.rs` now keeps the `AppUi` shell and grouped helper entry points, while bridge forwarding lives
beside the other narrow facade modules. Source-shape tests and render-authoring capability tests now
aggregate `view/bridges.rs`, so the public capability contract follows the module split instead of
assuming all impls stay physically in `view.rs`.

# Verification

- `cargo check -p fret --locked --no-default-features --features app`
- `cargo nextest run -p fret grouped_authoring_surfaces_replace_flat_app_ui_helpers app_ui_keeps_command_gating_and_animation_frame_surface_without_deref --no-fail-fast`
- `cargo nextest run -p fret --test render_authoring_capability_surface --no-fail-fast`
- `cargo nextest run -p fret --test backend_free_app_authoring_profile --no-fail-fast`
- `cargo nextest run -p fret --test raw_state_advanced_surface_docs --no-fail-fast`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `cargo fmt --all --check`
- `git diff --check`

# Next Action

Continue U9 by splitting AppUi shell methods into focused scheduling/layout-query/lane-barrier
modules, or stop U9 once the facade file is small enough for the current plan checkpoint.
