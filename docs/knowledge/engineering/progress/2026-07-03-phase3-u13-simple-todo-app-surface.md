---
type: Work Progress
title: Phase 3 U13 simple todo app surface shrink
tags: fret,phase3,u13,fret-examples,app-facade,source-policy
timestamp: 2026-07-03
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
---

# Summary

Phase 3 U13 ninth slice shrinks the `simple_todo_demo` source-policy quarantine by moving its
view-body helper seams onto the app-facing facade.

# Changes

- `fret::semantics` now explicitly re-exports `SemanticsDecoration` next to `SemanticsRole`, keeping
  advanced semantics nouns off `fret::app::prelude::*` while avoiding direct app imports from
  `fret_ui`.
- `apps/fret-examples/src/simple_todo_demo.rs` no longer imports `fret_core`, `fret_ui`,
  `AnyElement`, `ElementContext`, or `UiHost` for its view helpers.
- Simple todo text helpers now accept `AppRenderContext` and return app-facing `UiChild` opaque
  values. Row text accepts `ColorRef` and resolves foreground through the app render context.
- `tools/check_surface_policy.py` shrinks the simple todo advanced/manual record to launch/runtime
  glue only: `fret::advanced`, `fret_launch`, and `fret_runtime`.

# Verification

Passed:

- `cargo nextest run -p fret --lib root_surface_exposes_explicit_style_and_icon_modules app_prelude_exports_are_curated_for_default_app_authors app_prelude_pub_use_budget_is_curated_and_closed app_and_style_modules_expose_explicit_secondary_app_nouns --no-fail-fast`
- `python3 tools/test_check_surface_policy.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_layering.py`
- `python3 ~/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `cargo fmt --all --check`
- `git diff --check`

Known broader blocker:

- `cargo check -p fret-examples --lib` still fails in unrelated example files because the advanced
  raw local-state traits were removed from `advanced::prelude::*` and those examples have not yet
  imported the needed explicit `advanced::raw` traits or migrated off the raw helpers. A filtered
  rerun showed no remaining `simple_todo_demo.rs` errors.

# Next Action

Continue U13 by either splitting `simple_todo_demo` runner glue from the copyable view module, or
by migrating the broader `fret-examples` files that still rely on raw local-state helper traits.

# Citations

- [Phase 3 plan](../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md)
- [`fret-examples` precise quarantine](2026-07-03-phase3-u13-fret-examples-precise-quarantine.md)
- [Comparison surface classification](2026-07-03-phase3-u13-comparison-surface-classification.md)
