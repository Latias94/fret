---
title: Diagnostics Start Here
status: draft
date: 2026-03-09
scope: diagnostics, navigation, maintainer, onboarding
---

# Diagnostics Start Here

Status: Draft maintainer/workstream navigation note

Status note (2026-04-12): this file remains the diagnostics v2 navigation page for maintainers and
contract readers, but the canonical first-open diagnostics workflow now lives in
`docs/diagnostics-first-open.md`. Read this file after the first-open loop is already clear or when
you specifically need workstream/background routing.

Use this page when you need one quick answer to:

- where should I start reading,
- which note owns the current plan,
- and which document answers my specific diagnostics question.

## 1) Default starting point

If you are not sure where to begin with a live UI debugging task, do **not** start here.

Start with:

- `docs/diagnostics-first-open.md`

If you are already past first-open onboarding and need diagnostics workstream/navigation context,
start here:

1. `docs/workstreams/diag-fearless-refactor-v2/README.md`
2. `docs/workstreams/diag-fearless-refactor-v2/NEXT_DEVELOPMENT_PRIORITIES.md`
3. `docs/workstreams/diag-fearless-refactor-v2/MAINTAINER_CHECKLIST.md`

That sequence answers:

- what diagnostics is trying to become,
- what should be built next,
- what you must leave behind when you land a change.

## 2) Which note to open for which question

Open the matching note first:

- Current roadmap / sequencing
  - `docs/workstreams/diag-fearless-refactor-v2/NEXT_DEVELOPMENT_PRIORITIES.md`
  - `docs/workstreams/diag-fearless-refactor-v2/CURRENT_STATUS_AND_PRIORITIES.md`
- Artifact and evidence vocabulary
  - `docs/workstreams/diag-fearless-refactor-v2/ARTIFACT_AND_EVIDENCE_MODEL_V1.md`
- Regression / campaign output contracts
  - `docs/workstreams/diag-fearless-refactor-v2/REGRESSION_CAMPAIGN_V1.md`
  - `docs/workstreams/diag-fearless-refactor-v2/REGRESSION_SUMMARY_SCHEMA_V1.md`
- Naming / orchestration vocabulary
  - `docs/workstreams/diag-fearless-refactor-v2/M3_ORCHESTRATION_VOCABULARY_AND_CONTRACT_V1.md`
- Policy-skip / capability provenance interpretation
  - `docs/workstreams/diag-fearless-refactor-v2/MAINTAINER_CHECKLIST.md`
  - `docs/workstreams/diag-fearless-refactor-v2/NON_FILESYSTEM_CAPABILITY_SOURCE_V1.md`
  - `docs/workstreams/diag-fearless-refactor-v2/CAPABILITY_PROVENANCE_MINIMAL_IMPLEMENTATION_V1.md`
- What maintainers must update when landing a change
  - `docs/workstreams/diag-fearless-refactor-v2/MAINTAINER_CHECKLIST.md`
- How old notes relate to v2
  - `docs/workstreams/diag-fearless-refactor-v2/DOCUMENT_MIGRATION_INTENT.md`

## 3) Common tasks

If your task is:

- "I need repo orientation"
  - start with `docs/workstreams/diag-fearless-refactor-v2/README.md`
- "I need to know what to build next"
  - open `docs/workstreams/diag-fearless-refactor-v2/NEXT_DEVELOPMENT_PRIORITIES.md`
- "I am changing diagnostics output or wording"
  - open `docs/workstreams/diag-fearless-refactor-v2/MAINTAINER_CHECKLIST.md`
  - then open `docs/workstreams/diag-fearless-refactor-v2/M3_ORCHESTRATION_VOCABULARY_AND_CONTRACT_V1.md`
- "I need to understand `skipped_policy`, `capability_source`, or `capabilities_check_path`"
  - open `docs/workstreams/diag-fearless-refactor-v2/MAINTAINER_CHECKLIST.md`
  - then open `docs/workstreams/diag-fearless-refactor-v2/NON_FILESYSTEM_CAPABILITY_SOURCE_V1.md`
- "I am changing aggregate artifacts or evidence paths"
  - open `docs/workstreams/diag-fearless-refactor-v2/ARTIFACT_AND_EVIDENCE_MODEL_V1.md`
- "I am changing campaign or regression summary payloads"
  - open `docs/workstreams/diag-fearless-refactor-v2/REGRESSION_CAMPAIGN_V1.md`
  - and `docs/workstreams/diag-fearless-refactor-v2/REGRESSION_SUMMARY_SCHEMA_V1.md`
- "I am trying to understand an older diagnostics note"
  - open `docs/workstreams/diag-fearless-refactor-v2/DOCUMENT_MIGRATION_INTENT.md`
  - then follow the linked older note only if it still adds unique depth

## 4) Practical rule

When in doubt:

- use v2 notes for active plan and current vocabulary,
- use older notes for background and deep topic detail,
- avoid treating older notes as parallel roadmap owners.

## 5) Short version

If one rule is enough:

- start from the v2 umbrella, then branch into contract notes or background notes only as needed.
