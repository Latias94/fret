---
type: Work Progress
title: Phase 3 U13 comparison surface classification
tags: fret,phase3,u13,source-policy,fret-examples,comparison
timestamp: 2026-07-03
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
git_branch: feat/ui-framework-phase2-refactor
---

# Summary

Phase 3 U13 source policy now distinguishes temporary `advanced_manual` quarantine records from
longer-lived comparison surfaces.

New tool-internal categories:

- `comparison_surface`: ergonomics, migration, or parity comparisons that may intentionally retain
  raw seams as evidence. These require owner/reason/allowed seam classification but do not require a
  retirement condition.
- `internal_harness`: reserved for stress, conformance, renderer, and maintainer harnesses. The
  category is recognized by the checker so the next slice can classify internal surfaces without
  weakening `advanced_manual` retirement semantics.

`api_workbench_lite_demo.rs` moved from `advanced_manual` to `comparison_surface`, and
`hello_world_compare_demo.rs` plus `imui_editor_proof_demo/authoring_parity` are now exact
comparison-surface records. `PUBLIC_EXAMPLE_SCAN_ROOTS` includes those exact paths, not
`apps/fret-examples/src` as a broad root.

# Verification

Passed on 2026-07-03:

- `python3 tools/test_check_surface_policy.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_layering.py`

# Design Note

The category split follows the read-only explorer audit: `apps/fret-examples` is shared harness
code, while selected files are public proof/comparison surfaces. `advanced_manual` should mean
"temporary until a public wrapper or split lands"; comparison surfaces should not inflate the
bridge-deletion closeout as false debt.

# Citations

- [Phase 3 plan](../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md)
- [`fret-examples` precise quarantine](2026-07-03-phase3-u13-fret-examples-precise-quarantine.md)
