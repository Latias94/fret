---
type: Current State
title: Fret architecture planning current state
tags: fret,architecture,planning
timestamp: 2026-07-01
related_plan: docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
---

# Current State

- Goal: execute the implementation-ready fearless refactor plan for Fret's UI framework architecture convergence.
- Branch: `feat/ui-framework-convergence` from local `main` after the planning commit.
- Last verified: U6 root resizable chrome export slice passed source-policy unit/gate, `cargo fmt --all --check`, `cargo check` for `fret-ui`, `fret-ui-kit`, `fret-ui-shadcn`, and `fret-workspace`, focused `fret-ui` and `fret-ui-shadcn` resizable nextest gates, layering, consumption profiles, module-size guard, root-import search, engineering memory validation, and `git diff --check`.
- Done: local ADR/workstream research, crate/perf snapshots, GPUI/Zed comparison, architecture boundary audit, framework consumer audit, performance audit, implementation-ready plan, U1 convergence contract freeze, U2 source-policy gate, U3 first slice (`workbench-lite` public scaffold), U4 first observability slice for identity fallback pressure/parent repair/GC reachability/dispatch snapshot cache pressure, U4 second observability slice for dirty frontier breadth plus model/global observation-index churn, U5 first slice replacing raw `UiTree::dirty_boundaries` ownership with `DirtyViewFrontier`, U5 BoundaryFrameProducts slice grouping boundary dirty/prepaint/scene-fragment/paint-cache state under `ViewBoundaryState::frame_products`, U5 interaction replay entry slice removing `Node::interaction_cache` in favor of `BoundaryFrameProducts::interaction_cache`, U5 dispatch snapshot pre-slice moving raw `UiTree` dispatch snapshot generation/cache fields behind `DispatchSnapshotFrameProductState`, U5 hit-test bounds tree slice moving per-layer reusable bounds indexes into `BoundaryFrameProducts::hit_test_bounds` while keeping build/query scratch in `HitTestBoundsTrees`, U5 semantics subtree slice moving reusable clean boundary semantics subtree products into `BoundaryFrameProducts::semantics` while preserving final window-level `SemanticsSnapshot` ownership, U5 hit-test path routing cache slice replacing raw `Option<HitTestPathCache>` ownership with `HitTestPathRoutingCacheState` as a window-owned input-routing state, U5 command routing correctness slice making pending command source starts plus command dispatch/availability bubbling use `UiDispatchSnapshot` membership and parent maps instead of retained parent pointers, U5 command routing cache owner-state slice grouping command availability revision, action-availability signature, focus-traversal availability cache, and command-interest cache under `CommandRoutingSnapshotState`, U5 touch-drag routing correctness slice making scroll/virtual-list owner lookup use `UiDispatchSnapshot.parent` instead of retained parent pointers, U5 paint replay owner-state slice naming previous-frame `Scene` replay generation/stats as `WindowPaintReplayState` while keeping boundary paint-cache entry metadata in `BoundaryFrameProducts::paint_cache`, and U6 first slice removing the `ResizablePanelGroupStyle` `fret-ui` root export/source-policy transitional classification while preserving the explicit `fret_ui::resizable_panel_group::ResizablePanelGroupStyle` mechanism path for ecosystem recipes.
- In progress: continue U6 policy vocabulary convergence after root resizable chrome export slice.
- Blocked: no blocking issue. Runtime implementation is intentionally deferred to follow-on execution.
- Next action: finish U6 root resizable chrome export verification and commit; dismiss/focus/roving policy vocabulary remains higher risk and should be audited separately before renaming or moving.

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
- Explorer `019f1883-7496-7011-bf5e-1f7b0e6ca2be` command routing snapshot-parent audit
- Explorer `019f1883-c202-7be2-81ac-3597a09ef050` remaining U5 owner-state candidates audit
- Explorer `019f18e8-c8d7-74a1-986a-9034c4467802` dispatch retained-parent fallback audit
- Explorer `019f18eb-afc8-7910-83b4-8d90e2a6f07c` paint/text/input owner-state audit
- Explorer `019f197d-45fe-7f82-b88f-6d7ff9848f2f` U6 root policy vocabulary audit
