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
- Last verified: U5 BoundaryFrameProducts slice passed `cargo fmt --all --check`, `cargo check -p fret-ui --all-targets`, 13 focused `cargo nextest -p fret-ui` tests covering boundary frame products, canvas prepaint/scene-fragment carriers, prepaint output ownership, paint-cache boundary ownership, view-cache contained relayout, dispatch snapshot cache, hit-test, outside-press, and modal barrier behavior, `python3 tools/check_layering.py`, `python3 tools/check_surface_policy.py`, `python3 tools/check_consumption_profiles.py`, and `git diff --check`.
- Done: local ADR/workstream research, crate/perf snapshots, GPUI/Zed comparison, architecture boundary audit, framework consumer audit, performance audit, implementation-ready plan, U1 convergence contract freeze, U2 source-policy gate, U3 first slice (`workbench-lite` public scaffold), U4 first observability slice for identity fallback pressure/parent repair/GC reachability/dispatch snapshot cache pressure, U4 second observability slice for dirty frontier breadth plus model/global observation-index churn, U5 first slice replacing raw `UiTree::dirty_boundaries` ownership with `DirtyViewFrontier`, and U5 BoundaryFrameProducts slice grouping boundary dirty/prepaint/scene-fragment/paint-cache state under `ViewBoundaryState::frame_products`.
- In progress: choose the next narrow U5 frame-product follow-on.
- Blocked: no blocking issue. Runtime implementation is intentionally deferred to follow-on execution.
- Next action: choose the smallest remaining boundary-owned frame-product follow-on without moving dispatch snapshots, paint recording, or scene chunks in the same slice.

# Citations

- [Plan](../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md)
- [Subagent Findings](subagents/2026-06-30-ui-framework-architecture-audit-findings.md)
- Commit `020bb34a37 docs(architecture): freeze ui convergence contract`
- Commit `84f60d8355 feat(tools): add ui surface policy gate`
- Commit `df0d6620ff feat(ui): expose dirty frontier diagnostics`
- Commit `09debbceae refactor(ui): move dirty frontier behind view ids`
- Commit `ac1aa7ba27 refactor(ui): group boundary frame products`
