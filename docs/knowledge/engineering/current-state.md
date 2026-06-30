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
- Last verified: U4 first observability slice passed `cargo check -p fret-ui`, `cargo check -p fret-bootstrap`, `cargo check -p fret-ui-kit`, focused `cargo nextest` runs for `fret-ui`, `fret-ui-kit`, `fret-diag`, and `fret-bootstrap`, `cargo fmt --all --check`, `python3 tools/check_consumption_profiles.py`, `python3 tools/check_layering.py`, `python3 tools/check_surface_policy.py`, and `git diff --check`.
- Done: local ADR/workstream research, crate/perf snapshots, GPUI/Zed comparison, architecture boundary audit, framework consumer audit, performance audit, implementation-ready plan, U1 convergence contract freeze, U2 source-policy gate, U3 first slice (`workbench-lite` public scaffold), and U4 first observability slice for identity fallback pressure, parent repair, GC reachability, and dispatch snapshot cache pressure.
- In progress: U4 follow-up design for stable identity handles and dirty graph migration gates.
- Blocked: no blocking issue. Runtime implementation is intentionally deferred to follow-on execution.
- Next action: commit the U4 observability slice, then continue U4 with a narrower stable identity/dirty graph migration design using the new counters as evidence.

# Citations

- [Plan](../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md)
- [Subagent Findings](subagents/2026-06-30-ui-framework-architecture-audit-findings.md)
- Commit `020bb34a37 docs(architecture): freeze ui convergence contract`
- Commit `84f60d8355 feat(tools): add ui surface policy gate`
