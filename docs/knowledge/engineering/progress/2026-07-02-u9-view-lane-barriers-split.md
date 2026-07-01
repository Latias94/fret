---
type: Work Progress
title: U9 view lane barriers split
tags: fret,u9,facade,lane-barriers,modularity
timestamp: 2026-07-02
related_plan: docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Summary

The default app lane-sealing barrier methods moved from `ecosystem/fret/src/view.rs` into
`ecosystem/fret/src/view/lane_barriers.rs`.

These hidden methods intentionally remain callable-but-unusable on `AppUi` so method resolution does
not fall through to lower-level `ElementContext` helpers. The public contract and unreachable
messages are unchanged, and source-shape tests aggregate the new module.

# Verification

- `cargo check -p fret --locked --no-default-features --features app`
- `cargo nextest run -p fret grouped_authoring_surfaces_replace_flat_app_ui_helpers app_ui_keeps_command_gating_and_animation_frame_surface_without_deref --no-fail-fast`
- `cargo nextest run -p fret --test render_authoring_capability_surface --no-fail-fast`
- `cargo nextest run -p fret --test backend_free_app_authoring_profile --no-fail-fast`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `cargo fmt --all --check`
- `git diff --check`

# Next Action

Continue U9 by deciding whether to move the remaining core `AppUi` shell helpers into a final
`view/shell.rs`, or stop once the facade file is small enough for the current checkpoint.
