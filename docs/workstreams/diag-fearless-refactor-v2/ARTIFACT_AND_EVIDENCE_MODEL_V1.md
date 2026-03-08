---
title: Diag Artifact & Evidence Model v1
status: draft
date: 2026-03-07
scope: diagnostics, artifacts, evidence, compatibility, triage
---

# Diag Artifact & Evidence Model v1

Status: Draft

Tracking workstream: `docs/workstreams/diag-fearless-refactor-v2/README.md`

Related references:

- `docs/ui-diagnostics-and-scripted-tests.md`
- `docs/workstreams/diag-fearless-refactor-v2/REGRESSION_SUMMARY_SCHEMA_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/REGRESSION_CAMPAIGN_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/MAINTAINER_AUTOMATION_FLOW.md`

## Purpose

This note defines one shared artifact and evidence model for Fret diagnostics.

It exists to answer five practical questions:

1. which files are source-of-truth artifacts,
2. which files are derived caches or projections,
3. which files are optional evidence for handoff or deeper triage,
4. which small subset should be opened first during normal maintainer workflows,
5. how campaign, batch, share, and bundle-sidecar outputs relate to each other.

This document is intentionally consumer-neutral.

The same rules should work for:

- CLI flows,
- DevTools GUI,
- CI pipelines,
- share/handoff tooling,
- MCP or future automation consumers.

## Design goals

The artifact model should remain:

- explicit,
- additive,
- bounded for first-open triage,
- safe for offline handoff,
- stable enough that new consumers do not invent parallel file vocabularies.

## Artifact classes

Diagnostics artifacts fall into four classes.

### 1. Source-of-truth run artifacts

These are the canonical outputs of a run or orchestrated run.

Examples:

- `bundle.json` or `bundle.schema2.json`
- `script.result.json`
- `regression.summary.json`
- `campaign.result.json`
- `batch.result.json`

Rules:

- These artifacts describe the actual observed run output or the canonical structured summary of it.
- If a derived artifact disagrees with a source-of-truth artifact, the source-of-truth artifact wins.
- New consumers should prefer these artifacts before inventing new persisted projections.

### 2. Derived summary and index artifacts

These are convenience artifacts that repackage or aggregate source-of-truth data.

Examples:

- `regression.index.json`
- `matrix.summary.json`
- `campaign.manifest.json`
- `batch.manifest.json`
- future compact dashboard-style index files

Rules:

- These artifacts may be regenerated.
- They must not become the only place where essential source data exists.
- They are allowed to optimize first-open inspection, ranking, or navigation.

### 3. Optional evidence artifacts

These are handoff- and debugging-oriented files that may or may not exist depending on failure mode,
sharing mode, or policy.

Examples:

- `triage.json`
- screenshots manifests
- screenshot image bodies
- packed bundle zips
- AI-only share zips
- combined failure zips
- `share/share.manifest.json`

Rules:

- Optional evidence is additive.
- Missing optional evidence must not make the canonical result unreadable.
- Optional evidence may be pruned, packed, or copied for portability.

### 4. Presentation-facing projections

These are consumer-friendly files or views whose main purpose is navigation, ranking, or display.

Examples:

- dashboard-oriented ranked lists from `regression.index.json`
- GUI panels backed by summary/index artifacts
- future HTML or richer static report projections

Rules:

- Presentation-facing projections are never the canonical truth.
- GUI and dashboard surfaces must consume shared artifacts rather than define parallel persistence.

## Canonical taxonomy

### Bundle artifacts

Definition:

- A diagnostics bundle is the most detailed captured snapshot/log/event package for a single run or
  bundle-producing step.

Typical contents:

- `bundle.json` or `bundle.schema2.json`
- event streams or frame metadata
- optional bundle-local sidecars

Ownership:

- Source-of-truth for low-level captured diagnostics state.

Best use:

- deep debugging,
- replay-style analysis,
- offline handoff when detailed inspection is required.

### Sidecars

Definition:

- Sidecars are files colocated with a bundle or run directory that add extra structured evidence
  without redefining the bundle itself.

Examples:

- screenshots manifests,
- compact notes,
- AI packet sidecars,
- tooling-specific metadata.

Ownership:

- Optional evidence or derived evidence, depending on the file.

Best use:

- attach extra context while keeping the main bundle stable.

### `script.result.json`

Definition:

- The canonical structured outcome of a scripted run step.

Ownership:

- Source-of-truth for script pass/fail status and structured script-stage output.

Best use:

- script authoring,
- tooling failure classification,
- reproducible automation outcomes.

Notes:

- `script.result.json` should remain small enough to inspect without opening the largest raw artifact.
- It may point to richer evidence, but should not require that richer evidence for basic status.

### `triage.json`

Definition:

- A compact failure-oriented summary for first-pass diagnosis and handoff.

