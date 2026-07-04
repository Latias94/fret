---
type: Work Progress
title: Phase 3 U13 renderer lab classification
tags: fret,phase3,u13,cookbook,renderer-lab,surface-policy
timestamp: 2026-07-04
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
git_branch: feat/ui-framework-phase2-refactor
---

# Summary

Phase 3 U13 now has a dedicated source-policy lane for low-level renderer labs.

- Added `renderer_lab` as a classified raw-surface category in `tools/check_surface_policy.py`.
- Moved `compositing_alpha_basics.rs`, `image_asset_cache_basics.rs`, and
  `customv1_basics.rs` out of `advanced_manual` quarantine and into `RENDERER_LAB_SURFACES`.
- Kept raw seam allowlists and owner requirements for renderer labs, but intentionally removed the
  advanced-manual retirement requirement. These examples are deterministic screenshot baselines for
  renderer/assets semantics, not default app-authoring lessons waiting for facade migration.
- This avoids deleting useful diagnostics evidence while still keeping default cookbook/starter
  surfaces clean.

# Verification

- `python3 tools/test_check_surface_policy.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_layering.py`
- `python3 -m py_compile tools/check_surface_policy.py tools/test_check_surface_policy.py`
- `git diff --check`

# Next

Continue U13 remaining-surface cleanup. `gizmo_basics.rs` still looks like the next migration
candidate, but it needs a scoped facade slice for wheel input, vector/path canvas helpers, and
app-facing local-state/action wiring before it should leave advanced/manual quarantine.

# Citations

- [Phase 3 retained bridge deletion plan](../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md)
- [Surface policy gate](../../../tools/check_surface_policy.py)
- [Cookbook examples index](../../../apps/fret-cookbook/EXAMPLES.md)
- [Example suite inventory](../../../workstreams/example-suite-fearless-refactor-v1/inventory.md)
