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
- Last verified: U5 hit-test path routing cache owner-state slice passed `cargo fmt --all --check`, `cargo check -p fret-ui --all-targets`, focused `fret-ui` hit-test cache/stale-path/prepaint/pointer-occlusion/modal-barrier/outside-press nextest gates, `git diff --check`, `python3 tools/check_layering.py`, `python3 tools/check_surface_policy.py`, and `python3 tools/check_consumption_profiles.py`.
- Done: local ADR/workstream research, crate/perf snapshots, GPUI/Zed comparison, architecture boundary audit, framework consumer audit, performance audit, implementation-ready plan, U1 convergence contract freeze, U2 source-policy gate, U3 first slice (`workbench-lite` public scaffold), U4 first observability slice for identity fallback pressure/parent repair/GC reachability/dispatch snapshot cache pressure, U4 second observability slice for dirty frontier breadth plus model/global observation-index churn, U5 first slice replacing raw `UiTree::dirty_boundaries` ownership with `DirtyViewFrontier`, U5 BoundaryFrameProducts slice grouping boundary dirty/prepaint/scene-fragment/paint-cache state under `ViewBoundaryState::frame_products`, U5 interaction replay entry slice removing `Node::interaction_cache` in favor of `BoundaryFrameProducts::interaction_cache`, U5 dispatch snapshot pre-slice moving raw `UiTree` dispatch snapshot generation/cache fields behind `DispatchSnapshotFrameProductState`, U5 hit-test bounds tree slice moving per-layer reusable bounds indexes into `BoundaryFrameProducts::hit_test_bounds` while keeping build/query scratch in `HitTestBoundsTrees`, U5 semantics subtree slice moving reusable clean boundary semantics subtree products into `BoundaryFrameProducts::semantics` while preserving final window-level `SemanticsSnapshot` ownership, and U5 hit-test path routing cache slice replacing raw `Option<HitTestPathCache>` ownership with `HitTestPathRoutingCacheState` as a window-owned input-routing state.
- In progress: continue U5 ownership convergence after deciding that hit-test path routing cache is not boundary-owned.
- Blocked: no blocking issue. Runtime implementation is intentionally deferred to follow-on execution.
- Next action: choose the next U5 owner-state or boundary-owned frame product. Read-only audits concluded input/dispatch snapshots and hit-test path routing cache should stay window/layer-forest/input-routing owned; the next plausible slice is a command routing correctness pre-slice using `UiDispatchSnapshot.parent`, or another narrow boundary product only after a read-only ownership audit proves it is truly per-boundary.

# Citations

- [Plan](../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md)
- [Subagent Findings](subagents/2026-06-30-ui-framework-architecture-audit-findings.md)
- Commit `020bb34a37 docs(architecture): freeze ui convergence contract`
- Commit `84f60d8355 feat(tools): add ui surface policy gate`
- Commit `df0d6620ff feat(ui): expose dirty frontier diagnostics`
- Commit `09debbceae refactor(ui): move dirty frontier behind view ids`
- Commit `ac1aa7ba27 refactor(ui): group boundary frame products`
- Commit `8572864d49 refactor(ui): move interaction replay entries to boundaries`
- Commit `51551ae554 refactor(ui): name dispatch snapshot frame product state`
- Subagent `019f17f1-0549-72a3-abba-e3108c26da81` hit-test bounds ownership audit
- Subagent `019f181e-fe0a-72c0-ba7c-79cf3a60ccf3` input/dispatch snapshot ownership audit
- Subagent `019f181f-9af6-7741-93ae-7c5dd821f1fb` semantics subtree ownership audit
- Subagent `019f1853-0640-79d2-8e19-8411b52ef258` hit-test path cache ownership audit
