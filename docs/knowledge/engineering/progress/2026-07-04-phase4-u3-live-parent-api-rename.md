---
type: Work Progress
title: Phase 4 U3 live parent API rename
tags: fret,phase4,topology,parent-query,naming
timestamp: 2026-07-04T14:38:04Z
related_plan: ../../../plans/2026-07-04-001-refactor-ui-framework-phase4-topology-epoch-plan.md
git_branch: feat/ui-framework-phase2-refactor
---

# Summary

This Phase 4 U3 sub-slice renamed the internal live parent query from
`parent_in_layer_forest_via_children` to `live_parent_in_layer_forest`.

# Design Finding

After the DFS fallback deletion, the old name was misleading: the normal query no longer scans the
child forest. The new name keeps callers aligned with the contract that live parent lookup comes
from the `LiveTopologyIndex` product and is validated against current child edges.

# Verification

Passed:

- `cargo check -p fret-ui`
- `cargo nextest run -p fret-ui --no-fail-fast --status-level fail`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Next Action

U3 is now structurally complete for normal parent lookup. The next planned frontier is U4 renderer
`FrameAssembler` stream support, unless a final debug-only topology assertion oracle is desired
before entering renderer work.

# Citations

- [Phase 4 topology epoch plan](../../../plans/2026-07-04-001-refactor-ui-framework-phase4-topology-epoch-plan.md)
- [Phase 4 U3 parent DFS fallback deletion](2026-07-04-phase4-u3-parent-dfs-fallback-deletion.md)
