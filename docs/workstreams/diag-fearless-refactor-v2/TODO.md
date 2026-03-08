# Diag Fearless Refactor v2 — TODO

Status: Draft

Tracking doc: `docs/workstreams/diag-fearless-refactor-v2/README.md`

## M0 — Scope, vocabulary, and boundaries

- [ ] Confirm the umbrella positioning:
  - [ ] diagnostics is a general automation/debugging/evidence platform,
  - [ ] DevTools GUI is included as a consumer lane, not the architecture center.
- [x] Write a short “where does this change belong?” mapping for:
  - evidence: `docs/workstreams/diag-fearless-refactor-v2/CRATE_AND_MODULE_MAP.md`
  - [ ] protocol/contracts,
  - [ ] runtime service,
  - [ ] transport,
  - [ ] tooling engine,
  - [ ] presentation surfaces.
- [ ] Identify overlapping or stale diag workstream docs and classify them:
  - [ ] still active,
  - [ ] superseded but still useful,
  - [ ] migrate into this v2 folder,
  - [ ] retire later with redirects.

## M1 — Runtime and tooling seam cleanup

- [ ] Audit the main runtime/export modules and list remaining monolith hotspots.
- [ ] Audit `crates/fret-diag` orchestration entry points and list duplication hotspots.
- [x] Write a phased implementation roadmap that maps design docs to code landing order:
  - evidence: `docs/workstreams/diag-fearless-refactor-v2/IMPLEMENTATION_ROADMAP.md`
