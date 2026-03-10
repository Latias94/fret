---
title: Diag Fearless Refactor v2
status: draft
date: 2026-03-06
scope: diagnostics, automation, artifacts, devtools, refactor
---

# Diag Fearless Refactor v2

Status: Draft (workstream note)

Tracking files:

- `docs/workstreams/diag-fearless-refactor-v2/TODO.md`
- `docs/workstreams/diag-fearless-refactor-v2/MILESTONES.md`
- `docs/workstreams/diag-fearless-refactor-v2/START_HERE.md`
- `docs/workstreams/diag-fearless-refactor-v2/CRATE_AND_MODULE_MAP.md`
- `docs/workstreams/diag-fearless-refactor-v2/ARTIFACT_AND_EVIDENCE_MODEL_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/REGRESSION_CAMPAIGN_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/REGRESSION_SUMMARY_SCHEMA_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/M3_ORCHESTRATION_VOCABULARY_AND_CONTRACT_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/IMPLEMENTATION_ROADMAP.md`
- `docs/workstreams/diag-fearless-refactor-v2/NEXT_DEVELOPMENT_PRIORITIES.md`
- `docs/workstreams/diag-fearless-refactor-v2/NON_FILESYSTEM_CAPABILITY_SOURCE_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/CAPABILITY_PROVENANCE_MINIMAL_IMPLEMENTATION_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/MAINTAINER_CHECKLIST.md`
- `docs/workstreams/diag-fearless-refactor-v2/DOCUMENT_MIGRATION_INTENT.md`
- `docs/workstreams/diag-fearless-refactor-v2/DEBT_RETIREMENT_TRACKER.md`
- `docs/workstreams/diag-fearless-refactor-v2/SEAM_GATE_MATRIX.md`
- `docs/workstreams/diag-fearless-refactor-v2/RETIREMENT_CRITERIA.md`
- `docs/workstreams/diag-fearless-refactor-v2/RESIDUAL_NAMING_AUDIT.md`
- `docs/workstreams/diag-fearless-refactor-v2/BUNDLE_ARTIFACT_ALIAS_AUDIT.md`
- `docs/workstreams/diag-fearless-refactor-v2/LAYER_B_PAYLOAD_FAMILIES_AUDIT_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/RUN_MANIFEST_BUNDLE_JSON_CHUNK_INDEX_CONTRACT_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/ORCHESTRATED_OUTPUT_EVIDENCE_PATH_CONTRACT_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/OPTIONAL_COMPACT_PACK_FOR_SHARING_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/REASON_CODE_CONSUMER_ALIGNMENT_AUDIT_V1.md`

Related workstreams and context:

- Existing refactor notes: `docs/workstreams/diag-fearless-refactor-v1.md`
- Existing architecture folder: `docs/workstreams/diag-architecture-fearless-refactor-v1/README.md`
- DevTools GUI: `docs/workstreams/diag-devtools-gui-v1.md`
- DevTools MCP workflow: `docs/workstreams/diag-devtools-gui-v1-ai-mcp.md`
- Bundle + script workflow: `docs/ui-diagnostics-and-scripted-tests.md`

## 0) Why this workstream exists

Fret diagnostics has grown into a genuinely capable platform:

- in-app diagnostics runtime,
- filesystem and WebSocket transports,
- bundle artifacts and sidecars,
- script/suite/repeat/shrink/repro/matrix/perf tooling,
- DevTools GUI and MCP-facing automation surfaces.

That is good news for users, but it raises the cost of refactoring.

The current challenge is no longer "can diag do enough?" but rather:

1. can we evolve the stack without regressions,
2. can contributors find the right extension point quickly,
3. can runtime/tooling/UI move independently without reintroducing a monolith,
4. can the same architecture serve CLI, GUI, CI, and future automation surfaces equally well.

This workstream is the umbrella plan for the next phase of diagnostics refactoring.

## 1) Positioning

This workstream intentionally does **not** frame diagnostics as an "AI feature".

Diagnostics should be treated as a general-purpose **automation + debugging + evidence platform** for Fret.
AI agents are only one consumer. Other consumers matter equally:

- maintainers doing manual triage,
- component authors writing regression scripts,
- CI pipelines running smoke/correctness/perf gates,
- DevTools GUI users browsing live state and artifacts.

This positioning should drive naming and architecture:

- prefer `diag`, `automation`, `regression`, `artifacts`, `perf`,
- avoid making protocol or storage assumptions specific to one consumer.

## 2) Main problem statement

The current documentation and implementation already describe many good local designs, but the overall
stack still feels fragmented when viewed as one product surface.

Symptoms:

- runtime, tooling, artifacts, and GUI notes are split across multiple workstreams,
- there is still some overlap between "diag architecture", "diag simplification", "DevTools GUI", and
  "bundle/script workflow" mental models,
- contributors can still end up asking "where should this change live?" before they can start,
- GUI/DevTools can accidentally become a second source of truth instead of a thin consumer of the same contracts,
- some refactor steps are obvious locally but lack a single program-level sequence and exit criteria.

## 3) Scope

### In scope

- diagnostics runtime architecture and module boundaries,
- tooling engine boundaries in `crates/fret-diag`,
- artifact model and invariants,
- script/suite/repeat/matrix/perf orchestration surfaces,
- transport seams (filesystem, WebSocket, future-compatible seams),
- diagnostics-facing ecosystem extension points,
- DevTools GUI and viewer **as diagnostics consumers**,
- documentation consolidation and migration guidance.

### Explicit non-goals

- inventing a brand new scripting language,
- replacing bundles with a database,
- moving policy-heavy UI decisions into `crates/fret-ui`,
- rewriting the existing DevTools GUI from scratch,
- requiring all existing workstream docs to be deleted immediately.

