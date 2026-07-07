---
type: "Work Progress"
title: "App prelude invalidation facade"
description: "Work Progress for App prelude invalidation facade."
timestamp: 2026-07-07T05:49:18Z
tags: ["ui-surface", "examples", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/app-prelude-invalidation-facade"
---

# Summary

Added `Invalidation` to the curated `fret::app::prelude` so default app code can use responsive
environment helpers without importing `fret_ui` directly.

# Details

Changed files:

- `ecosystem/fret/src/lib.rs`
- `apps/fret-examples/src/todo_demo.rs`
- `apps/fret-examples/tests/todo_demo_surface.rs`
- `tools/check_surface_policy.py`
- `tools/test_check_surface_policy.py`

Decision:

- Treat `Invalidation` as app-facing responsive-query vocabulary. It is already accepted in the
  component prelude and is required by `fret::env::*` helpers such as `viewport_width_at_least`.
- Keep lower-level mechanism nouns (`UiTree`, `ElementContext`, `AnyElement`) out of
  `fret::app::prelude`; the existing prelude contract test still enforces that budget.
- Move `todo_demo.rs` off direct `fret_ui::Invalidation` import and tighten its policy allowance
  from `("fret_core", "fret_ui")` to `("fret_core",)`.

Verification passed before commit:

- `cargo fmt --all --check`
- `cargo nextest run -p fret app_prelude_pub_use_budget_is_curated_and_closed app_prelude_stays_explicit_instead_of_reexporting_legacy_surface app_prelude_omits_low_level_mechanism_types --no-fail-fast`
- `cargo nextest run -p fret-examples todo_demo --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`

# Next Action

The remaining `todo_demo.rs` public-surface allowance is `fret_core`, mostly because rich text and
low-level style nouns (`AttributedText`, `TextSpan`, `DashPatternV1`) are still imported from
`fret_core`. The next useful slice is to decide whether those belong in `fret::style`, a text
decoration helper, or an explicit advanced styling lane.

# Citations

- `docs/knowledge/engineering/progress/2026-07-07T053543Z-todo-demo-anyelement-helper-cleanup.md`
- `ecosystem/fret/src/lib.rs`
