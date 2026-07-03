---
type: Current State
title: Fret architecture planning current state
tags: fret,architecture,planning
timestamp: 2026-07-03
related_plan: docs/plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
---

# Current State

- Goal: execute `docs/plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md` as a breaking Phase 3 retained-bridge deletion refactor.
- Branch: `feat/ui-framework-phase2-refactor`.
- Last verified: Phase 3 U3 propagation depth topology slice passed focused propagation/model
  invalidation tests, full `fret-ui` nextest, formatting, whitespace, layering, surface,
  consumption-profile, execution-surface, ADR-number, workstream-catalog, and wiki-memory gates on
  2026-07-03.
- Done: local ADR/workstream research, crate/perf snapshots, GPUI/Zed comparison, architecture boundary audit, framework consumer audit, performance audit, implementation-ready plan, U1 convergence contract freeze, U2 source-policy gate, U3 first slice (`workbench-lite` public scaffold), U4 identity/dirty graph observability slices, U5 `ViewId` / boundary frame-product ownership slices, U6 policy vocabulary demotion/cleanup slices, U7 renderer scene/upload observability plus retained scene chunk and guarded quad resident upload lanes, U8 text/glyph/wasm budget work through web runtime evidence, U9 modular consumption profiles and `AppUi` facade split, Phase 2 U9 VertexColor viewport partial upload, Phase 2 U10 workbench-lite public settings diagnostics, Phase 2 U11 public mutation/toast wrappers, workstream closeout audit, duplicate ADR ID `0324` resolution, and execution-surface allowlist alignment.
- Latest done: Phase 3 U3 ninth slice migrated observation invalidation propagation depth from
  retained `Node.parent` to layer-forest child-edge parents.
- In progress: Phase 3 U3 remaining normal-query retained parent audit.
- Blocked: none known after the boundary store migration.
- Next action: continue U3 by migrating bounds-tree prepaint parent reconstruction, then classify
  remaining invalidation/debug-only parent reads where they are normal query paths; keep retained
  storage mutation and U5 parent repair/dirty-count work separate.

# Citations

- [Phase 2 plan](../../plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md)
- [Phase 3 retained bridge deletion plan](../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md)
- [Phase 3 U2 retained pressure gates](progress/2026-07-03-phase3-u2-retained-pressure-gates.md)
- [Phase 3 U3 child-edge topology](progress/2026-07-03-phase3-u3-child-edge-topology.md)
- [Phase 3 U3 layout and viewport topology](progress/2026-07-03-phase3-u3-layout-viewport-topology.md)
- [Phase 3 U3 root and descendant topology](progress/2026-07-03-phase3-u3-root-descendant-topology.md)
- [Phase 3 U3 dispatch fallback topology](progress/2026-07-03-phase3-u3-dispatch-fallback-topology.md)
- [Phase 3 U3 input ancestry topology](progress/2026-07-03-phase3-u3-input-ancestry-topology.md)
- [Phase 3 U3 hit-test topology](progress/2026-07-03-phase3-u3-hit-test-topology.md)
- [Phase 3 U3 semantics topology](progress/2026-07-03-phase3-u3-semantics-topology.md)
- [Phase 3 U3 widget coordinate topology](progress/2026-07-03-phase3-u3-widget-coordinate-topology.md)
- [Phase 3 U3 propagation depth topology](progress/2026-07-03-phase3-u3-propagation-depth-topology.md)
- [Phase 1 convergence plan](../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md)
- [Subagent Findings](subagents/2026-06-30-ui-framework-architecture-audit-findings.md)
- [U9 AppUi shell split audit](subagents/2026-07-02-u9-appui-shell-split-audit.md)
- [U8 web wasm runtime evidence closeout](progress/2026-07-02-u8-web-wasm-runtime-evidence.md)
- [U8 web gallery-dev page availability audit](subagents/2026-07-02-u8-web-gallery-dev-page-audit.md)
- [UI convergence closeout audits](subagents/2026-07-02-ui-convergence-closeout-audits.md)
- [UI convergence plan closeout](progress/2026-07-02-ui-convergence-closeout.md)
- [ADR numbering and pre-release smoke restored](progress/2026-07-02-adr-numbering-pre-release-smoke.md)
- [Phase 2 U2 stable element node index](progress/2026-07-02-phase2-u2-stable-element-index.md)
- [Phase 2 U3 live fallback scan deletion](progress/2026-07-02-phase2-u3-live-fallback-scan-deletion.md)
- [Phase 2 U4 ViewId bridge split](progress/2026-07-02-phase2-u4-viewid-bridge-split.md)
- [Phase 2 U4 boundary store migration](progress/2026-07-02-phase2-u4-boundary-store-migration.md)
- [Phase 2 U4 durable ViewId lifecycle](progress/2026-07-02-phase2-u4-durable-viewid-lifecycle.md)
- [Phase 2 U5 boundary layout candidates](progress/2026-07-02-phase2-u5-boundary-layout-candidates.md)
- [Phase 2 U5 observation boundary subscribers](progress/2026-07-02-phase2-u5-observation-boundary-subscribers.md)
- [Phase 2 U6 chunk closure native payload](progress/2026-07-02-phase2-u6-chunk-closure-native-payload.md)
- [Phase 2 U7 chunk-local text resource closure](progress/2026-07-02-phase2-u7-chunk-local-text-resource-closure.md)
- [Phase 2 U8 explicit render scene source](progress/2026-07-02-phase2-u8-explicit-render-scene-source.md)
- [Phase 2 U9 VertexColor viewport partial upload](progress/2026-07-02-phase2-u9-viewport-partial-upload.md)
- [Phase 2 U10 workbench-lite settings diagnostics](progress/2026-07-02-phase2-u10-workbench-settings-diagnostics.md)
- [Phase 2 U11 public mutation and toast wrappers](progress/2026-07-02-phase2-u11-public-mutation-toast-wrappers.md)
- [Phase 2 U12 mutation-workbench starter](progress/2026-07-02-phase2-u12-mutation-workbench-starter.md)
- [Phase 2 U13 surface quarantine records](progress/2026-07-02-phase2-u13-surface-quarantine-records.md)
- [Phase 2 U14 AppUi facade narrowing](progress/2026-07-02-phase2-u14-appui-facade-narrowing.md)
- [Phase 2 closeout](progress/2026-07-02-phase2-closeout.md)
- [Phase 2 U4 boundary store audit](subagents/2026-07-02-phase2-u4-boundary-store-audit.md)
- [Phase 2 U4 durable ViewId audit](subagents/2026-07-02-phase2-u4-durable-viewid-audit.md)
- [Phase 2 U5 boundary bridge audit](subagents/2026-07-02-phase2-u5-boundary-bridge-audit.md)
- [Phase 2 U5 observation fanout audit](subagents/2026-07-02-phase2-u5-observation-fanout-audit.md)
- [Phase 2 U6 chunk closure audit](subagents/2026-07-02-phase2-u6-chunk-closure-audit.md)
- [Phase 2 U7 text resource closure audit](subagents/2026-07-02-phase2-u7-text-closure-audit.md)
- [Phase 2 U8 flat scene source audit](subagents/2026-07-02-phase2-u8-flat-scene-source-audit.md)
- [Phase 2 U9 partial upload guard audit](subagents/2026-07-02-phase2-u9-partial-upload-audit.md)
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
