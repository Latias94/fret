---
type: Work Progress
title: Phase 3 U13 examples root internal harness
tags: fret,phase3,u13,source-policy,fret-examples,internal-harness
timestamp: 2026-07-03
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
---

# Summary

Phase 3 U13 fourteenth slice reclassifies `apps/fret-examples/src/lib.rs` as internal harness
infrastructure instead of an `advanced_manual` public-looking migration surface.

# Changes

- Moved `apps/fret-examples/src/lib.rs` from `ADVANCED_MANUAL_SURFACES` to
  `INTERNAL_HARNESS_SURFACES`.
- Kept the same owner and raw seam allowance because the file owns crate-private runner/helper glue.
- Extended source-policy tests so the precise public scan root remains classified, but no longer
  counts as a retirement-tracked advanced manual example.

# Rationale

The examples crate root exports demo modules but its raw seams are crate-private harness helpers:
native/web compat runner wrappers, launch glue, and shared theme interop. It is not a copyable app
surface waiting for public wrappers, so keeping it in `advanced_manual` made the quarantine ledger
less accurate.

# Verification

Passed:

- `python3 tools/test_check_surface_policy.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_layering.py`
- `cargo fmt --all --check`
- `python3 ~/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`

# Next Action

Continue U13 by migrating a real copyable advanced cookbook file, with `virtual_list_basics.rs`
still the highest-value candidate.

# Citations

- [Phase 3 plan](../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md)
- [Router facade cookbook migration](2026-07-03-phase3-u13-router-facade-cookbook-migration.md)
