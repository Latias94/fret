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
- Last verified: U3 first slice passed `cargo fmt --all --check`, `cargo nextest run -p fretboard scaffold`, `python3 tools/check_surface_policy.py`, `python3 tools/check_layering.py`, `python3 tools/check_consumption_profiles.py`, and `git diff --check`.
- Done: local ADR/workstream research, crate/perf snapshots, GPUI/Zed comparison, architecture boundary audit, framework consumer audit, performance audit, implementation-ready plan, U1 convergence contract freeze, U2 source-policy gate, and U3 first slice (`workbench-lite` public scaffold).
- In progress: U4 identity and dirty graph observability planning.
- Blocked: no blocking issue. Runtime implementation is intentionally deferred to follow-on execution.
- Next action: start U4 with counters/diagnostics for identity fallback scans, parent pointer repair, GC reachability, dispatch snapshot cache, observation churn, and dirty frontier breadth before introducing `StableNodeHandle`.

# Citations

- [Plan](../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md)
- [Subagent Findings](subagents/2026-06-30-ui-framework-architecture-audit-findings.md)
- Commit `020bb34a37 docs(architecture): freeze ui convergence contract`
- Commit `84f60d8355 feat(tools): add ui surface policy gate`
