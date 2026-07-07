---
type: "Work Progress"
title: "Async playground uses app text facade"
description: "Work Progress for Async playground uses app text facade."
timestamp: 2026-07-07T06:53:46Z
tags: ["ui-surface", "examples", "text", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/async-text-facade"
---

# Summary

Moved `async_playground_demo.rs` text helpers from raw `ElementContext`/`UiHost` and
`fret_ui_kit::declarative::text` calls onto the app-facing `fret::app::text` facade.

# Details

Changed files:

- `apps/fret-examples/src/async_playground_demo.rs`
- `apps/fret-examples/tests/async_playground_demo_surface.rs`
- `tools/check_surface_policy.py`
- `tools/test_check_surface_policy.py`

Decision:

- Keep role-specific local helper names in the demo, but implement them over
  `Cx: AppRenderContext<'a>` and `fret::app::text`.
- Remove `ElementContext` and `UiHost` from the async playground source and from its allowed raw
  seam list.
- Keep the example in advanced/manual quarantine because it still owns direct `PressableProps` and
  `AnyElement` child vectors.

# Verification

Passed before commit:

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples --test async_playground_demo_surface --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- raw seam scan for `async_playground_demo.rs` found no `ElementContext`, `UiHost`, `decl_text`,
  `fret_ui_kit::declarative::text`, `use fret_core::`, or `fret_core::` hits.

# Next Action

Retire the remaining async playground quarantine only after adding an app-facing pressable row or
typed-child collection helper. Do not widen `fret::app::prelude` with `AnyElement`.

# Citations

- `apps/fret-examples/src/async_playground_demo.rs`
- `apps/fret-examples/tests/async_playground_demo_surface.rs`
- `tools/check_surface_policy.py`
