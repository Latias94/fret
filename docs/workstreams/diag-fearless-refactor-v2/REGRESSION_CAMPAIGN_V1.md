# Diag Fearless Refactor v2 — Regression Campaign v1

Status: Draft

Tracking doc: `docs/workstreams/diag-fearless-refactor-v2/README.md`

Related notes:

- `docs/workstreams/diag-fearless-refactor-v2/M3_ORCHESTRATION_VOCABULARY_AND_CONTRACT_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/REGRESSION_SUMMARY_SCHEMA_V1.md`

## 0) Why this note exists

Fret already has strong diagnostics primitives:

- `diag run`
- `diag suite`
- `diag repeat`
- `diag repro`
- `diag matrix`
- `diag perf`
- `diag compare`
- `diag script shrink`

What is still missing is a single **regression orchestration model** that explains how these primitives combine
into day-to-day repo workflows.

This note proposes that model.

The key point is that this is **not** an AI-only design. It is a diagnostics regression platform design that must
serve equally well:

- local maintainers,
- CI pipelines,
- DevTools GUI,
- MCP and other automation surfaces,
- future batch or scheduled runs.

## 1) Problem statement

Today, Fret has many powerful commands but no single top-level execution story for questions like:

- What should run on every PR?
- What should run only nightly?
- How should correctness vs perf vs cache-preserving checks be grouped?
- How should flaky failures be classified and retried?
- What is the one summary artifact for a repo-level run?

Without a unifying campaign model, the repo risks:

- ad-hoc shell scripts per team or per feature area,
- inconsistent naming of “smoke”, “suite”, “perf”, and “repro” workflows,
- duplicated retry/pack/evidence logic,
- DevTools GUI inventing its own run grouping model,
- CI flows that are hard to compare with local maintainer workflows.

## 2) Proposed concept: Campaign

Recommendation: introduce a first-class **campaign** concept.

A campaign is a higher-level orchestration unit that composes existing diag primitives.

It is not a replacement for `run`, `suite`, `matrix`, or `perf`.
Instead, it is the level that answers:

- which inputs run,
- in what grouping,
- with what retry/flake policy,
- with what evidence expectations,
- and with what summary outputs.

Suggested mental model:

- `script` = one repro unit
- `suite` = a named list of scripts
- `campaign` = a named regression lane made of suites/scripts + policies + outputs

## 3) Lane model

Campaigns should start from a small, explicit lane vocabulary.

### 3.1 `smoke`

Purpose:

- very fast confidence,
- low output volume,
- intended for local iteration and PR presubmit.

Typical contents:

- selected high-signal suites,
- one pass only,
- no heavy screenshots unless required,
- no broad matrix expansion,
- bounded artifacts by default.

### 3.2 `correctness`

Purpose:

- deterministic functional regression coverage.

Typical contents:

- curated suites across major surfaces,
- stable post-run checks,
- screenshot steps only where correctness truly depends on pixels,
- clear evidence bundles for every failing item.

### 3.3 `matrix`

Purpose:

- behavior-preserving comparisons across a chosen axis.

Typical contents:

- cached vs uncached,
- shell on/off,
- selected renderer/runtime toggles where meaningful,
- compare results bundled into one summary.

Note:

- matrix should remain opt-in and tightly scoped; it is too expensive to make the default lane for all scripts.

### 3.4 `perf`

Purpose:

- regressions against latency, worst-frame, throughput, or footprint baselines.

Typical contents:

- repeat runs,
- baseline comparison,
- worst bundle evidence,
- bounded perf summaries,
- optional follow-up attribution hooks.

### 3.5 `nightly` / `full`

Purpose:

- broad confidence rather than tight feedback loops.

Typical contents:

- large suite coverage,
- selected matrix lanes,
- selected perf lanes,
- richer packing and archival,
- flake classification instead of immediate developer interruption.

Vocabulary rule:

- `nightly` is the canonical persisted broad lane name.
- `full` may remain as a user-facing selector or convenience alias, but summaries and aggregate
  artifacts should normalize broad scheduled runs to `nightly` unless the repo later gives `full`
  a distinct semantic meaning.

## 4) Campaign metadata model

Campaign orchestration becomes much easier if suites and scripts can expose lightweight metadata.

Recommended metadata fields:

- `tier`
  - examples: `smoke`, `default`, `extended`, `nightly`
- `owner`
  - team or workstream responsible for failures
- `platforms`
  - e.g. `native`, `web`, `desktop`, `windows`, `macos`, `linux`
- `expected_duration`
  - rough bucket such as `short`, `medium`, `long`
- `feature_tags`
  - e.g. `overlay`, `docking`, `text_input`, `virtual_list`, `ai_ui`
- `requires_capabilities`
  - strong dependency on stable runtime/export capability tags
- `flake_policy`
  - e.g. `fail_fast`, `retry_once`, `retry_three`, `classify_only`
- `evidence_profile`
  - e.g. `bounded`, `with_screenshots`, `with_pack`, `perf_heavy`

