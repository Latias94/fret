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
- Last verified: U5 interaction replay entry boundary slice passed `cargo fmt --all --check`, `git diff --check`, `cargo check -p fret-ui --all-targets`, `cargo check -p fret-bootstrap`, `cargo check -p fret-diag`, 8 focused `cargo nextest -p fret-ui` tests covering prepaint interaction cache reuse plus hit-test/focus/outside-press consumers, `cargo nextest run -p fret-diag bundle_stats_preserves_cache_root_boundary_summary --no-fail-fast`, `cargo nextest run -p fret-bootstrap --lib --no-fail-fast`, `python3 tools/check_layering.py`, `python3 tools/check_surface_policy.py`, and `python3 tools/check_consumption_profiles.py`.
- Done: local ADR/workstream research, crate/perf snapshots, GPUI/Zed comparison, architecture boundary audit, framework consumer audit, performance audit, implementation-ready plan, U1 convergence contract freeze, U2 source-policy gate, U3 first slice (`workbench-lite` public scaffold), U4 first observability slice for identity fallback pressure/parent repair/GC reachability/dispatch snapshot cache pressure, U4 second observability slice for dirty frontier breadth plus model/global observation-index churn, U5 first slice replacing raw `UiTree::dirty_boundaries` ownership with `DirtyViewFrontier`, U5 BoundaryFrameProducts slice grouping boundary dirty/prepaint/scene-fragment/paint-cache state under `ViewBoundaryState::frame_products`, and U5 interaction replay entry slice removing `Node::interaction_cache` in favor of `BoundaryFrameProducts::interaction_cache`.
- In progress: choose the next narrow U5 frame-product follow-on.
- Blocked: no blocking issue. Runtime implementation is intentionally deferred to follow-on execution.
- Next action: audit the smallest remaining U5 boundary-owned frame-product follow-on, likely dispatch snapshot ownership or another node-local prepaint side channel, without mixing it with paint recording, scene chunks, or text/glyph budget work.

# Citations

- [Plan](../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md)
- [Subagent Findings](subagents/2026-06-30-ui-framework-architecture-audit-findings.md)
- Commit `020bb34a37 docs(architecture): freeze ui convergence contract`
- Commit `84f60d8355 feat(tools): add ui surface policy gate`
- Commit `df0d6620ff feat(ui): expose dirty frontier diagnostics`
- Commit `09debbceae refactor(ui): move dirty frontier behind view ids`
- Commit `ac1aa7ba27 refactor(ui): group boundary frame products`
- Commit `8572864d49 refactor(ui): move interaction replay entries to boundaries`
