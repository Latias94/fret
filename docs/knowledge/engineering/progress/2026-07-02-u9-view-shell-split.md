---
type: Work Progress
title: U9 AppUi shell split
tags: fret,u9,facade,app-ui,shell,modularity
timestamp: 2026-07-02
related_plan: docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
subagent_id: 019f1ffb-5277-79a2-ac4f-519bddec460c
---

# Summary

`AppUi` moved from `ecosystem/fret/src/view.rs` into
`ecosystem/fret/src/view/shell.rs` together with its core shell helpers:
construction, `elements`, app/window accessors, grouped `state/actions/data/effects`
entrypoints, `keyed`, action-handler registration, raw-model/local-state bridges, local watching,
and cached action-handler extraction.

`view.rs` now acts as the module index, public re-export surface, and source-shape test host. The
public app facade still exposes `view::AppUi` through `pub use shell::AppUi`. `AppUi` fields are
`pub(super)` so sibling modules keep their existing internal access without widening the contract
outside `view`.

# Verification

- `cargo check -p fret --lib`
- `cargo check -p fret --locked --no-default-features --features app`
- `cargo nextest run -p fret --lib app_ui --no-fail-fast`
- `cargo nextest run -p fret --lib grouped_authoring_surfaces_replace_flat_app_ui_helpers --no-fail-fast`
- `cargo nextest run -p fret --test render_authoring_capability_surface --no-fail-fast`
- `cargo nextest run -p fret --test app_render_actions_surface --no-fail-fast`
- `cargo nextest run -p fret --test app_render_data_surface --no-fail-fast`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `cargo fmt --all --check`
- `git diff --check`

# Next Action

Audit whether U9 has remaining documentation/profile-map gaps, then either close the U9 checkpoint
or continue with the next implementation unit in the fearless refactor plan.
