---
type: Work Progress
title: U9 view raw seam split
tags: fret,u9,facade,raw-seam,modularity
timestamp: 2026-07-02
related_plan: docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Summary

The advanced raw escape hatch moved from `ecosystem/fret/src/view.rs` into
`ecosystem/fret/src/view/raw.rs`: `AppUiComponentLaneRequiresExplicitElementsEscapeHatch`,
`AppUiRawModelExt`, and `AppUiRawActionNotifyExt`.

`view.rs` re-exports the same names, keeps the `AppUi` shell and lower-level helper methods in
place, and now aggregates `view/raw.rs` in source-shape tests. This preserves the public advanced
seam while keeping default app code on grouped `state()`, `actions()`, `data()`, and `effects()`
lanes.

# Verification

- `cargo check -p fret --locked --no-default-features --features app`
- `cargo nextest run -p fret grouped_authoring_surfaces_replace_flat_app_ui_helpers raw_model_with_reuses_element_context_local_model_substrate app_ui_keeps_command_gating_and_animation_frame_surface_without_deref --no-fail-fast`
- `cargo nextest run -p fret --test raw_state_advanced_surface_docs --no-fail-fast`
- `cargo nextest run -p fret --test render_authoring_capability_surface --no-fail-fast`
- `cargo nextest run -p fret --test backend_free_app_authoring_profile --no-fail-fast`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `cargo fmt --all --check`
- `git diff --check`

# Next Action

Continue U9 by splitting the remaining `AppUi` shell helpers, command/theme bridge impls, and
layout-query surface into smaller modules without changing public app names.
