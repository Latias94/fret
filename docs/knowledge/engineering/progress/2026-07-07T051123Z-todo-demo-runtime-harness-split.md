---
type: "Work Progress"
title: "Todo demo runtime harness split"
description: "Work Progress for Todo demo runtime harness split."
timestamp: 2026-07-07T05:11:23Z
tags: ["ui-surface", "examples", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/todo-demo-view-driver-surface"
---

# Summary

Split the raw view-runtime/cache tests out of `apps/fret-examples/src/todo_demo.rs` into the child
test module `apps/fret-examples/src/todo_demo_runtime_tests.rs`.

# Details

Changed files:

- `apps/fret-examples/src/todo_demo.rs`
- `apps/fret-examples/src/todo_demo_runtime_tests.rs`
- `apps/fret-examples/tests/todo_demo_surface.rs`
- `tools/check_surface_policy.py`
- `tools/test_check_surface_policy.py`

Decision:

- Keep the runtime/cache coverage. The tests verify the desktop runner's cache-enable transition
  order and compact resize semantics, so deleting them would lose meaningful regression evidence.
- Move the raw `fret::advanced::view`, `fret_runtime`, and `UiTree` usage into an explicit
  internal harness file. `todo_demo.rs` remains the app-facing demo source and now only owns the
  remaining text/helper return boundaries (`fret_core`, `fret_ui`, `AnyElement`).
- Add source-level tests so raw view-runtime harness code does not move back into the app source.

Verification passed before commit:

- `python3 -m py_compile tools/check_surface_policy.py tools/test_check_surface_policy.py`
- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples todo_demo --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_layering.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_consumption_profiles.py`
- engineering wiki memory validation for `docs/knowledge/engineering`
- `git diff --check`

# Next Action

The remaining `todo_demo.rs` public-surface cleanup is not view-driver related. The next possible
breakable refactor is to remove `AnyElement` from app-facing text helper returns, likely by moving
those helpers to `impl UiChild` or a typed app text wrapper.

# Citations

- `docs/knowledge/engineering/progress/2026-07-07T045347Z-simple-todo-demo-launch-helper-migration.md`
- `apps/fret-examples/src/todo_demo_runtime_tests.rs`
