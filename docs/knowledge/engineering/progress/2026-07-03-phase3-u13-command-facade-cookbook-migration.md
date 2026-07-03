---
type: Work Progress
title: Phase 3 U13 command facade cookbook migration
tags: fret,phase3,u13,commands,cookbook,source-policy
timestamp: 2026-07-03
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
---

# Summary

Phase 3 U13 twelfth slice adds an explicit `fret::commands` app-facing lane and migrates the
lowest-risk command/semantics cookbook examples off raw `fret_app`, `fret_core`, and `fret_ui`
imports.

# Changes

- Added `fret::commands::{...}` for command identity, metadata, availability, keybindings, keymap
  lookup, shortcut formatting, and key input value types.
- Migrated `commands_keymap_basics.rs`, `form_basics.rs`, and `text_input_basics.rs` to use
  `fret::commands` and `fret::semantics`.
- Moved those three cookbook files from `ADVANCED_MANUAL_SURFACES` to
  `DEFAULT_AUTHORING_SURFACES`.
- Updated facade policy tests and source-policy tests so command/keymap cookbook examples cannot
  drift back to raw app/runtime/UI imports.
- Updated usage docs to point command registration and availability examples at `fret::commands`.

# Rationale

The affected examples are copyable default cookbook material. They were quarantined only because
the facade lacked a named command/keymap lane, not because the examples are inherently advanced.
Keeping command registration on `fret::commands` preserves the curated default prelude while
removing public-looking raw crate imports.

# Verification

Passed:

- `cargo nextest run -p fret --lib root_surface_module_budget_is_curated_and_closed root_surface_exposes_explicit_style_and_icon_modules app_prelude_omits_low_level_mechanism_types usage_docs_prefer_grouped_app_ui_actions usage_docs_expose_curated_component_surface --no-fail-fast`
- `cargo check -p fret-cookbook --all-targets`
- `python3 tools/test_check_surface_policy.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_layering.py`
- `cargo fmt --all --check`
- `python3 ~/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`

# Next Action

Continue U13 by migrating another narrow cookbook cluster, likely `virtual_list_basics.rs` through
an explicit virtual-list facade lane or a tighter advanced/manual classification if it proves to be
mechanism documentation rather than default app material.

# Citations

- [Phase 3 plan](../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md)
- [Internal harness classification](2026-07-03-phase3-u13-internal-harness-classification.md)
