---
type: Work Progress
title: Phase 3 U13 fret-examples precise quarantine
tags: fret,phase3,u13,fret-examples,source-policy
timestamp: 2026-07-03
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
git_branch: feat/ui-framework-phase2-refactor
---

# Summary

Phase 3 U13 now extends source-policy discovery into selected `apps/fret-examples/src` paths without
classifying the whole examples crate as a broad raw-surface quarantine.

The classified paths are the high-visibility docs/proof/harness surfaces that app authors or
maintainers are most likely to inspect:

- `apps/fret-examples/src/lib.rs`
- `apps/fret-examples/src/api_workbench_lite_demo.rs`
- `apps/fret-examples/src/simple_todo_demo.rs`
- `apps/fret-examples/src/todo_demo.rs`
- `apps/fret-examples/src/components_gallery.rs`
- `apps/fret-examples/src/docking_demo.rs`
- `apps/fret-examples/src/docking_arbitration_demo.rs`
- `apps/fret-examples/src/plot_demo.rs`
- `apps/fret-examples/src/plot_stress_demo.rs`
- `apps/fret-examples/src/gizmo3d_demo.rs`

Each path now has an exact owner, reason, retirement condition, and `allowed_raw_seams` list. The
existing unused-seam check means future migrations must shrink the quarantine records as raw imports
disappear.

# Design Note

`apps/fret-examples` is documented as shared harness code rather than the primary onboarding path,
but several files are referenced from public docs as proof surfaces. The gate therefore scans exact
files instead of adding `apps/fret-examples/src` as a scan root or quarantine path. This keeps
renderer/stress/conformance fixtures from being mislabeled as default authoring while still making
high-visibility raw seams explicit and temporary.

# Verification

Passed on 2026-07-03:

- `python3 tools/test_check_surface_policy.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`

# Remaining U13 Work

The next shrink target is not more broad classification. Prefer one of:

- Split `simple_todo_demo` copyable view code away from web/native runner glue.
- Move `todo_demo` runtime semantics harness support into a test/helper module so the public view
  file can become default-clean.
- Continue cookbook migrations that still use `AppUiRawActionNotifyExt`, `ModelStore`, or raw
  pointer-action hosts.

# Citations

- [Phase 3 plan](../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md)
- [Examples README](../../../apps/fret-examples/README.md)
- [Examples index](../../../docs/examples/README.md)
