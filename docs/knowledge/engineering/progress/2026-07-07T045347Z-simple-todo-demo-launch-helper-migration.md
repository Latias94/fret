---
type: "Work Progress"
title: "Simple todo demo launch helper migration"
description: "Work Progress for Simple todo demo launch helper migration."
timestamp: 2026-07-07T04:53:47Z
tags: ["ui-surface", "examples", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/default-view-demo-helper-rollout"
---

# Summary

Moved `simple_todo_demo/driver.rs` onto the shared default-view demo launch helpers added in the
previous slice.

# Details

Changed files:

- `apps/fret-examples/src/simple_todo_demo/driver.rs`
- `apps/fret-examples/tests/simple_todo_demo_surface.rs`
- `tools/check_surface_policy.py`
- `tools/test_check_surface_policy.py`

Decision:

- Keep demo-specific app setup (`install_demo_icons`, `install_demo_theme`) in the driver, after
  `crate::build_default_view_demo_app()`.
- Reuse `crate::build_default_view_demo_runner_config(...)` and
  `crate::build_default_view_demo_fn_driver::<SimpleTodoView>(...)` for the web/native demo shell
  launch parts.
- Tighten `simple_todo_demo/driver.rs` policy ownership so it only allows the remaining
  `fret_launch` signature seam.

Verification passed before commit:

- `python3 -m py_compile tools/check_surface_policy.py tools/test_check_surface_policy.py`
- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples --test simple_todo_demo_surface --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo check -p fret-examples --target wasm32-unknown-unknown`
- `cargo check -p fret-demo-web --target wasm32-unknown-unknown`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_layering.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_consumption_profiles.py`
- engineering wiki memory validation for `docs/knowledge/engineering`
- `git diff --check`

# Next Action

Audit the remaining `view_init_window/view_view` usage. After this slice it should be limited to
the shared helper plus `todo_demo.rs`, which needs a separate decision because it carries much more
legacy compatibility/test surface than the simple demo driver.

# Citations

- `docs/knowledge/engineering/progress/2026-07-07T044141Z-view-demo-launch-helper-migration.md`
- `apps/fret-examples/src/simple_todo_demo/driver.rs`
