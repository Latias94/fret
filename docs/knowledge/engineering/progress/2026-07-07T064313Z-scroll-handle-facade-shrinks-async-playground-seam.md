---
type: "Work Progress"
title: "Scroll handle facade shrinks async playground seam"
description: "Work Progress for Scroll handle facade shrinks async playground seam."
timestamp: 2026-07-07T06:43:13Z
tags: ["ui-surface", "examples", "facade", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/scroll-handle-facade"
---

# Summary

Added an explicit `fret::scroll::ScrollHandle` facade and used it in `async_playground_demo.rs`,
removing that example's direct `fret_core` seam from the surface policy quarantine.

# Details

Changed files:

- `ecosystem/fret/src/lib.rs`
- `apps/fret-examples/src/async_playground_demo.rs`
- `apps/fret-examples/tests/async_playground_demo_surface.rs`
- `tools/check_surface_policy.py`
- `tools/test_check_surface_policy.py`

Decision:

- Expose `ScrollHandle` under explicit `fret::scroll`, not `fret::app::prelude`.
- Route `SemanticsRole` through `fret::semantics` and `ThemeSnapshot` through `fret::style`.
- Keep `async_playground_demo.rs` in advanced/manual quarantine for now because it still owns raw
  `PressableProps`, `AnyElement`, and `ElementContext` helper boundaries.
- Remove `fret_core` from the async playground allowed raw seams so future direct core imports fail
  the surface policy gate.

# Verification

Passed before commit:

- `cargo fmt --all --check`
- `cargo nextest run -p fret app_and_style_modules_expose_explicit_secondary_app_nouns root_surface_exposes_explicit_style_and_icon_modules app_prelude_pub_use_budget_is_curated_and_closed --no-fail-fast`
- `cargo nextest run -p fret-examples --test async_playground_demo_surface --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- raw seam scan for `async_playground_demo.rs` found no `fret_core::`, `use fret_core::`, or
  `fret_ui::scroll::ScrollHandle` hits.

# Next Action

The remaining async playground quarantine should be retired by adding app-facing pressable and
typed-child helpers, not by widening the default prelude with `AnyElement`.

# Citations

- `apps/fret-examples/src/async_playground_demo.rs`
- `ecosystem/fret/src/lib.rs`
- `tools/check_surface_policy.py`
