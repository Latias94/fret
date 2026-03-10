---
title: Next Development Priorities
status: draft
date: 2026-03-09
scope: diagnostics, campaign, priorities, roadmap
---

# Next Development Priorities

Status: Draft

Tracking context:

- `docs/workstreams/diag-fearless-refactor-v2/CURRENT_STATUS_AND_PRIORITIES.md`
- `docs/workstreams/diag-fearless-refactor-v2/CAMPAIGN_CAPABILITY_PREFLIGHT_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/M3_ORCHESTRATION_VOCABULARY_AND_CONTRACT_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/TODO.md`
- `docs/workstreams/diag-fearless-refactor-v2/MILESTONES.md`

## Purpose

This note gives one short, execution-oriented answer to:

- what should be built next,
- what should explicitly wait,
- and why.

It is intentionally narrower than the broader status and milestone notes.

## Current baseline

The following are now already true:

- campaign metadata can express `requires_capabilities` and `flake_policy`,
- campaign execution now performs a first capability preflight,
- policy mismatches now surface as `skipped_policy` with `capability.missing`,
- campaign/batch outputs now distinguish policy skips from ordinary failures,
- campaign capability loading now reuses the shared filesystem capability loader,
- `diag doctor` now reports normalized capabilities from that same resolved source,
- campaign authoring now has a dedicated `diag campaign validate` entrypoint for repo-owned or
  explicit ad hoc manifests.

This means the highest-value next work is no longer "make campaign capability preflight exist".

The next value is making the new behavior easier to consume safely and avoiding premature growth in
areas that still have weak ownership.

## Priority order

### Priority 1 — Consumer adoption of the new policy-skip contract

Why this is first:

- the capability-preflight behavior now exists,
- but not every consumer or maintainer workflow will automatically interpret the new fields
  correctly,
- this is the shortest path to reducing confusion and preventing contract drift.

Recommended work:

- ensure maintainer-facing docs explicitly say how to interpret:
  - `skipped_policy`,
  - `capability.missing`,
  - `campaigns_skipped_policy`,
  - `capabilities_check_path`,
  - capability source path reporting,
- decide whether `diag campaign validate` should stay as a maintainer-only command or become the
  first cheap always-on preflight in CI / doctor tooling,
- audit any non-CLI consumer that reads campaign/batch outputs and make sure it does not collapse
  policy skips back into generic failures,
- keep the wording aligned across CLI, docs, GUI, MCP, and machine-readable reports.

Recent landing in this priority bucket:

- DevTools `Regression` drill-down now surfaces `capabilities_check_path` for selected
  `skipped_policy` summaries instead of treating them as bundle-dir-only failures,
- MCP dashboard coverage now locks `skipped_policy` status counters and shared
  `non-passing summaries` wording,
- `fret-diag` now shares internal capability provenance helpers instead of keeping path/transport
  provenance as separate ad-hoc strings,
- additive `capability_source` payloads now exist in `diag doctor`, campaign preflight summary
  evidence/metadata, and campaign aggregate/result payloads,
- DevTools `Regression` drill-down now exposes `Capability Sources` as a separate evidence lane,
- MCP regression dashboard output now also surfaces capability provenance plus capability-check
  paths when the sibling `regression.summary.json` is available.

Definition of done:

- a maintainer can tell, from one campaign root or batch root, whether the run was:
  - executed and failed,
  - skipped by policy,
  - or failed due to summarize/share follow-up issues.

### Priority 2 — Finish the capability-source contract for non-filesystem cases

Why this is second:

- the filesystem path is now aligned,
- but the broader contract question is still open: what should "capability source" mean when the
  transport is not filesystem-first?

Recommended work:

- decide whether non-filesystem transports should expose:
  - a source path,
  - a source label,
  - a transport/session identity,
  - or some additive source object,
- document that answer before another consumer depends on filesystem-only assumptions,
- keep the first extension additive rather than replacing the current string-based source field.

Recent landing in this priority bucket:

- the first-pass contract direction now lives in
  `NON_FILESYSTEM_CAPABILITY_SOURCE_V1.md`,
- the bounded follow-up implementation sketch now lives in
  `CAPABILITY_PROVENANCE_MINIMAL_IMPLEMENTATION_V1.md`,
- the documented recommendation is to keep filesystem path reporting unchanged and grow
  non-filesystem provenance through an additive source object rather than fabricated paths.

Definition of done:

- the repo has one documented answer for how capability provenance is reported outside the
  filesystem case.

Current status:

- the contract direction is now documented,
- the internal provenance helper is now landed,
- additive payload emission is now landed,
- first consumer adoption is now landed in DevTools and MCP,
- so the remaining work in this priority is no longer "introduce provenance" but "finish any
  additional consumer adoption and maintainer-facing guidance only where it adds real value".

### Priority 3 — Keep `flake_policy` deferred until there is a concrete consumer

Why this is third:

- the current capability work already solves the first real execution need,
- `flake_policy` still has unclear ownership and unclear evidence expectations,
- implementing retries too early will create a larger contract surface than the repo currently
  needs.

Recommended work:

- do not implement campaign-level retry orchestration just because the metadata field exists,
- only reopen this when a concrete CI or batch-orchestration consumer needs:
  - retry budget semantics,
  - result shaping for retries,
  - and explicit evidence rules for flaky-vs-hard failures.

Definition of done:

- `flake_policy` stays documented as passive metadata until a concrete execution consumer exists.

### Priority 4 — Decide when campaign definitions should move out of the built-in registry

Why this is fourth:

- campaign execution is now real enough that registry format questions matter,
- but moving definitions out of Rust too early would create migration churn before the execution
  contract is fully settled.

Recommended work:

- first decide what must be stable in external manifests:
  - lane,
  - tags,
  - capability requirements,
  - owner/tier/platform metadata,
  - additive future fields,
- only after that, decide whether built-in registry definitions should:
  - remain the primary source,
  - co-exist with external manifests,
  - or become generated from external data.

Definition of done:

- the repo has a documented externalization decision, not just an implicit "we might move this
  later".

### Priority 5 — Avoid broad new refactor waves unless they unlock a contract

Why this is fifth:

- the workstream has already landed many seam extractions,
- the current bottleneck is more often vocabulary/consumer alignment than another large internal
  split,
- broad refactor motion without a contract payoff is now lower leverage.

Recommended work:

- prefer bounded changes that close one contract gap,
- avoid starting another large "fearless refactor" branch unless it clearly unlocks:
  - a shared contract,
  - a reusable artifact surface,
  - or a concrete maintainer workflow.

## Explicit non-priorities

These should not lead the roadmap right now:

- another round of DevTools GUI polish by itself,
- campaign-level retry automation before a real consumer exists,
- moving campaign definitions to external manifests before registry contracts are stabilized,
- inventing a new capability probing subsystem,
- renaming raw DevTools/MCP `*json` text-holder state just for naming purity.

## Recommended execution sequence

If work continues immediately, the most defensible order is:

1. update maintainer-facing docs so the landed provenance and policy-skip behavior is described in
   one place,
2. only add more consumer adoption if another real consumer needs richer provenance than DevTools
   and MCP already expose,
3. keep `flake_policy` deferred unless a concrete consumer appears,
4. only then decide whether campaign definitions should externalize.

## Short version

If only one sentence is needed:

- make the newly landed campaign capability-preflight contract easy to consume correctly before
  growing the next execution feature.