- [x] Choose the next 2?3 high-ROI seam extractions for landable follow-up PRs:
  - [x] run planning/context,
  - [x] artifact resolution/materialization,
  - [ ] check planning/execution,
    - first landing: `diag_suite` now extracts suite core default post-run checks into a dedicated helper, shrinking the main orchestration body around viewport/vlist/view-cache/retained defaults
    - second landing: `diag_suite` now extracts editor/markdown/text default post-run checks plus merge wiring into dedicated helpers, keeping policy-heavy boolean gate assembly out of `cmd_suite`
    - third landing: `diag_suite` now extracts the explicit-or-policy post-run trigger predicate into a dedicated helper, so trigger growth no longer expands inline beside bundle wait/doctor stages
    - fourth landing: `diag_suite` now extracts retained-vlist script override filtering into `SuiteScriptOverrideChecks`, so trigger planning and post-run application share the same per-script override seam
    - fifth landing: `diag_suite` now routes suite success/failure summary payload assembly and emission through `SuiteSummaryEmitInput` plus dedicated helpers, so setup failures, run failures, lint failures, and pass-result writing reuse one summary-write path
    - sixth landing: `diag_suite` now builds tooling-error rows and script-result rows through dedicated helpers, so setup/tooling/script/lint outcome payloads reuse one row-shaping path
    - seventh landing: `diag_suite` now routes stop-demo, summary emit, and return/exit decisions through dedicated failure-finalization helpers plus a shared summary context, so setup/run/lint failure branches reuse one cleanup + summary path
    - eighth landing: `diag_suite` now routes tooling failure script-result writes, row shaping, and main finalize wiring through dedicated helpers, so DevTools/connect/launch failure branches reuse one tooling-error bookkeeping path
    - ninth landing: `diag_suite` now routes failed/unexpected/lint-failed script outcomes through a dedicated exit helper, so those branches reuse one outcome row + finalize path
    - tenth landing: `diag_suite` now prepares per-script stage/reason accounting plus evidence/lint context through a dedicated helper, so each script iteration reuses one bookkeeping path after transport execution
    - eleventh landing: `diag_suite` now routes dump-label planning, `run_script_over_transport` lowering, and `tooling.suite.error` fallback through dedicated helpers, so transport result decoding reuses one path
    - next focus: script-execution block assembly for prewarm/prelude/load-script wiring
  - [ ] suite/campaign resolution,
    - first landing: `diag_suite` now uses `ResolvedSuiteRunInputs` for suite input normalization and env/default resolution
    - second landing: `diag_campaign` now uses a shared invocation builder for per-item `diag_suite::SuiteCmdContext` handoff
    - third landing: `diag_campaign` now uses explicit execution-plan helpers for per-run and batch output roots
    - fourth landing: `diag_campaign` now uses a unified `CampaignItemInvocation` builder for suite/script item dispatch
    - fifth landing: `diag_campaign` now uses `CampaignRunOutcome` for aggregate counters, output rendering, and command-failure collection in `cmd_campaign_run`
    - sixth landing: `diag_campaign` now separates item execution, finalize/summarize/share/result writing, and failed-item formatting inside `execute_campaign_inner`
    - seventh landing: `diag_campaign` now shares summarize/share timing through `CampaignSummaryArtifacts`, and batch execution reuses a matching finalize stage plus shared aggregate counters
    - eighth landing: `diag_campaign` now reuses shared `run`, `selection`, `aggregate`, and item-result JSON payload helpers across manifest/result writers
    - ninth landing: `diag_campaign` now reuses shared `resolved` and `counters` payload helpers across manifest/result writers
    - tenth landing: `diag_campaign` now reuses shared campaign-report JSON shaping and share/failure text helpers across run/result emitters and failure aggregation
    - eleventh landing: `diag_campaign` now reuses pure single-run and batch-run CLI output helpers inside `print_campaign_run_output`
    - twelfth landing: `diag_campaign` now reuses dedicated helpers for run-selection JSON, explicit/filter selection, and run-flag parsing
    - thirteenth landing: `diag_campaign` now reuses a dedicated subcommand resolver and `CampaignCmdContext` -> `CampaignRunContext` conversion boundary
    - fourteenth landing: `diag_campaign` now reuses dedicated normalization and report-construction helpers for `execute_campaign`
    - fifteenth landing: `diag_campaign` now reuses dedicated execution outcome/error helpers for summarize-failure priority and failed-item wording inside `execute_campaign_inner`
    - sixteenth landing: `diag_campaign` now reuses dedicated single-run and batch result payload helpers driven by `plan + summary_artifacts`, so result JSON assembly no longer depends on wide writer signatures
    - seventeenth landing: `diag_campaign` now reuses dedicated execution/batch artifact builders, so finalization no longer re-splices summarize/share outputs into outward-facing structs inline
    - eighteenth landing: `diag_campaign` now reuses a named aggregate-artifact contract for summary/index/share paths and summarize/share errors across finalization, batch artifacts, and result payload assembly
    - nineteenth landing: `diag_campaign` now reuses the same aggregate-artifact contract inside `CampaignExecutionReport`, so per-campaign report paths/share-export state no longer travel as a parallel shape
    - twentieth landing: `diag_campaign` now reuses a shared aggregate path-projection helper for report and batch JSON emitters, so summary/index/share visibility rules no longer drift across JSON output modes
    - twenty-first landing: `diag_campaign` now reuses dedicated counters/batch/runs helpers for top-level run-outcome JSON, so `campaign_run_outcome_to_json` no longer grows as another inline payload blob
    - twenty-second landing: `diag_campaign` now reuses dedicated status/paths/counters helpers for per-report JSON assembly, so `campaign_report_json` no longer grows as another long inline field-insertion block
    - twenty-third landing: `diag_campaign` now reuses dedicated root/paths/status helpers for batch JSON assembly, so `campaign_batch_to_json` no longer remains a one-off inline emitter blob beside the report JSON helpers
    - twenty-fourth landing: `diag_campaign` now reuses dedicated `run` and `aggregate` helpers across single-run and batch result payload assembly, so the two result payload writers no longer duplicate the same result-artifact section pair inline
    - twenty-fifth landing: `diag_campaign` now reuses dedicated manifest payload helpers across single-run and batch manifest writing, so `write_campaign_manifest` / `write_campaign_batch_manifest` mainly own output-path resolution plus file IO while manifest JSON shaping gains direct regression coverage
    - twenty-sixth landing: `diag_campaign` now separates per-item execution planning from suite-context assembly, so item kind/path/script-input selection no longer grows in the same helper that wires runtime flags and checks into `diag_suite::SuiteCmdContext`
    - twenty-seventh landing: `diag_campaign` now routes per-item suite success/error mapping through a dedicated item-result helper, so `run_campaign_item` no longer open-codes the same `CampaignItemRunResult` projection inline after each `diag_suite` execution
    - twenty-eighth landing: `diag_campaign` now separates batch item planning from plan consumption, so `execute_campaign_items` no longer mixes campaign-item enumeration with "run planned items" in the same loop body
    - twenty-ninth landing: `diag_campaign` now builds a shared summary-finalize plan for single-run and batch finalize paths, so summarize inputs, output roots, timestamps, and failure-share conditions stop being re-derived inline in two separate finalize branches
    - thirtieth landing: `diag_campaign` now builds dedicated result-write plans for single-run and batch result artifacts, so output-path resolution and payload shaping are settled before file IO and the write layer no longer duplicates the same "path + payload + write" pattern inline
    - thirty-first landing: `diag_campaign` now builds dedicated result-payload section bundles for single-run and batch result artifacts, so payload roots no longer open-code the same run/counters/aggregate/list section planning inline before composing the final JSON object
  - [x] transport dispatch.
  - evidence: `docs/workstreams/diag-fearless-refactor-v2/IMPLEMENTATION_ROADMAP.md`
- [ ] Define “no new blob growth” guardrails for follow-up work.

