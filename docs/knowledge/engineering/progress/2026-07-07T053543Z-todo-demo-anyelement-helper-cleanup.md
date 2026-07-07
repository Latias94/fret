---
type: "Work Progress"
title: "Todo demo AnyElement helper cleanup"
description: "Work Progress for Todo demo AnyElement helper cleanup."
timestamp: 2026-07-07T05:35:43Z
tags: ["ui-surface", "examples", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/todo-demo-anyelement-cleanup"
---

# Summary

Removed app-facing `AnyElement` helper return signatures from `apps/fret-examples/src/todo_demo.rs`.

# Details

Changed files:

- `apps/fret-examples/src/todo_demo.rs`
- `apps/fret-examples/tests/todo_demo_surface.rs`
- `tools/check_surface_policy.py`
- `tools/test_check_surface_policy.py`

Decision:

- Match the `simple_todo_demo` app-facing helper style: text helpers return `impl UiChild` with
  explicit Rust 2024 capture bounds instead of exposing `AnyElement`.
- Convert to owned elements only at call sites that need a uniform branch type or a component API
  requiring element children, such as `ToggleGroupItem::new(...)`.
- Tighten `todo_demo.rs` policy ownership from `("fret_core", "fret_ui", "AnyElement")` to
  `("fret_core", "fret_ui")`.

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

The remaining `todo_demo.rs` raw policy is now `fret_core`/`fret_ui`; future cleanup should target
the direct `Invalidation` and `DashPatternV1`/core styling imports only if an app-facing wrapper is
worth the abstraction.

# Citations

- `docs/knowledge/engineering/progress/2026-07-07T051123Z-todo-demo-runtime-harness-split.md`
- `apps/fret-examples/tests/todo_demo_surface.rs`