Ownership:

- Optional evidence, derived from canonical results and available artifacts.

Best use:

- first-open human triage,
- share packaging,
- support for future automation summarizers.

Notes:

- `triage.json` is helpful but not required for correctness.
- It should remain bounded and portable.

### Compact pack and AI packet artifacts

Definition:

- Compact share artifacts that intentionally package only the bounded subset needed for handoff.

Examples:

- AI-only packet outputs,
- per-failure share zips,
- combined failure zips.

Ownership:

- Optional evidence.

Best use:

- remote handoff,
- issue filing,
- CI attachment uploads,
- automation consumers that do not need the full raw run tree.

## Source-of-truth vs derived vs optional vs GUI-facing

The table below is the recommended default classification.

| Artifact | Classification | Required for correctness | Typical consumer |
| --- | --- | --- | --- |
| `bundle.json` / `bundle.schema2.json` | Source-of-truth | No, unless deep run state is needed | Maintainers, tooling, offline debugging |
| `script.result.json` | Source-of-truth | Yes for script outcome | CLI, CI, script tooling |
| `regression.summary.json` | Source-of-truth | Yes for regression-level status | CLI, GUI, CI, dashboards |
| `campaign.result.json` | Source-of-truth | Yes for campaign-level status | CLI, GUI, CI |
| `batch.result.json` | Source-of-truth | Yes for batch-level status | CLI, GUI, CI |
| `regression.index.json` | Derived/cache-like | No | GUI, dashboards, first-open triage |
| `campaign.manifest.json` | Derived descriptor | No | Maintainers, GUI, registry inspection |
| `batch.manifest.json` | Derived descriptor | No | Maintainers, GUI |
| `triage.json` | Optional evidence | No | Humans, share flows |
| screenshots manifest | Optional evidence | No | Humans, share flows |
| screenshot image bodies | Optional evidence | No | Humans |
| share zips / AI packets | Optional evidence | No | Handoff, CI attachments |
| GUI panels / dashboards | Presentation-facing | No | Humans |

## Compatibility policy

Diagnostics artifact compatibility should follow these rules.

### Additive changes are preferred

Preferred changes:

- add new nullable fields,
- add new optional companion artifacts,
- add new ranked/index-style convenience views,
- add new evidence paths without removing old canonical fields abruptly.

### Hard removals require a migration note

If a field or artifact path is removed or renamed from a canonical artifact:

- document the change in the owning workstream or ADR note,
- update all known consumers in the same landing where possible,
- avoid silently changing meaning under the same field name.

### `schema_version` and `kind` stay explicit

All persisted top-level artifacts should continue to carry explicit:

- `schema_version`
- `kind`

Rationale:

- mixed artifact directories are common,
- tooling must be able to distinguish result, manifest, share, summary, and index artifacts without
  guessing by filename alone.

### Derived artifacts may be regenerated

If a derived artifact becomes stale or goes missing:

- canonical source-of-truth artifacts remain authoritative,
- consumers may regenerate derived views where that workflow already exists,
- correctness must not depend on an index cache being perfect.

## First-open artifact set

Most maintainers should not need to open the raw largest artifact first.

The recommended first-open set is:

1. `regression.summary.json` or `campaign.result.json` / `batch.result.json`
2. `regression.index.json` when ranking or multiple failures matter
3. `triage.json` if present
4. `script.result.json` for the failing script or step
5. share manifest / compact zip only when handing off or collecting bounded evidence

The raw bundle should usually be opened after that, not before.

This keeps common triage bounded while still preserving a path to deeper diagnostics.

## Relationship between campaign, batch, share, and bundle-sidecar outputs

The intended layering is:

### Bundle layer

- Owns the detailed captured diagnostics state for a run step.
- Sidecars may enrich the bundle, but do not replace it.

### Script/suite result layer

- Owns structured step outcome and script-level status.
- May point to bundle evidence or sidecars.

### Regression summary layer

- Owns the canonical run-level or orchestrated summary for humans and tools.
- May aggregate multiple step results.

### Campaign and batch artifact layer

- `campaign.manifest.json` / `batch.manifest.json` describe requested and resolved execution shape.
- `campaign.result.json` / `batch.result.json` describe canonical campaign or batch outcomes.
- These artifacts should point downward to summary/index/share evidence, not duplicate raw bundle
  contents inline.

### Share layer

- `share/share.manifest.json`, compact packets, and combined failure zips package bounded evidence for
  handoff.
- Share artifacts are convenience packages over existing canonical artifacts and optional evidence.
- Share artifacts must not be the only place where canonical machine-readable status exists.

## Consumer expectations

### CLI

- Should prefer source-of-truth artifacts for status.
- May use derived/index artifacts for summary displays.

### DevTools GUI

- Should treat summary/result/index artifacts as the shared contract.
- Should not require GUI-only persisted models for the same run state.

