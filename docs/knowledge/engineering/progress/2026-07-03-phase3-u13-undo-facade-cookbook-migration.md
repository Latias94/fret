---
type: Work Progress
title: Phase 3 U13 undo facade cookbook migration
tags: fret,phase3,u13,undo,cookbook,source-policy
timestamp: 2026-07-03
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
---

# Summary

Phase 3 U13 sixteenth slice moves `undo_basics.rs` from advanced/manual quarantine to the default
cookbook lane by converting the example from raw action/model/effect seams to app-facing
`LocalState`, command, style, and semantics lanes.

# Changes

- Added `AppUiLocalsWith::availability::<A>(...)` and the extracted-render equivalent so command
  availability can read `LocalState<T>` through `LocalStateTxn` without naming a raw
  availability host or `ModelStore`.
- Exposed `LocalStateTxn` on the explicit `fret::app` lane for helper signatures while keeping it
  out of `fret::app::prelude::*`.
- Expanded `fret::style` with `FontWeight`.
- Migrated `undo_basics.rs` to `LocalState<T>` handles allocated by `app.local_state(...)`.
- Replaced raw `cx.on_action_notify::<Undo/Redo>` plus `Effect::RequestAnimationFrame` with
  `cx.actions().locals_with(...).on::<A>(...)`; undo/redo now rely on the existing handled-action
  redraw/notify path because this example performs one-shot history traversal, not animation.
- Moved `undo_basics.rs` from `ADVANCED_MANUAL_SURFACES` to `DEFAULT_AUTHORING_SURFACES` and
  tightened cookbook source-shape tests so raw action notify remains only in `async_inbox_basics`.

# Rationale

`undo_basics.rs` is copyable default cookbook material. Its previous advanced classification came
from missing app-facing availability plumbing and an over-conservative RAF effect in one-shot
undo/redo handlers. Keeping the action on `LocalStateTxn` preserves the explicit transaction
boundary while deleting raw host/effect teaching from the public example.

# Verification

Passed:

- `cargo check -p fret-cookbook --features cookbook-undo --example undo_basics`
- `cargo check -p fret-cookbook --all-targets`
- `cargo nextest run -p fret-cookbook --lib cookbook_examples_follow_surface_contracts cookbook_examples_limit_raw_action_notify_to_host_owned_cases --no-fail-fast`
- `cargo nextest run -p fret-cookbook --lib authoring_surface_policy_tests --no-fail-fast`
- `cargo nextest run -p fret --lib root_surface_exposes_explicit_style_and_icon_modules app_prelude_stays_explicit_instead_of_reexporting_legacy_surface app_and_style_modules_expose_explicit_secondary_app_nouns app_prelude_omits_low_level_mechanism_types usage_docs_prefer_grouped_app_ui_actions --no-fail-fast`
- `cargo nextest run -p fret --lib authoring_surface_policy_tests --no-fail-fast`
- `python3 tools/test_check_surface_policy.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_layering.py`
- `cargo fmt --all --check`
- `python3 ~/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`

# Next Action

Continue U13 by addressing the read-only audit findings: migrate IMUI cookbook examples toward
explicit `fret::commands` / `LocalState` facades, or fix the default-clean `data_table_basics.rs`
raw `Model<T>` policy blind spot before tightening the source-policy gate.

# Citations

- [Phase 3 plan](../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md)
- [Readonly U13 cookbook quarantine audit](../subagents/2026-07-03-phase3-u13-cookbook-quarantine-readonly-audit.md)
