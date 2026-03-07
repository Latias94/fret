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
- initial landing now exists in
  `docs/workstreams/diag-fearless-refactor-v2/ARTIFACT_AND_EVIDENCE_MODEL_V1.md`.
- the next success condition is adoption rather than more drafting: cross-link the contract from
  `docs/README.md`, `docs/ui-diagnostics-and-scripted-tests.md`, and maintainer-facing flow notes so
  CLI / GUI / CI consumers inherit the same vocabulary.

### Priority 2. Extract the next 2-3 high-ROI seams in `crates/fret-diag`

Why it is second:

- the command surface is usable enough now,
- the real medium-term risk is continued growth in a few orchestration-heavy files.

Current hotspots worth treating as explicit seam candidates:

- `crates/fret-diag/src/diag_suite.rs` (summary/failure emit factoring and per-row payload shaping have landed; the next target inside this file is failure-finalization helpers for stop/emit/return-exit paths)
- `crates/fret-diag/src/diag_campaign.rs`
- `crates/fret-diag/src/diag_run.rs`
- `crates/fret-diag/src/commands/artifacts.rs`

Recommended next seam choices:

1. artifact resolution and materialization,
2. run planning/context assembly,
3. suite/campaign resolution and item expansion.

Recent progress since this note was drafted:

- the first repo-level artifact/evidence contract now exists in
  `ARTIFACT_AND_EVIDENCE_MODEL_V1.md`, which names source-of-truth artifacts, derived/index
  artifacts, optional evidence, presentation-facing projections, the bounded first-open set, and
  consumer checklists for CLI / GUI / CI / share flows in one place,
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
- `diag_suite` now also builds core default post-run checks through a dedicated helper, so viewport,
  vlist, retained-host, view-cache, and gc-liveness defaults no longer sprawl across the main
  command body.
- `diag_suite` now also builds editor/markdown/text default post-run checks through dedicated
  helper + merge seams, which keeps policy-heavy boolean gate assembly out of `cmd_suite` and makes
  future text/IME audit work easier to land without reopening one orchestration blob.
- `diag_suite` now also routes explicit-or-policy post-run trigger decisions through a named helper,
  and retained-vlist script-specific overrides now flow through `SuiteScriptOverrideChecks`, so the
  trigger predicate and post-run application share one per-script override vocabulary.
- `diag_suite` now also routes suite success/failure summary payload assembly and emission through
  `SuiteSummaryEmitInput` + dedicated helpers, so setup failures, run failures, lint failures, and
  pass-result writing no longer duplicate the same `suite.summary.json` / `regression.summary.json`
  plumbing inline.
- `diag_suite` now also builds tooling-error rows and script-result rows through dedicated helpers,
  so setup/tooling/script/lint outcome payloads stop open-coding the same JSON fragments inline
  beside execution control flow.
- `diag_campaign` now routes per-item `diag_suite::SuiteCmdContext` construction through a
  shared invocation builder, so suite items and script items no longer maintain parallel handoff
  structs inline.
- `diag_campaign` now also routes campaign roots, batch roots, and summary/index destinations
  through explicit execution-plan helpers, so manifest/summarize/share wiring no longer re-derives
  those paths inline.
- `diag_campaign` item dispatch now uses a single `CampaignItemInvocation` builder, so suite
  items and script items share one `diag_suite` handoff path instead of duplicating nearly identical
  branch bodies.
- `diag_campaign` run orchestration now uses an explicit `CampaignRunOutcome` seam for selection
  execution, aggregate counters, output rendering, and command-failure collection, which keeps
  `cmd_campaign_run` closer to a thin command adapter.
- `diag_campaign` per-campaign execution now separates item dispatch, finalize/summarize/share
  writing, and failed-item error formatting, reducing another orchestration blob in
  `execute_campaign_inner`.
- `diag_campaign` now also shares summarize/share timing through a small `CampaignSummaryArtifacts`
  seam, and batch execution mirrors the same finalize staging instead of keeping a second inline
  summarize/share/result block.
- `diag_campaign` manifest/result writers now reuse shared JSON-fragment helpers for `run`,
  `selection`, `aggregate`, and item-result payloads, reducing record-shape drift across the file
  emitters.
- `diag_campaign` now also routes `resolved` and `counters` payload assembly through named helpers,
  so item/campaign totals are computed in one place instead of being re-derived across manifest and
  result emitters.
- `diag_campaign` report JSON shaping and share/failure text formatting now also live behind small
  helpers, which reduces drift between run-output JSON, artifact JSON, and command-failure wording.
- `diag_campaign` human-readable run output now flows through pure single-run and batch-run output
  helpers, which keeps `print_campaign_run_output` thin and makes the CLI output shape easier to
  regression-test without shell capture indirection.
- `diag_campaign` run-selection JSON, explicit/filter selection, and run-flag parsing now also sit
  behind dedicated helpers, which trims more command-adapter glue from the top-level orchestration
  path and makes parse/selection edge cases cheaper to test directly.