## M2 — Artifact and evidence consolidation

- [x] Document one canonical artifact model:
  - [x] bundle artifact,
  - [x] sidecars,
  - [x] `script.result.json`,
  - [x] `triage.json`,
  - [x] compact pack/AI packet style artifacts.
- [x] Define which artifacts are:
  - [x] source of truth,
  - [x] derived/cache-like,
  - [x] optional evidence,
  - [x] GUI-friendly projections.
- [x] Define a compatibility policy for artifact field additions/removals.
- [x] Define one bounded “first-open” artifact set for common triage.
  - evidence: `docs/workstreams/diag-fearless-refactor-v2/ARTIFACT_AND_EVIDENCE_MODEL_V1.md`

## M3 — Regression orchestration model

- [x] Write a first-pass campaign model for repo-level regression lanes:
  - [x] `smoke`
  - [x] `correctness`
  - [x] `matrix`
  - [x] `perf`
  - [x] `nightly/full`
  - evidence: `docs/workstreams/diag-fearless-refactor-v2/REGRESSION_CAMPAIGN_V1.md`
- [ ] Write a single vocabulary for regression lanes:
  - [ ] smoke,
  - [ ] correctness,
  - [ ] matrix,
  - [ ] perf,
  - [ ] nightly/full.
- [ ] Define suite metadata needed for scalable execution:
  - [x] first-pass campaign metadata is now present (`tier`, `owner`, `platforms`, `expected_duration_ms`, `tags`),
  - [ ] flake policy,
  - [ ] capability/feature tags.
- [x] Decide whether to introduce a first-class “campaign” orchestration layer.
  - [x] Land a minimal aggregation/index consumer first via `fretboard diag summarize`.
  - [x] Land a first `fretboard diag campaign` surface that composes existing `suite` + `summarize` flows.
  - [ ] Decide when campaign definitions should move from built-in Rust registry to external manifests.
- [ ] Define expected outputs for orchestrated runs:
  - [x] one machine-readable summary,
    - evidence: `docs/workstreams/diag-fearless-refactor-v2/REGRESSION_SUMMARY_SCHEMA_V1.md`
    - implementation: `diag suite`, `diag repeat`, `diag perf`, and `diag matrix`
      now emit `regression.summary.json`
  - [ ] stable reason codes,
  - [ ] evidence bundle/artifact paths,
    - in progress: current summary emitters already attach bounded artifact/evidence paths,
      but the path vocabulary still needs one explicit repo-level contract
  - [ ] optional compact pack for sharing.

## M4 — DevTools GUI alignment

- [ ] Define which GUI features belong in this workstream now:
  - [ ] artifact browser,
  - [ ] gate runner UX,
  - [ ] live inspect summaries,
  - [ ] script library/editor wiring,
  - [ ] resource subscriptions.
- [ ] Explicitly defer GUI-only polish that should not block core refactors.
- [ ] Ensure GUI uses the same contracts and artifact terminology as CLI/tooling.
- [x] Land a first GUI consumer over the shared aggregate artifacts:
  - [x] `apps/fret-devtools` now includes a read-only `Regression` details tab,
  - [x] the tab reads `regression.summary.json` and `regression.index.json` from the existing artifacts root,
  - [x] the tab exposes a manual refresh path without defining a parallel campaign model.
- [x] Add the first GUI drill-down over failing regression summaries:
  - [x] the `Regression` tab now lists `failing_summaries` from `regression.index.json`,
  - [x] selecting a row loads the corresponding `regression.summary.json`,
  - [x] selected summary path, first bundle dir, and bundle dir list can be copied for evidence follow-up.
  - [x] selected summary evidence can now be packed directly from the first failing bundle dir.
- [x] Add a thin GUI summarize trigger over the shared aggregate artifacts:
  - [x] the `Regression` tab now includes a `Summarize` action next to `Refresh`,
  - [x] the action runs the existing `diag summarize` flow against the current artifacts root,
  - [x] successful completion refreshes the aggregate artifacts instead of creating a GUI-only summary model.
- [x] Expose aggregate summary/index artifacts through the MCP consumer lane:
  - [x] `apps/fret-devtools-mcp` now exposes `regression.summary.json`,
  - [x] `apps/fret-devtools-mcp` now exposes `regression.index.json`,
  - [x] resources reuse the existing artifacts-root contract instead of defining a new store.
- [x] Add one end-to-end “dogfood” workflow that proves alignment:
  - [x] pick selector,
  - [x] patch or choose script,
  - [x] run,
  - [x] inspect artifacts,
  - [x] pack/share.
  - [x] Documented in `docs/workstreams/diag-fearless-refactor-v2/DEVTOOLS_GUI_DOGFOOD_WORKFLOW.md`.
