---
type: "Work Progress"
title: "Plot overlay drivers leave default authoring surface"
description: "Work Progress for Plot overlay drivers leave default authoring surface."
timestamp: 2026-07-07T06:22:13Z
tags: ["ui-surface", "examples", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/example-surface-policy-next"
---

# Summary

Moved `plot_image_demo` and `tags_demo` launch wiring out of the default authoring source files and
into driver submodules, then promoted the app view files back to the default authoring surface.

# Details

Changed files:

- `apps/fret-examples/src/plot_image_demo.rs`
- `apps/fret-examples/src/plot_image_demo/driver.rs`
- `apps/fret-examples/src/tags_demo.rs`
- `apps/fret-examples/src/tags_demo/driver.rs`
- `apps/fret-examples/tests/basic_plot_demos_surface.rs`
- `tools/check_surface_policy.py`
- `tools/test_check_surface_policy.py`

Decision:

- Keep `build_app`, `build_runner_config`, `build_fn_driver`, and native/web `run` compatibility
  entry points in demo-local driver modules.
- Keep the view source files focused on default declarative app authoring: `fret::app::prelude`,
  `LinePlotPanelBinding`, and declarative plot panel composition.
- Classify `plot_image_demo.rs` and `tags_demo.rs` as default-clean surfaces.
- Classify `plot_image_demo/driver.rs` and `tags_demo/driver.rs` as internal harness surfaces that
  may own the `fret_launch` compatibility seam.

# Verification

Passed before commit:

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples --test basic_plot_demos_surface --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo check -p fret-demo`
- `cargo check -p fret-demo-web --target wasm32-unknown-unknown`

# Next Action

Repeat the same split for other examples whose app-facing view is clean but whose source file still
owns host, web, retained-runtime, or diagnostic launch seams.

# Citations

- `docs/knowledge/engineering/progress/2026-07-07T044141Z-view-demo-launch-helper-migration.md`
- `docs/knowledge/engineering/progress/2026-07-07T060252Z-todo-demo-returns-to-default-authoring-surface.md`
- `tools/check_surface_policy.py`