- `diag_campaign` top-level dispatch now also resolves subcommands through a small enum helper and
  converts `CampaignCmdContext` into `CampaignRunContext` via a dedicated boundary, making the
  command adapter more obviously separate from the run-time orchestration context.
- `diag_campaign` execution result mapping now also runs through dedicated normalization and report
  construction helpers, which removes another tuple-shaped conversion seam from the main execution
  path.
- `diag_campaign` now also builds per-campaign execution outcomes through dedicated outcome/error
  helpers, so summarize-failure priority, failed-item wording, and failure-summary formatting no
  longer stay inline inside `execute_campaign_inner`.
- `diag_campaign` single-run and batch result writing now also flow through dedicated payload
  builders that consume `plan + summary_artifacts`, which removes another pair of wide
  `write_*result` signatures and keeps result JSON shaping testable without file IO.
- `diag_campaign` now also builds final execution/batch outward artifacts through dedicated builder
  helpers, so summarize/share outputs stop being manually re-spliced when finalization crosses from
  summary generation into outward command-facing structs.
- `diag_campaign` now also routes summary/index/share/summarize-error/share-error through a named
  aggregate-artifact contract, which reduces field drift between batch artifacts, finalization,
  and result JSON assembly.
- `diag_campaign` now also stores per-campaign report artifact paths and share-export state through
  the same aggregate contract, which removes another parallel path/error shape from
  `CampaignExecutionReport` and keeps report JSON/output helpers aligned with batch/finalization.
- `diag_campaign` now also routes report and batch JSON path projection through a shared aggregate
  helper, so summary/index/share path visibility rules no longer drift between run-outcome JSON and
  artifact-style JSON emitters.
- `diag_campaign` now also routes run-outcome JSON through dedicated counters/batch/runs helpers,
  which removes another inline payload blob from `campaign_run_outcome_to_json` and makes the top-
  level CLI JSON projection cheaper to regression-test directly.
- `diag_campaign` now also splits `campaign_report_json` into dedicated status/paths/counters
  sections, so per-report JSON assembly no longer grows as one long field-insertion block and each
  projection slice can be regression-tested in isolation.
- `diag_campaign` now also splits `campaign_batch_to_json` into dedicated root/paths/status
  sections, so batch JSON projection follows the same decomposition style as per-report JSON instead
  of keeping one more inline emitter blob alive.
- `diag_campaign` now also reuses dedicated `run` and `aggregate` helpers across single-run and
  batch result payload assembly, which removes another repeated result-artifact section pair from
  `campaign_result_payload` / `campaign_batch_result_payload`.
- `diag_campaign` campaign and batch manifest writing now also flow through dedicated manifest
  payload helpers, so `write_campaign_manifest` / `write_campaign_batch_manifest` mostly own path
  resolution plus file IO while the manifest JSON shape gains direct regression coverage.
- `diag_campaign` per-item execution now also splits item planning from suite-context assembly
  through a small execution-plan seam, so item kind/path/script-input selection no longer grows in
  the same helper that wires runtime flags and checks into `diag_suite::SuiteCmdContext`.
- `diag_campaign` per-item execution now also routes `diag_suite` success/error mapping through a
  dedicated item-result helper, so `run_campaign_item` no longer open-codes the same output-shape
  projection inline after every suite execution.
- `diag_campaign` per-item execution now also separates batch item planning from plan consumption,
  so `execute_campaign_items` no longer mixes "enumerate campaign items" with "run planned items"
  in the same loop body.
- `diag_campaign` single-run and batch finalize paths now also build a shared summary-finalize plan
  before summarize/share work begins, so finalize orchestration no longer re-derives summarize
  inputs, output roots, and failure-share conditions inline in two separate places.
- `diag_campaign` single-run and batch result writing now also build dedicated result-write plans,
  so output-path resolution and payload shaping are decided before file IO and the writer layer no
  longer duplicates the same "path + payload + write" pattern inline.
- `diag_campaign` single-run and batch result payload assembly now also builds dedicated section
  bundles before composing the final JSON object, so payload roots no longer open-code the same
  run/counters/aggregate/list section planning inline.

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

1. apply and cross-link the canonical artifact/evidence contract update across remaining diagnostics notes and consumers,
2. continue from the new `diag_campaign` invocation + execution-plan + run-outcome + shared-finalize + shared-payload + outcome/error + result-payload + artifact-builder + aggregate-artifact + report-artifact + aggregate-projection + run-outcome-json + report-json-section + batch-json-section + result-section + manifest-payload + item-execution-plan + item-result + item-plan-list + summary-finalize-plan + result-write-plan + result-payload-sections seams into the remaining JSON consolidation or check planning/execution,
3. keep the new transport helpers thin instead of growing fresh inline path-assembly branches,
4. only then revisit optional output projections or larger packaging policies.

## Bottom line

The workstream is now past the point of proving that campaign automation is viable.

The next stage should optimize for:

- clearer contracts,
- smaller orchestration seams,
- and more stable scaling vocabulary,

not for more output formats or more GUI polish first.
