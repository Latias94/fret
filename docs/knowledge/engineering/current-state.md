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
- Last verified: U5 first dirty-frontier wrapper slice passed `cargo fmt --all --check`, `cargo check -p fret-ui`, focused `cargo nextest` runs for view-cache/frontier/hover behavior, `python3 tools/check_layering.py`, `python3 tools/check_surface_policy.py`, `python3 tools/check_consumption_profiles.py`, `python3 /Users/frankorz/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`, and `git diff --check`.
- Done: local ADR/workstream research, crate/perf snapshots, GPUI/Zed comparison, architecture boundary audit, framework consumer audit, performance audit, implementation-ready plan, U1 convergence contract freeze, U2 source-policy gate, U3 first slice (`workbench-lite` public scaffold), U4 first observability slice for identity fallback pressure/parent repair/GC reachability/dispatch snapshot cache pressure, U4 second observability slice for dirty frontier breadth plus model/global observation-index churn, and U5 first slice replacing raw `UiTree::dirty_boundaries` ownership with `DirtyViewFrontier`.
- In progress: next narrow U5 follow-on for boundary-owned frame products.
- Blocked: no blocking issue. Runtime implementation is intentionally deferred to follow-on execution.
- Next action: choose the smallest boundary-owned frame-product follow-on without moving dispatch snapshots, paint recording, or scene chunks in the same slice.

# Citations

- [Plan](../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md)
- [Subagent Findings](subagents/2026-06-30-ui-framework-architecture-audit-findings.md)
- Commit `020bb34a37 docs(architecture): freeze ui convergence contract`
- Commit `84f60d8355 feat(tools): add ui surface policy gate`
- Commit `df0d6620ff feat(ui): expose dirty frontier diagnostics`
