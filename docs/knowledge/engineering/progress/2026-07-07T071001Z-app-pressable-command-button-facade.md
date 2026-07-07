---
type: "Work Progress"
title: "App pressable command button facade"
description: "Work Progress for App pressable command button facade."
timestamp: 2026-07-07T07:10:01Z
tags: ["ui-surface", "examples", "pressable", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/async-text-facade"
---

# Summary

Added `fret::app::pressable::command_button` and migrated the async playground catalog row away
from direct `PressableProps`, `PressableA11y`, and raw pressable command-dispatch hooks.

# Details

Changed files:

- `ecosystem/fret/src/lib.rs`
- `apps/fret-examples/src/async_playground_demo.rs`
- `apps/fret-examples/tests/async_playground_demo_surface.rs`
- `tools/check_surface_policy.py`
- `tools/test_check_surface_policy.py`

Decision:

- Keep `command_button` under explicit `fret::app::pressable`; do not add pressable nouns to
  `fret::app::prelude`.
- Re-export only `PressableState` from this explicit module so app code can style hover/pressed
  states without importing the raw prop bag.
- Keep async playground in advanced/manual quarantine for now because it still uses erased
  `AnyElement` child vectors.

# Verification

Passed before commit:

- `cargo fmt --all --check`
- `cargo nextest run -p fret app_and_style_modules_expose_explicit_secondary_app_nouns app_prelude_pub_use_budget_is_curated_and_closed --no-fail-fast`
- `cargo nextest run -p fret-examples --test async_playground_demo_surface --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- raw seam scan for `async_playground_demo.rs` found no `PressableProps`, `PressableA11y`,
  `pressable_dispatch_command_if_enabled`, `SemanticsRole`, `ElementContext`, `UiHost`, `decl_text`,
  `use fret_core::`, or `fret_core::` hits.

# Next Action

The remaining async playground cleanup is a typed-child/vector cleanup. Avoid solving it by adding
`AnyElement` to the app prelude; prefer an app-facing typed child collection helper.

# Citations

- `ecosystem/fret/src/lib.rs`
- `apps/fret-examples/src/async_playground_demo.rs`
- `tools/check_surface_policy.py`
