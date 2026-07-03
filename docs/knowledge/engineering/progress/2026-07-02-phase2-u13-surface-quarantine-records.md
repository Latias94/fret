---
type: Work Progress
title: Phase 2 U13 surface quarantine records
tags: fret,phase2,u13,source-policy,quarantine,public-facade
timestamp: 2026-07-02
related_plan: docs/plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md
git_branch: feat/ui-framework-phase2-refactor
subagent_id: 019f256e-c3e1-7503-9756-ab0783fc381a
---

# Phase 2 U13 Surface Quarantine Records

## Summary

Phase 2 U13 changes advanced/manual source-policy allowlists from bare path exceptions into owned
quarantine records. An advanced surface now needs an owner, a reason, an explicit list of allowed
raw seams, and a retirement condition. The gate also scans the classified source and fails if a raw
runtime seam appears without being listed in the record.

This keeps public starter surfaces strict while preserving deliberate advanced examples as
temporary, auditable migration records.

## Changes

- Extended `SurfacePath` with `owner`, `allowed_raw_seams`, and `retirement` metadata.
- Updated every `ADVANCED_MANUAL_SURFACES` entry with owner, scoped reason, allowed raw seam list,
  and retirement condition.
- Added raw-seam scanning for advanced/manual surfaces. New uses of `fret_ui`, `fret_core`,
  `fret_app`, `fret_runtime`, `fret_launch`, `fret::advanced`, `AnyElement`, `ElementContext`,
  `FnDriver`, `ModelStore`, `UiActionHostAdapter`, or `UiTree` must be explicitly recorded.
- Added policy tests for missing quarantine metadata, unlisted raw seams, and default starter raw
  advanced imports.
- Incorporated the read-only subagent audit that narrowed the existing records: the
  `api_workbench_lite_demo` reason was stale after public workbench scaffolds landed, and
  `workspace_shell_demo`, `canvas_pan_zoom_basics`, and `node_graph_demo` now describe their actual
  remaining advanced seams.

## Verification

Verification passed before commit:

- `python3 -m unittest tools/test_check_surface_policy.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_layering.py`
- `cargo fmt --all --check`
- `git diff --check`

## Next Action

Continue to U14: split and narrow the `AppUi` facade internals. Use the U13 quarantine records as
the source of truth for deciding whether raw model, element, or host-adapter seams belong in the
default app lane or behind explicit advanced APIs.

# Citations

- [Phase 2 plan](../../../plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md)
- [Surface policy gate](../../../../tools/check_surface_policy.py)
- [Surface policy tests](../../../../tools/test_check_surface_policy.py)
