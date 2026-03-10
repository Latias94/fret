---
title: Diagnostics Document Migration Intent
status: draft
date: 2026-03-09
scope: diagnostics, docs, migration, maintenance
---

# Diagnostics Document Migration Intent

Status: Draft

Purpose:

- reduce duplicate maintenance across old diagnostics workstreams,
- clarify which notes remain active sources of truth,
- and explain which older notes should be treated as specialized background or historical context.

## 1) Primary rule

For diagnostics refactor/navigation questions, start from:

- `docs/workstreams/diag-fearless-refactor-v2/README.md`

Treat the v2 umbrella as the primary navigation entry for:

- current refactor direction,
- artifact/evidence vocabulary,
- regression/campaign summary contracts,
- maintainer workflow and next priorities.

## 2) Document roles

### Keep as current umbrella / active coordination notes

- `docs/workstreams/diag-fearless-refactor-v2/README.md`
- `docs/workstreams/diag-fearless-refactor-v2/TODO.md`
- `docs/workstreams/diag-fearless-refactor-v2/MILESTONES.md`
- `docs/workstreams/diag-fearless-refactor-v2/CURRENT_STATUS_AND_PRIORITIES.md`
- `docs/workstreams/diag-fearless-refactor-v2/NEXT_DEVELOPMENT_PRIORITIES.md`
- `docs/workstreams/diag-fearless-refactor-v2/MAINTAINER_CHECKLIST.md`

Use these for:

- what is active now,
- what should be built next,
- what maintainers must update when they land changes.

### Keep as active contract/supporting notes

- `docs/workstreams/diag-fearless-refactor-v2/ARTIFACT_AND_EVIDENCE_MODEL_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/M3_ORCHESTRATION_VOCABULARY_AND_CONTRACT_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/REGRESSION_CAMPAIGN_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/REGRESSION_SUMMARY_SCHEMA_V1.md`

Use these for:

- canonical schema/vocabulary decisions,
- artifact and evidence expectations,
- campaign/regression output contracts.

### Keep older notes as specialized background, not umbrella sources

- `docs/workstreams/diag-fearless-refactor-v1.md`
- `docs/workstreams/diag-architecture-fearless-refactor-v1/README.md`
- `docs/workstreams/diag-devtools-gui-v1.md`
- `docs/workstreams/diag-devtools-gui-v1-ai-mcp.md`
- `docs/workstreams/diag-simplification-v1-m0-baseline.md`

Intent:

- keep them for historical rationale, design depth, or topic-specific detail,
- add forward links to the v2 umbrella,
- avoid continuing to evolve them as parallel top-level planning documents.

### Keep topic notes only when they still provide unique depth

Examples:

- architecture deep dives,
- transport/capability background,
- GUI-specific UX exploration,
- older migration guides that still explain one narrow topic well.

Rule:

- if a note adds unique technical depth, keep it and link back to v2,
- if a note only repeats current status or roadmap language, prefer trimming or leaving it frozen.

## 3) What should move, and what should not

Move into v2 umbrella notes when content is:

- current roadmap or sequencing,
- active maintainer workflow,
- current contract vocabulary,
- "where should this change live?" guidance.

Do not force-move into v2 umbrella when content is:

- a deep technical appendix,
- a focused historical rationale,
- a topic-specific design note that still stands on its own.

## 4) Update policy for old notes

For older diagnostics workstreams:

- do add a short forward link to the v2 umbrella,
- do keep factual topic-specific content if it still helps,
- do not keep editing their roadmap/status sections as if they are still primary,
- do not create a second active checklist or priority list there.

## 5) Practical maintainer rule

When touching diagnostics docs during implementation:

1. update the v2 umbrella/current-status/priority note first,
2. update the specific contract note if the contract changed,
3. only touch an older workstream note if it still provides unique topic depth or needs a forward
   link.

## 6) Non-goal

This note does not require deleting old workstreams now.

The goal is:

- fewer parallel planning surfaces,
- not aggressive doc churn.

## 7) Short version

If one rule is enough:

- keep v2 as the live umbrella, keep old notes as linked background unless they still carry unique
  technical depth.
