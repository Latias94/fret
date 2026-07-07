---
type: "Work Progress"
title: "Async playground default authoring surface"
description: "Work Progress for Async playground default authoring surface."
timestamp: 2026-07-07T07:28:58Z
tags: ["ui-surface", "examples", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/app-element-alias"
---

# Summary

Added the explicit `fret::app::AppElement` alias and moved `async_playground_demo.rs` out of
advanced/manual quarantine into the default app authoring surface.

# Details

Changed files:

- `ecosystem/fret/src/lib.rs`
- `apps/fret-examples/src/async_playground_demo.rs`
- `apps/fret-examples/tests/async_playground_demo_surface.rs`
- `tools/check_surface_policy.py`
- `tools/test_check_surface_policy.py`

Decision:

- Expose `AppElement` as an explicit app helper return alias, not as a default prelude name.
- Keep `AnyElement` off default app source files and policy records.
- Classify `async_playground_demo.rs` as default-clean now that it uses app-facing data, text,
  scroll, pressable, and element aliases.

# Verification

Passed before commit:

- `cargo fmt --all --check`
- `cargo nextest run -p fret app_and_style_modules_expose_explicit_secondary_app_nouns ui_child_alias_uses_unified_component_conversion_trait app_prelude_pub_use_budget_is_curated_and_closed --no-fail-fast`
- `cargo nextest run -p fret-examples --test async_playground_demo_surface --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- raw seam scan for `async_playground_demo.rs` found no `AnyElement`, `fret_ui::element`,
  `fret_ui::`, `fret_core::`, `use fret_core::`, `PressableProps`, `PressableA11y`,
  `ElementContext`, or `UiHost` hits.

# Next Action

Continue promoting examples only when the remaining seam maps to an app-facing alias/helper. Keep
renderer/effect/docking/runtime proofs explicit.

# Citations

- `ecosystem/fret/src/lib.rs`
- `apps/fret-examples/src/async_playground_demo.rs`
- `tools/check_surface_policy.py`