- [x] Refresh the DevTools GUI enough for daily dogfooding:
  - [x] top-level workspace shell and footer status strip now read as a product surface,
  - [x] `Script Studio` now reads as one workflow (`Workflow Controls` + `Outputs & Bundles` + focused panes),
  - [x] `Regression` now uses a summary-first master/detail layout with inspector sections,
  - [x] failing summary rows now expose lane/failure/item badges for faster scanning,
  - [x] `Regression Workspace` now uses a clearer summary strip (`Aggregate Status` + `Primary Actions` + `Dashboard Preview`).
  - [x] Documented in `docs/workstreams/diag-devtools-gui-refresh-v1.md`.

## Next focus after GUI refresh

- [x] Define the first campaign/suite execution slice over existing diag scripts:
  - [x] Drafted command-surface and output-layout proposal in `docs/workstreams/diag-fearless-refactor-v2/CAMPAIGN_EXECUTION_ENTRY_V1.md`.
  - [x] Chose the CLI entry shape: `fretboard diag campaign`.
  - [x] Landed a minimal built-in campaign registry with `list` / `show` / `run`.
  - [x] Landed the minimum stable output layout for campaign runs:
    - `campaigns/<campaign_id>/<run_id>/campaign.manifest.json`
    - `campaigns/<campaign_id>/<run_id>/campaign.result.json`
    - `campaigns/<campaign_id>/<run_id>/suite-results/<suite>/...`
    - `campaigns/<campaign_id>/<run_id>/regression.index.json`
    - `campaigns/<campaign_id>/<run_id>/regression.summary.json`
  - [x] Kept DevTools and MCP on the same aggregate artifact handoff (`regression.index.json` + `regression.summary.json`).
- [ ] Expand the campaign surface beyond the first skeleton:
  - [x] move campaign definitions behind an explicit resolver seam (`registry/campaigns.rs`),
  - [x] promote that seam from built-in-only registry to manifest-backed resolver (`tools/diag-campaigns/*.json`),
  - [ ] decide whether to keep JSON-only or add TOML / generated registry inputs later,
  - [x] add first-pass campaign metadata (`owner`, `platforms`, `tier`, `expected_duration_ms`, `tags`),
  - [x] add direct script items in addition to suites,
  - [x] move canonical manifest authoring from top-level `suites`/`scripts` to ordered `items`,
  - [x] persist one batch artifact root for filtered or multi-id runs that resolve to multiple campaigns,
  - [ ] decide when legacy top-level `suites`/`scripts` compatibility can be removed,
  - [ ] decide whether campaign runs should emit a persisted dashboard text or HTML projection.
- [ ] Make failed automation runs leave predictable evidence by default:
  - [x] summary/index artifacts,
  - [x] first automatic failing evidence export for campaign and batch roots,
  - [x] first best-effort `triage.json` export through the share manifest path,
  - [x] first batch-level combined failure zip for handoff,
  - [ ] failing evidence bundles,
  - [x] copy/share-friendly paths.
- [x] Add first campaign discovery filters to keep selection scalable (`--lane`, `--tier`, `--tag`, `--platform`).
- [x] Extend those selectors into `diag campaign run` so filtered campaign batches can execute without enumerating ids by hand.
- [x] Add one thin maintainer note that explains the intended automation flow:
  - [x] author or choose script,
  - [x] run suite/campaign,
  - [x] inspect aggregate summary,
  - [x] pack/share evidence.


## M5 — Documentation consolidation

- [ ] Add a concise navigation note that tells contributors where to start for diag work.
- [ ] Cross-link existing v1/v1-architecture docs to this v2 umbrella where appropriate.
- [ ] Record migration intent for large existing diag docs rather than duplicating content forever.
- [x] Document the first aggregate dashboard/index fields for consumers:
  - [x] counters by lane/status/tool/reason,
  - [x] top reason codes,
  - [x] failing summaries ranking.
- [x] Land one thin consumer over the aggregate index:
  - [x] `fretboard diag dashboard` reads `regression.index.json`,
  - [x] default output gives a first-open human summary,
  - [x] `--json` preserves machine-readable access to the full index.
- [ ] Add a short maintainer checklist for new diagnostics features:
  - [ ] which layer changes,
  - [ ] what gate must be added,
  - [ ] what evidence should be left behind,
  - [ ] what docs must be updated.

## M6 — Debt removal and enforcement

- [ ] Identify duplicated logic that should be removed only after seam adoption is proven.
- [ ] Add at least one regression gate or lint/test expectation for each major seam migration.
- [ ] Define “done” criteria for retiring older diag notes or compatibility shims.
- [ ] Keep a visible debt list so future refactors stay incremental instead of reverting to ad-hoc growth.