This metadata should not become a dumping ground. The point is to support scheduling and ownership, not to encode all logic.

Vocabulary rule:

- lane names, `flake_policy`, and `requires_capabilities` should reuse the normalized vocabulary in
  `M3_ORCHESTRATION_VOCABULARY_AND_CONTRACT_V1.md` rather than introducing local synonyms in
  campaign manifests, docs, or dashboards.

## 5) Orchestration rules

### 5.1 A campaign should compose existing primitives

Preferred composition examples:

- `smoke` → selected `suite` runs
- `correctness` → selected `suite` + limited `repeat` for sensitive surfaces
- `matrix` → selected `suite` + `diag matrix`
- `perf` → selected perf suites + `diag perf`
- `nightly` → union of correctness, selected matrix, selected perf

### 5.2 A campaign should define outputs explicitly

Every campaign run should produce one top-level summary artifact.

Recommended file:

- `regression.summary.json`

Schema draft:

- `docs/workstreams/diag-fearless-refactor-v2/REGRESSION_SUMMARY_SCHEMA_V1.md`

Recommended top-level fields:

- campaign name and version
- run id / created time
- selected lanes
- totals using the normalized status vocabulary
- per-item result rows
- stable `reason_code`
- evidence bundle/artifact paths
- pack paths when generated
- flake classification results

More specifically, campaign outputs should align with the canonical status set:

- `passed`
- `failed_deterministic`
- `failed_flaky`
- `failed_tooling`
- `failed_timeout`
- `skipped_policy`
- `quarantined`

### 5.3 Campaigns should prefer bounded evidence by default

Default expectation:

- use `bundle.schema2.json` or bounded sidecars where possible,
- include raw large bundle artifacts only when required,
- keep first-open debugging lightweight.

## 6) Flake handling model

Campaigns need an explicit flake policy instead of treating every failure identically.

Recommended classification flow:

1. first failure occurs,
2. classify whether the failure is obviously deterministic,
3. if configured, re-run via `diag repeat`,
4. if still failing inconsistently, classify as `flaky`,
5. if a minimal repro is needed, invoke `diag script shrink`,
6. if the repo decides not to block on that class yet, quarantine at the campaign layer rather than hiding the raw signal.

Suggested result classes:

- `passed`
- `failed_deterministic`
- `failed_flaky`
- `failed_tooling`
- `failed_timeout`
- `skipped_policy`
- `quarantined`

Important policy rule:

- quarantine should be visible and explicit in summaries; it must not silently become “green”.
- flake handling policy and result status must remain separate:
  - `flake_policy` describes retry/classification behavior,
  - `failed_flaky` and `quarantined` are normalized result states.

## 7) Suggested execution policies by context

### 7.1 Local maintainer loop

Default target:

- `smoke`

Optional follow-ups:

- one relevant `correctness` suite,
- one focused `matrix` lane when cache-preserving behavior is in question,
- one focused `perf` lane when performance is the issue.

### 7.2 PR validation

Default target:

- `smoke`
- impacted `correctness`

Optional target:

- a limited `matrix` lane for touched subsystems with behavior-preserving risk.

### 7.3 Nightly or scheduled validation

Default target:

- broad `correctness`
- selected `matrix`
- selected `perf`
- `nightly` as the canonical persisted broad lane label

Extra expectation:

- richer summary and archive outputs,
- flake classification should be more important than fail-fast ergonomics.

## 8) DevTools GUI relationship

DevTools GUI should understand campaigns, but it should not redefine them.

That means GUI may eventually offer:

- lane pickers,
- suite/script filters,
- progress views,
- flake classification views,
- artifact browser entry points.

But campaign semantics themselves should live below the GUI layer so that CLI, CI, and MCP all share them.

## 9) Proposed deliverables for v1

Recommended v1 scope:

1. document the lane model,
2. define metadata requirements,
3. define `regression.summary.json`,
4. define flake classification vocabulary,
5. align README/TODO/MILESTONES to treat campaigns as a first-class layer,
6. align lane/status/reason/evidence path wording with
   `M3_ORCHESTRATION_VOCABULARY_AND_CONTRACT_V1.md`.

Recommended v1 non-goals:

- implementing a full scheduler UI,
- automatic impacted-test selection,
- cross-machine distributed execution,
- replacing existing `suite` manifests.

## 10) Suggested future command surface

Possible future surface:

- `fretboard-dev diag campaign smoke`
- `fretboard-dev diag campaign correctness --filter overlay`
- `fretboard-dev diag campaign nightly --json`

This surface is intentionally future-facing. The design should remain valid even if the final command name ends up
being `campaign`, `regression`, or another near-synonym.

## 11) Definition of done for this note

This note is successful when:

- the repo has one shared vocabulary for regression lanes,
- broad scheduled runs normalize to `nightly` in persisted artifacts even if `full` remains a
  user-facing alias,
- `suite`, `matrix`, `perf`, `repeat`, and `shrink` are understood as parts of one larger workflow,
- GUI and CLI can both consume the same campaign model,
- future implementation work can land incrementally without re-arguing the basics each time.