## 4) Should diag UI be included?

Recommendation: **yes, but with a strict boundary**.

The DevTools/diag UI should be included in this workstream because it is one of the main consumers of the
diagnostics contracts. If we exclude it entirely, we risk refactoring runtime/tooling into shapes that later
force GUI-only adapters, duplicated logic, or transport-specific hacks.

However, UI should be included as a **consumer lane**, not as the architecture center.

That means:

- the source of truth remains the runtime export contracts and tooling artifact contracts,
- GUI should reuse existing commands, artifact stores, and resource/event surfaces whenever possible,
- GUI-specific polish should not block core runtime/tooling cleanup,
- "live inspect UX" and "artifact browser UX" are valid deliverables, but not reasons to widen core contracts casually.

Practical rule:

- if a feature is required by CLI, CI, MCP, and GUI, it belongs in the core diagnostics contracts,
- if a feature is only about how a human browses or edits diag data, it belongs in DevTools/UI.

## 5) Proposed architecture spine

The stack should be described and refactored as one layered system:

1. **Protocol + contracts**
   - script schema,
   - result schema,
   - bundle artifact and sidecars,
   - capability/version negotiation.
2. **Runtime service**
   - inspect/pick/script execution,
   - bundle/screenshot export,
   - extension slots,
   - bounded evidence generation.
3. **Transport layer**
   - filesystem,
   - WebSocket,
   - future seam-compatible transports.
4. **Tooling engine**
   - run/suite/repeat/shrink/repro/matrix/perf,
   - post-run checks,
   - artifact packing, indexing, compare, triage.
5. **Presentation surfaces**
   - `fretboard diag` CLI,
   - DevTools GUI,
   - offline bundle viewer,
   - MCP adapter.

The architecture should make it obvious that the top layer consumes the lower layers; it must not redefine them.

## 6) Refactor principles

### 6.1 Refactor by seams, not by slogans

Each refactor step should move one concern behind one explicit seam:

- artifact resolution,
- run orchestration,
- check planning,
- transport bridging,
- extension registration,
- UI resource browsing.

### 6.2 Prefer additive migration over flag-day rewrites

- add a seam,
- migrate one production path,
- add a gate,
- only then remove the old path.

### 6.3 Keep artifacts portable and bounded

- `bundle.schema2.json`, sidecars, `triage.json`, and compact evidence should remain first-class,
- avoid designs that force humans or tools to open giant raw bundle payloads for common workflows.

### 6.4 Keep GUI honest

GUI should reveal diagnostics state, not invent a parallel model of diagnostics state.

### 6.5 Keep demos separate from engine contracts

UI Gallery and demo-specific helpers are valuable proving grounds, but diagnostics contracts must stay usable by
other ecosystem crates and external app surfaces.

## 7) What "fearless" means here

For this workstream, "fearless refactor" means contributors can change internals while preserving:

- stable artifact entry points,
- stable command/task workflows,
- stable evidence expectations,
- stable extension points,
- stable mental model for where new features belong.

It does **not** mean "large rewrites are cheap". It means we deliberately build the seams and gates that make
refactoring safe enough to perform continuously.

## 8) Target outcomes

By the end of this workstream, we should be able to say:

- a contributor can locate the right diagnostics layer in minutes,
- adding a new regression gate does not require editing a giant orchestration blob,
- adding a new artifact view does not require changing the runtime schema casually,
- DevTools GUI can browse and trigger the same diagnostics model as CLI/MCP,
- diagnostics can be described as one platform instead of a set of adjacent tools.

## 8.5) Current implementation snapshot

The workstream already has a first end-to-end aggregate path in-tree:

- `diag suite`, `diag repeat`, `diag perf`, and `diag matrix` now emit
  `regression.summary.json`.
- `fretboard diag summarize` aggregates one or more summaries into:
  - `regression.summary.json` as the canonical merged summary,
  - `regression.index.json` as a lighter consumer-oriented index.
- `fretboard diag dashboard` is the first thin consumer over the aggregate index:
  - default output is a human-readable first-open summary,
  - `--json` keeps machine-readable access to the full index payload.
- `apps/fret-devtools-mcp` now exposes the same aggregate artifacts as MCP resources when they
  exist in the active artifacts root:
  - `regression.summary.json`,
  - `regression.index.json`.

This means the current question is no longer whether aggregate summaries are useful, but how
presentation surfaces should reuse them without creating a second diagnostics model.

## 9) Recommended execution order

Recommended order for landable work:

1. consolidate architecture narrative and boundaries,
   - start with `docs/workstreams/diag-fearless-refactor-v2/CRATE_AND_MODULE_MAP.md`,
2. clean up runtime/tooling seams that affect every consumer,
3. unify regression orchestration vocabulary,
   - see `docs/workstreams/diag-fearless-refactor-v2/REGRESSION_CAMPAIGN_V1.md`,
4. align DevTools GUI to the same contracts,
5. remove redundancy and close migration debt.

Current emphasis after the latest `diag_suite` / `diag_run` landings:

- `diag_campaign` is now the default next hotspot.
- within `diag_campaign`, the next highest-ROI slice is still share/export artifact planning around
  `write_campaign_share_manifest`, `write_campaign_combined_failure_zip_inner`, and
  `build_campaign_share_manifest_item`.
- `diag_run` is effectively parked unless a clearly reviewable sixth seam appears.
- artifact/materialization and presentation-surface work should follow the next campaign slice,
  not jump ahead of it.

See `TODO.md` and `MILESTONES.md` for the staged plan.


