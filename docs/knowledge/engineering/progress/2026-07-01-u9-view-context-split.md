---
type: Work Progress
title: U9 view context split
tags: fret,u9,facade,authoring-surface,modularity
timestamp: 2026-07-01
related_plan: docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Summary

U9 first implementation slice starts the `ecosystem/fret` facade split without changing the public
app authoring surface. `View`, `RenderContextAccess`, and `AppRenderContext` moved from
`ecosystem/fret/src/view.rs` into `ecosystem/fret/src/view/context.rs`; `view.rs` remains the private
aggregator and re-exports those names for the existing root/prelude aliases.

Source-shape tests now include the context module when checking the app-facing render authoring API.
This keeps the tests focused on the public contract instead of requiring every authoring trait to
stay physically inside `view.rs`.

# Subagent Findings

- Poincare confirmed the smallest safe U9 split is to land `view/context.rs` first while preserving
  `fret::app::prelude::*`, `fret::advanced::*`, `FretApp`, and `fret-framework` feature bundles.
- Anscombe found that `tools/check_consumption_profiles.py` still lacks explicit default `fret` and
  `--features batteries` checks, and that `tools/pre_release.py` does not yet run the consumption
  profile gate.

# Changed Files

- `ecosystem/fret/src/view/context.rs`
- `ecosystem/fret/src/view.rs`
- `ecosystem/fret/src/lib.rs`
- `ecosystem/fret/tests/render_authoring_capability_surface.rs`

# Verification

- `cargo nextest run -p fret --test render_authoring_capability_surface --no-fail-fast`
- `cargo nextest run -p fret --test app_render_data_surface --no-fail-fast`
- `cargo nextest run -p fret --test app_render_actions_surface --no-fail-fast`
- `cargo nextest run -p fret --test backend_free_app_authoring_profile --no-fail-fast`
- `cargo nextest run -p fret grouped_authoring_surfaces_replace_flat_app_ui_helpers view_runtime_exposes_only_app_ui_as_the_public_context_name --no-fail-fast`
- `cargo check -p fret --locked --no-default-features --features app`
- `cargo check -p fret --locked`
- `cargo check -p fret --locked --features batteries`
- `cargo check -p fret-framework --locked --no-default-features --features core,runtime,ui`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `cargo fmt --all --check`
- `git diff --check`

# Next Action

Make the U9 consumption-profile gate match the verified evidence: add default `fret` and
`--features batteries` compile checks, include the consumption profile checker in pre-release, and
add a small unit test for banned dependency-tree detection before the next facade split.
