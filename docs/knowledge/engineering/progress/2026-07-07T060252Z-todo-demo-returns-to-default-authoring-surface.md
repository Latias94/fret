---
type: "Work Progress"
title: "Todo demo returns to default authoring surface"
description: "Work Progress for Todo demo returns to default authoring surface."
timestamp: 2026-07-07T06:02:52Z
tags: ["ui-surface", "examples", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/todo-style-facade-cleanup"
---

# Summary

Moved `apps/fret-examples/src/todo_demo.rs` back to the default authoring surface by routing its
remaining style/text-decoration nouns through `fret::style`.

# Details

Changed files:

- `ecosystem/fret/src/lib.rs`
- `apps/fret-examples/src/todo_demo.rs`
- `apps/fret-examples/tests/todo_demo_surface.rs`
- `tools/check_surface_policy.py`
- `tools/test_check_surface_policy.py`

Decision:

- Extend `fret::style` with app-facing style/text-decoration vocabulary:
  `DashPatternV1`, `AttributedText`, `DecorationLineStyle`, `StrikethroughStyle`,
  `TextPaintStyle`, and `TextSpan`.
- Keep these out of `fret::app::prelude`; they are explicit secondary style nouns, not first-contact
  app prelude names.
- Move `todo_demo.rs` from `ADVANCED_MANUAL_SURFACES` to `DEFAULT_AUTHORING_SURFACES`.

Verification passed before commit:

- `python3 -m py_compile tools/check_surface_policy.py tools/test_check_surface_policy.py`
- `cargo fmt --all --check`
- `cargo nextest run -p fret app_and_style_modules_expose_explicit_secondary_app_nouns root_header_groups_primary_facades_and_keeps_advanced_out_of_root app_prelude_pub_use_budget_is_curated_and_closed --no-fail-fast`
- `cargo nextest run -p fret-examples todo_demo --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_layering.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_consumption_profiles.py`
- engineering wiki memory validation for `docs/knowledge/engineering`
- `git diff --check`
- raw seam scan for `apps/fret-examples/src/todo_demo.rs` found no `fret_core::`, `fret_ui::`,
  `AnyElement`, `fret::advanced`, `fret_runtime::`, `UiTree`, or `FnDriver` hits.

# Next Action

Repeat this promotion pattern for other default-looking examples: first move raw runtime tests into
internal harnesses, then expose app-facing vocabulary through explicit `fret::*` facades, then move
the example from advanced/manual quarantine to default authoring policy.

# Citations

- `docs/knowledge/engineering/progress/2026-07-07T054918Z-app-prelude-invalidation-facade.md`
- `apps/fret-examples/tests/todo_demo_surface.rs`