### CI

- Should archive the bounded first-open set by default.
- May attach compact share artifacts for failures.
- Should not assume every run needs the full raw bundle uploaded.

### Future automation or MCP consumers

- Should start from source-of-truth summary/result artifacts and only descend into raw bundles when
  needed.
- Should treat derived caches as convenience, not authority.

## Consumer checklist

This section turns the artifact model into a concrete checklist for the main consumer lanes.

### CLI checklist

Use this checklist when adding or changing CLI-facing diagnostics commands.

- Must read canonical status from `script.result.json`, `regression.summary.json`,
  `campaign.result.json`, or `batch.result.json`.
- May use `regression.index.json` for ranked or summary-first output.
- Should print bounded first-open paths before suggesting raw bundle inspection.
- Should treat missing optional evidence as non-fatal when canonical result artifacts still exist.
- Should not require GUI-only files or share-only packages to report basic status.

### DevTools GUI checklist

Use this checklist when adding or changing diagnostics GUI views.

- Must consume shared summary/result/index artifacts rather than invent a parallel persisted model.
- Should open `regression.index.json` or result artifacts first for navigation and ranking.
- May drill down into `triage.json`, share manifests, or bundle-sidecar evidence for details.
- Should expose evidence paths and copy actions using the same filenames and terminology as CLI.
- Should not reinterpret a derived cache as the canonical truth when source-of-truth artifacts exist.

### CI checklist

Use this checklist when defining default uploads, retention, or failure attachments.

- Must preserve canonical result/summary artifacts for failed runs.
- Should archive the bounded first-open set by default:
  - `regression.summary.json` or `campaign.result.json` / `batch.result.json`,
  - `regression.index.json` when present,
  - `script.result.json` for failing steps,
  - `triage.json` when present.
- May attach compact share artifacts or combined failure zips for remote inspection.
- Should upload raw bundles selectively based on failure depth, policy, or retention budget.
- Should not assume that every failure requires the largest raw artifact to be useful.

### Share and handoff checklist

Use this checklist when producing artifacts for humans, issue reports, or external automation.

- Must package or reference canonical source-of-truth status artifacts, not only a share zip.
- Should include bounded evidence that accelerates first-open triage:
  - `triage.json`,
  - share manifest,
  - compact failure zip,
  - selected screenshots or screenshot manifests when relevant.
- May omit raw bundles when the compact package is sufficient for the intended handoff.
- Should preserve enough path context that a receiver can trace evidence back to canonical outputs.
- Should not make `share/share.manifest.json` the only machine-readable status artifact in the handoff.

### Automation and MCP checklist

Use this checklist when adding machine-oriented consumers or future orchestration agents.

- Must begin from source-of-truth result/summary artifacts whenever they exist.
- Should use derived/index artifacts only as acceleration layers for ranking, routing, or first-open
  decisions.
- May descend into bundle artifacts and sidecars only when the bounded first-open set is
  insufficient.
- Should treat missing optional evidence as a capability difference, not automatic corruption.
- Should not infer semantics from filenames alone when `kind` and `schema_version` are present.

## Consumer-first artifact matrix

| Consumer | Open first | Optional follow-up | Must not depend on for basic status |
| --- | --- | --- | --- |
| CLI | result/summary artifact | index, triage, bundle | GUI-only state, share-only package |
| DevTools GUI | index or result/summary artifact | triage, share manifest, bundle | GUI-private canonical store |
| CI | result/summary artifact + failing `script.result.json` | index, triage, compact share zip, bundle | raw bundle for every run |
| Share flow | result/summary artifact + share manifest | triage, compact zips, selected screenshots | share manifest as only truth |
| Automation/MCP | result/summary artifact | index, triage, bundle, sidecars | derived cache as authority |

## Recommended default retention posture

Recommended default order of importance for retention:

1. canonical result/summary artifacts,
2. small index and triage artifacts,
3. bounded share artifacts for failures,
4. raw bundles and large evidence bodies when needed by policy or debugging depth.

This keeps ordinary workflows practical while preserving access to deep evidence for harder cases.

## Definition of done for future artifact additions

When a new artifact is introduced, the landing should answer:

1. is it source-of-truth, derived, optional evidence, or presentation-facing,
2. what existing artifact it derives from,
3. whether it belongs in the first-open set,
4. whether it is safe to omit in normal runs,
5. which consumers are expected to read it.

If those answers are unclear, the artifact probably does not yet have a stable place in the model.

## Bottom line

The diagnostics stack should treat artifacts as a layered evidence system, not as one giant file dump.

The stable model is:

- canonical results and summaries for truth,
- indexes and manifests for navigation,
- optional evidence for deeper diagnosis and handoff,
- presentation surfaces as consumers over the same files.

That is the contract M2 should preserve.
