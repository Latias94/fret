---
type: Current State
title: Fret architecture planning current state
tags: fret,architecture,planning
timestamp: 2026-06-30
related_plan: docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
---

# Current State

- Goal: execute the implementation-ready fearless refactor plan for Fret's UI framework architecture convergence.
- Branch: `feat/ui-framework-convergence` from local `main` after the planning commit.
- Last verified: U2 passed `python3 -m unittest tools/test_check_surface_policy.py`, `python3 tools/check_surface_policy.py`, `python3 tools/check_layering.py`, `python3 tools/check_consumption_profiles.py`, `python3 -m py_compile tools/check_surface_policy.py tools/test_check_surface_policy.py`, and `git diff --check`.
- Done: local ADR/workstream research, crate/perf snapshots, GPUI/Zed comparison, architecture boundary audit, framework consumer audit, performance audit, implementation-ready plan, U1 convergence contract freeze, and U2 source-policy gate.
- In progress: U3 second-hour public app ladder.
- Blocked: no blocking issue. Runtime implementation is intentionally deferred to follow-on execution.
- Next action: start U3 by adding/promoting a public `workbench-lite` ladder slice from the API workbench prototype, then add settings dialog and second-hour docs/gates without raw runtime imports.

# Citations

- [Plan](../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md)
- [Subagent Findings](subagents/2026-06-30-ui-framework-architecture-audit-findings.md)
- Commit `020bb34a37 docs(architecture): freeze ui convergence contract`
- Commit `84f60d8355 feat(tools): add ui surface policy gate`
