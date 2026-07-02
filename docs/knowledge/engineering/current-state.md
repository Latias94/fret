---
type: Current State
title: Fret architecture planning current state
tags: fret,architecture,planning
timestamp: 2026-07-02
related_plan: docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
---

# Current State

- Goal: execute the implementation-ready fearless refactor plan for Fret's UI framework architecture convergence.
- Branch: `feat/ui-framework-convergence` from local `main` after the planning commit.
- Last verified: ADR numbering and release-policy smoke passed on 2026-07-02: `python3 tools/check_adr_numbers.py`, `python3 tools/check_execution_surface.py`, `python3 tools/pre_release.py --skip-fmt --skip-clippy --skip-nextest --skip-icons --skip-release-closure --skip-portable-time --skip-diff-check`, workstream catalog, wiki-memory validation, and `git diff --check`.
- Done: local ADR/workstream research, crate/perf snapshots, GPUI/Zed comparison, architecture boundary audit, framework consumer audit, performance audit, implementation-ready plan, U1 convergence contract freeze, U2 source-policy gate, U3 first slice (`workbench-lite` public scaffold), U4 identity/dirty graph observability slices, U5 `ViewId` / boundary frame-product ownership slices, U6 policy vocabulary demotion/cleanup slices, U7 renderer scene/upload observability plus retained scene chunk and guarded quad resident upload lanes, U8 text/glyph/wasm budget work through web runtime evidence, U9 modular consumption profiles and `AppUi` facade split, workstream closeout audit, duplicate ADR ID `0324` resolution, and execution-surface allowlist alignment.
- Latest done: ADR numbering and skip-heavy pre-release smoke restoration.
- In progress: no active work in the closed convergence coordinator.
- Blocked: no blocking issue.
- Next action: start a narrow follow-on from the closeout retained/deferred table, or run the full
  aggregate pre-release gate when release scope needs it.

# Citations

- [Plan](../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md)
- [Subagent Findings](subagents/2026-06-30-ui-framework-architecture-audit-findings.md)
- [U9 AppUi shell split audit](subagents/2026-07-02-u9-appui-shell-split-audit.md)
- [U8 web wasm runtime evidence closeout](progress/2026-07-02-u8-web-wasm-runtime-evidence.md)
- [U8 web gallery-dev page availability audit](subagents/2026-07-02-u8-web-gallery-dev-page-audit.md)
- [UI convergence closeout audits](subagents/2026-07-02-ui-convergence-closeout-audits.md)
- [UI convergence plan closeout](progress/2026-07-02-ui-convergence-closeout.md)
- [ADR numbering and pre-release smoke restored](progress/2026-07-02-adr-numbering-pre-release-smoke.md)
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
- Commit `3366af80ee refactor(ui)!: demote resizable chrome root export`
- Commit `ebabd7a444 refactor(ui)!: rename scroll dismiss layer hook`
- Explorer `019f19a8-e921-7751-ac01-4575becdd6c4` U6 `fret-ui` action/focus/dismiss public API audit
- Explorer `019f19a9-2a2f-7741-bd71-2c603a0c9430` U6 ecosystem policy vocabulary consumption audit
