# fret-ui Layout Architecture Audit v1 - Milestones

Status: Closed

## M0 - Open the lane

- [x] Create the workstream with design, task ledger, gates, and handoff.
- [x] Record the initial assumption that this is an audit lane, not an immediate rewrite.

## M1 - Baseline and inventory

- [x] Inventory the current layout classification and execution model.
- [x] Capture one local perf/diagnostics baseline that can explain current solve owners.
- [x] Record whether current complexity is justified by known contracts.

## M2 - Architecture decision

- [x] Decide whether to keep, extract, or remodel the clean-geometry classification code.
- [x] If a refactor is chosen, define the smallest behavior-preserving first slice.
- [x] Update ADR alignment only if the runtime contract changes.
  - No ADR alignment update was required because FLA-040 only moved private implementation code and
    did not change a runtime contract.

## M3 - First landable follow-on

- [x] Either land a behavior-preserving organization refactor, or explicitly close the audit with a
  "no refactor now" decision.
- [x] Identify the next performance owner and its correct workstream.
  - Follow-on: `docs/workstreams/retained-layout-orchestration-v1/`.

## Exit criteria

This lane is complete when a maintainer can answer:

- whether the current node classification is conceptually sound,
- whether the file/module shape should change,
- what evidence supports that decision,
- and what the next performance lane should be.
