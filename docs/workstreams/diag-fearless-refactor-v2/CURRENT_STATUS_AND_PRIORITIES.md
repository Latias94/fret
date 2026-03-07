# Diag Fearless Refactor v2 - Current Status and Priorities

Status: Draft

Tracking doc: `docs/workstreams/diag-fearless-refactor-v2/README.md`

Related notes:

- `docs/workstreams/diag-fearless-refactor-v2/TODO.md`
- `docs/workstreams/diag-fearless-refactor-v2/IMPLEMENTATION_ROADMAP.md`
- `docs/workstreams/diag-fearless-refactor-v2/CAMPAIGN_EXECUTION_ENTRY_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/MAINTAINER_AUTOMATION_FLOW.md`

## Purpose

This note records the current state of the diagnostics workstream after the first campaign execution,
batch artifact, and failure-evidence slices.

The goal is to make the next stage explicit so follow-up work does not drift toward low-leverage
polish while the highest-value seams are still under-documented or too monolithic.

## What is now meaningfully landed

### 1. Campaign orchestration is no longer a sketch

The repo now has a usable campaign surface:

- `fretboard diag campaign list`
- `fretboard diag campaign show <campaign_id>`
- `fretboard diag campaign run <campaign_id>`
- filtered batch execution via `--lane`, `--tier`, `--tag`, and `--platform`
- `fretboard diag campaign share <campaign_or_batch_root>`

This is enough to treat campaign execution as a real maintainer and CI entry rather than only a
design note.

### 2. Batch artifact roots are now a stable handoff surface

Filtered or multi-id campaign runs now persist batch roots under:

- `campaign-batches/<selection_slug>/<run_id>/`

Those roots already expose the same aggregate consumer contract used elsewhere:

- `regression.summary.json`
- `regression.index.json`
- `batch.manifest.json`
- `batch.result.json`

This is important because it keeps DevTools, CLI, MCP, and future GUI surfaces on one shared
artifact vocabulary.

### 3. Failure evidence is now good enough to hand off

Failed campaign and batch roots now best-effort produce:

- `share/share.manifest.json`
- per-item AI-only share zips
- best-effort `triage.json`
- best-effort `screenshots_manifest`
- `share/combined-failures.zip`

This is the first point where a maintainer can usually hand off one directory without manually
collecting failing bundle paths.

## What is now "good enough" and should not lead the roadmap

The following areas are useful, but they should no longer be the primary driver for the next phase:

- basic campaign command discoverability,
- first-pass share helpers,
- first-pass failure evidence packaging,
- GUI polish that does not change contracts,
- dashboard text or HTML projection.

These are now in the "maintain and extend carefully" bucket, not the "main strategic blocker" bucket.

## The highest-value next priorities

### Priority 1. Finish the canonical artifact/evidence contract

Why it is first:

- implementation has moved faster than the repo-level artifact contract,
- multiple commands now emit related evidence (`summary`, `index`, `triage`, `ai.packet`, share zips,
  screenshot manifests),
- the workstream still lacks one explicit repo-level statement of what is source-of-truth vs derived
  vs optional handoff evidence.

Concretely, the next doc slice should settle:

- which artifacts are source-of-truth,
- which artifacts are derived caches or projections,
- which artifacts are required for first-open triage,
- how campaign/batch/share outputs relate to existing bundle-sidecar contracts.

Recommended landing target:

- extend `M2` in `TODO.md` rather than starting a separate note family.

### Priority 2. Extract the next 2-3 high-ROI seams in `crates/fret-diag`

Why it is second:

- the command surface is usable enough now,
- the real medium-term risk is continued growth in a few orchestration-heavy files.

Current hotspots worth treating as explicit seam candidates:

- `crates/fret-diag/src/diag_suite.rs`
- `crates/fret-diag/src/diag_campaign.rs`
- `crates/fret-diag/src/diag_run.rs`
- `crates/fret-diag/src/commands/artifacts.rs`

Recommended next seam choices:

1. artifact resolution and materialization,
2. run planning/context assembly,
3. suite/campaign resolution and item expansion.

Recent progress since this note was drafted:

- artifact resolution/materialization now has shared seams for bundle input resolution and
  `script.result.json` discovery under `crates/fret-diag/src/commands/resolve.rs`,
- run planning/context assembly now reuses `ResolvedScriptPaths` and a higher-level
  `ResolvedRunContext` instead of re-threading parallel path and transport arguments,
- transport dispatch for the main script-driven launch flows now reuses shared filesystem transport
  helpers instead of repeating path override assembly inline,
- `diag_suite` now reuses a dedicated result-only helper, which closes the last duplicated
  `script.result` override path among the main orchestration commands.
- `diag_suite` also now routes suite input expansion, reuse-process env-default merging, and
  prewarm/prelude normalization through `ResolvedSuiteRunInputs`, turning a large inline block into
  a named seam that future `diag_campaign` work can reuse or mirror intentionally.

These choices align with the biggest orchestration churn surfaces while avoiding a premature rewrite.

### Priority 3. Stabilize suite/campaign metadata and evidence vocabulary

Why it is third:

- campaign metadata now exists, but the scaling story is not complete,
- evidence paths are already emitted, but the vocabulary is still only partially documented,
- future CI and GUI work will be easier if capability tags, flake policy, and evidence path terms are
  stabilized before more surfaces depend on them.

Recommended scope:

- capability/feature tags,
- flake policy vocabulary,
- evidence path naming and field expectations,
- stable reason-code expectations for orchestrated runs.

## What should be deferred for now

The following items are still valid, but they should not jump ahead of the priorities above:

- dashboard text or HTML projection,
- packing screenshot PNG bodies into combined failure zips,
- TOML or generated campaign manifests,
- removing legacy top-level `suites` / `scripts` compatibility,
- GUI-only workflow polish.

These are meaningful follow-ups once the contract and seam story are more settled.

## Recommended next implementation sequence

1. write the canonical artifact/evidence contract update,
2. continue with `diag_campaign` item expansion/context extraction or move to check planning/execution,
3. keep the new transport helpers thin instead of growing fresh inline path-assembly branches,
4. only then revisit optional output projections or larger packaging policies.

## Bottom line

The workstream is now past the point of proving that campaign automation is viable.

The next stage should optimize for:

- clearer contracts,
- smaller orchestration seams,
- and more stable scaling vocabulary,

not for more output formats or more GUI polish first.
