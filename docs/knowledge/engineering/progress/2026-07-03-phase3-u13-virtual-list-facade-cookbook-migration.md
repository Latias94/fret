---
type: Work Progress
title: Phase 3 U13 virtual list facade cookbook migration
tags: fret,phase3,u13,virtual-list,cookbook,source-policy
timestamp: 2026-07-03
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
---

# Summary

Phase 3 U13 fifteenth slice moves `virtual_list_basics.rs` from advanced/manual quarantine to the
default cookbook lane by adding explicit style/layout and virtual-list facade exports.

# Changes

- Added `fret::virtual_list::{ItemKey, ScrollStrategy, VirtualListKeyCacheMode,
  VirtualListOptions, VirtualListScrollHandle}`.
- Expanded `fret::style` with explicit low-level layout props used by app-authored containers:
  `ContainerProps`, `LayoutStyle`, `Length`, `Overflow`, `SizeStyle`, and `Edges`.
- Migrated `virtual_list_basics.rs` off direct `fret_ui` and `fret_core` imports.
- Moved `virtual_list_basics.rs` from `ADVANCED_MANUAL_SURFACES` to
  `DEFAULT_AUTHORING_SURFACES`.
- Updated facade/source-policy tests and usage docs so these nouns stay explicit and off
  `fret::app::prelude::*`.

# Rationale

The virtual-list cookbook is a copyable app-facing performance lesson, not a manual runner or
interop harness. Its quarantine reason was low-level import leakage. The new explicit lanes keep
the default prelude small while making virtualization and layout configuration available without
teaching raw `fret_ui` / `fret_core` imports.

# Verification

Passed:

- `cargo check -p fret-cookbook --features cookbook-state --example virtual_list_basics`
- `cargo check -p fret-cookbook --all-targets`
- `cargo nextest run -p fret --lib root_surface_module_budget_is_curated_and_closed root_surface_exposes_explicit_style_and_icon_modules app_prelude_omits_low_level_mechanism_types usage_docs_prefer_grouped_app_ui_actions --no-fail-fast`
- `python3 tools/test_check_surface_policy.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_layering.py`
- `cargo fmt --all --check`
- `python3 ~/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`

Note: the feature compile still reports pre-existing unused-import warnings in `fret-ui-shadcn` and
`fret`; they are not introduced by this slice.

# Next Action

Continue U13 by tackling a remaining true raw bridge, likely `undo_basics.rs` raw action/model
plumbing or one of the IMUI cookbook import/facade gaps.

# Citations

- [Phase 3 plan](../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md)
- [Router facade cookbook migration](2026-07-03-phase3-u13-router-facade-cookbook-migration.md)
