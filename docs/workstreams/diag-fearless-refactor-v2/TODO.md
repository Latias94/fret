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

- [ ] Execute the current near-term seam order:
  - [ ] finish only the remaining reviewable tail in `commands::artifacts`,
  - [x] extract presentation-surface reuse over the aggregate artifact model once the artifact/materialization seams settle,
  - [ ] keep `commands::resolve` parked unless another clearly reviewable small seam appears.
- [ ] Audit the main runtime/export modules and list remaining monolith hotspots.
- [ ] Audit `crates/fret-diag` orchestration entry points and list duplication hotspots.
- [x] Write a phased implementation roadmap that maps design docs to code landing order:
  - evidence: `docs/workstreams/diag-fearless-refactor-v2/IMPLEMENTATION_ROADMAP.md`
- [x] Choose the next 2?3 high-ROI seam extractions for landable follow-up PRs:
  - [x] run planning/context,
    - ninth landing: `diag_run` now routes transport result stage normalization, bundle doctor/post-run checks, bundle artifact emission, and demo-exit-killed marking through dedicated helpers, so the shared result/post-run tail no longer stays duplicated across the DevTools WS and filesystem branches
    - tenth landing: `diag_run` now routes failed-run dump bundle backfill, bundle-path resolution/wait, and filesystem post-run finalization through dedicated helpers, so `cmd_run` no longer open-codes the same bundle wait/retry and post-run artifact flow inline in the filesystem branch
    - eleventh landing: `diag_run` now routes the DevTools WS path through a dedicated branch adapter, so command-level WS setup, connect/run handling, and result finalization no longer live inline beside the filesystem branch in `cmd_run`
    - twelfth landing: `diag_run` now routes the filesystem path through a dedicated branch adapter, so env/default wiring, launch/connect setup, stop-guard ownership, and branch-local finalization no longer live inline in `cmd_run`
    - thirteenth landing: `diag_run` now routes the remaining top-level option/policy setup through `RunCommandSetupRequest` and `PreparedRunCommandSetup`, so argument normalization, check-derived defaults, pack/bundle intent derivation, and transport-mode validation stop sharing one long inline prelude in `cmd_run`
    - next focus: treat `diag_run` as near diminishing returns and shift the next high-ROI seam to `diag_campaign` or artifact-resolution/materialization unless a new `cmd_run` holdout appears
  - [x] artifact resolution/materialization,
    - latest landing: `commands/resolve` now routes `diag resolve latest` through dedicated option-parse plus JSON/text render helpers, the deeper session path now routes target session-id selection plus existing-session directory validation through dedicated helpers, `resolve_script_result_json_path_or_latest` now routes latest-vs-src search-start selection through dedicated helpers, and the shared bundle resolution path now routes source-path selection, bundle-ref derivation, and artifacts-root policy through dedicated helpers, so top-level command parsing/output shaping plus session/script-result/bundle normalization no longer stay intertwined inline and the output contract gains direct helper-level regression coverage
    - latest landing: `commands::artifacts` now routes `cmd_pack` through a dedicated setup helper plus pure default-out-path logic, the repeated single-bundle emitters now reuse shared bundle-input plus path/json output helpers, `cmd_meta` now routes its human-readable projection through pure report-line helpers, `cmd_lint` now routes bundle/out-path preparation plus exit-policy dispatch through dedicated helpers while reusing the shared JSON artifact writer, `cmd_triage` now routes payload assembly through dedicated lite/full builders plus a tooling-warning attachment helper, and `commands/artifact.rs` now routes `cmd_artifact_lint` through dedicated setup/write/exit helpers, so shared validation, simple artifact emission, lint write/display policy, triage payload shaping, artifact-lint write/exit policy, and meta-report rendering no longer stay duplicated or inline-only across multiple commands
    - next focus: treat `commands/resolve` as near diminishing returns and re-evaluate whether the next higher-ROI seam now lives in `diag_campaign` or another artifact/materialization holdout
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
    - twelfth landing: `diag_suite` now routes prewarm/prelude execution and load-script wiring through a dedicated execution-block context, so the script loop reuses one setup path before transport execution
    - thirteenth landing: `diag_suite` now routes per-script launch env/default assembly and connected transport acquisition through `SuiteScriptLaunchRequest` plus `SuiteScriptTransportRequest` / `SuiteScriptTransportSelection`, so `maybe_launch_demo` and filesystem-vs-DevTools selection reuse one seam
    - fourteenth landing: `diag_suite` now routes transport-backed execution dispatch through `SuiteScriptExecutionRequest`, so `SuiteScriptExecutionBlockContext` assembly plus `execute_suite_script_iteration_block` invocation reuse one seam
    - fifteenth landing: `diag_suite` now routes per-script lint execution plus passed-script post-run preparation through `SuiteScriptLintRequest` and `SuiteScriptPostRunPreparationRequest`, so bundle waits, bundle doctor application, lint/report wiring, and post-run default-check planning reuse one seam
    - sixteenth landing: `diag_suite` now routes per-script result finalization through `SuiteScriptStageFinalizeRequest` and `SuiteScriptSuccessFinalizeRequest`, so stage branching, success-row emission, and stop-demo teardown reuse dedicated helpers instead of sharing one inline tail
    - seventeenth landing: `diag_suite` now routes the remaining per-script success tail through `SuiteScriptSuccessTailRequest`, so lint-failure exit, post-run apply, and success finalize orchestration reuse one helper seam instead of re-expanding the last success-only block inline
    - audit note: the remaining `diag_suite` holdouts are now mostly one-time setup and session-root-adjacent helpers, so further slicing here has lower ROI than the next command hotspot
    - next focus: keep `diag_suite` parked unless a new high-ROI holdout appears; the active seam work has moved to `diag_run`
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
    - latest landing: `diag_campaign` now routes `write_campaign_share_manifest` through dedicated item-planning, payload-build, and combined-zip-finalize helpers plus named counters/combined-entry/outcome shapes, so bundle/triage/share staging and final manifest update no longer expand inline in one artifact-handoff block
    - latest landing: `diag_campaign` now routes `execute_campaign_run_selection` through a dedicated `build_campaign_run_outcome` helper for counters plus command-failure aggregation, so selection/execution/batch-artifact orchestration no longer recomputes those outcome fields inline after report collection
    - latest landing: `diag_campaign` now routes summary-finalize execution through `execute_campaign_summary_finalize_outcome` and timing/materialization through `build_campaign_summary_artifacts`, so single-run and batch finalize flows reuse the same summarize/share outcome seam instead of reassembling it inline inside `finalize_campaign_summary_artifacts`
    - latest landing: `diag_campaign` now builds a dedicated `CampaignBatchArtifactWritePlan` plus `build_campaign_batch_manifest_write_plan`, so batch manifest output-path/payload shaping and summary-finalize setup are settled before IO rather than being re-derived inline inside `write_campaign_batch_artifacts`
    - latest landing: `diag_campaign` now builds a dedicated `CampaignExecutionStartPlan` plus `build_campaign_manifest_write_plan`, so single-campaign execution-plan and manifest setup are settled before IO rather than being re-threaded inline across `execute_campaign` and `execute_campaign_inner`
    - latest landing: `diag_campaign` now builds a dedicated `CampaignExecutionFinalizePlan`, so failure counting plus summary-finalize setup are settled before finalize IO rather than being re-derived inline inside `finalize_campaign_execution`
    - latest landing: `diag_campaign` now routes result normalization plus report construction through `build_campaign_execution_report_from_outcome_result`, so `execute_campaign` no longer rethreads that outcome/report handoff inline after startup execution returns
    - latest landing: `diag_campaign` now builds `CampaignShareManifestItems` plus `CampaignShareManifestItemArtifacts`, so `write_campaign_share_manifest` no longer owns the per-item aggregation loop inline and `build_campaign_share_manifest_item` now separates artifact IO from run-entry shaping with helper-level regression coverage
    - latest landing: `diag_campaign` now builds a dedicated `CampaignShareManifestWritePlan` and finalizes it through a share-manifest handoff helper, so initial payload/output-path planning and combined-zip update no longer live inline in `write_campaign_share_manifest`
    - latest landing: `diag_campaign` now splits share-item artifact planning into dedicated bundle-dir, supporting-artifact, and share-zip helpers, so `collect_campaign_share_manifest_item_artifacts` no longer expands triage, screenshots, and AI-packet zip work in one block and each segment gains direct helper-level regression coverage
    - latest landing: `diag_campaign` now builds share-manifest payload sections through dedicated source/selection/counters/share helpers, so `build_campaign_share_manifest_payload` no longer open-codes the full JSON object shape inline and payload-section shaping gains direct regression coverage
    - latest landing: `diag_campaign` now routes share-manifest combined-zip mutation through dedicated field-building and share-section apply helpers, so `apply_campaign_share_manifest_combined_zip` no longer splices the update values inline and final share-field mutation gains direct helper-level regression coverage
    - latest landing: `commands::artifacts` now routes meta canonical-path/default-out resolution through dedicated meta-sidecar, bundle-dir, and bundle-path helpers, so `resolve_meta_artifact_paths` no longer mixes all three source kinds inline and each path gains direct helper-level regression coverage
    - latest landing: `commands::resolve` now routes latest run and bundle projection through dedicated helpers, so `resolve_latest_for_out_dir` and `resolve_latest_bundle_dir_from_base_or_session_out_dir` no longer re-derive the same latest-bundle/run fields inline and the projection logic gains direct helper-level regression coverage
    - latest landing: `commands::resolve` now routes `script.result.json:last_bundle_dir` hint normalization and `latest.txt_or_scan` fallback through dedicated helpers, so `resolve_latest_bundle_dir_for_out_dir` now mostly owns orchestration and relative/absolute hint plus missing-dir fallback behavior gains direct regression coverage
    - latest landing: `commands::resolve` now routes session-root vs base-dir selection through a dedicated mode helper plus direct-resolution builder, so `resolve_session_out_dir_for_base_dir` no longer mixes branch policy with output shaping inline and nested-session marker preference plus direct-without-sessions behavior gain direct regression coverage
    - latest landing: `commands::artifacts` now routes canonical artifact materialization through a shared helper reused by `cmd_meta` and `cmd_test_ids`, so canonical-vs-custom output copy/no-op behavior no longer lives inline in each command and same-path / existing-file / copy-to-nested-out behavior gains direct regression coverage
    - latest landing: `commands::artifacts` now routes repeated ensured-artifact emitters through a shared helper reused by `cmd_test_ids_index` and `cmd_frames_index`, so ensure-then-emit orchestration no longer lives inline in each command and success / ensure-error / emit-error behavior gains direct regression coverage
    - latest landing: `commands::artifacts` now routes triage bundle resolution plus default out-path selection through a dedicated prepare helper, so `cmd_triage` no longer mixes request parsing, bundle-ref resolution, and out-path policy inline before payload construction and lite/default-vs-custom out behavior gains direct regression coverage
    - latest landing: `commands::artifacts` now routes artifact text/JSON reads plus pretty JSON text shaping through dedicated output helpers, so `emit_path_or_json_output`, `emit_artifact_output`, and `write_json_artifact_output` no longer duplicate file-read / JSON-parse / pretty-text formatting logic and output-surface behavior gains direct regression coverage
    - latest landing: `commands::artifacts` now routes test-ids bundle/out preparation through a dedicated helper plus an existing-file shortcut helper, so `cmd_test_ids` no longer mixes input parsing, default-vs-custom out selection, and cached-output short-circuit inline before canonical materialization and the prepare/shortcut behavior gains direct regression coverage
    - latest landing: `commands::artifacts` now routes meta parse/resolve/out preparation through a dedicated helper, so `cmd_meta` no longer mixes request parsing, canonical-path resolution, and default-vs-custom out selection inline before materialize/emit and the prepare-stage behavior gains direct regression coverage
    - latest landing: `commands::artifacts` now routes lint report write + exit decision through a dedicated helper, so `cmd_lint` no longer mixes JSON artifact emission with exit-policy computation inline after lint execution and report-output / exit-required behavior gains direct regression coverage
    - latest landing: `commands::artifacts` now routes pack ai.packet path resolution, best-effort ensure, and `--ai-only` directory validation through dedicated helpers, so `cmd_pack` no longer repeats ai.packet path joins and preflight branches inline before zip emission and pack preflight behavior gains direct regression coverage
    - latest landing: `commands::artifacts` now routes generated artifact materialize-plus-emit handoff through a shared helper reused by `cmd_test_ids` and `cmd_meta`, while path/json mode selection now also routes through a dedicated helper, so the existing-output reuse vs canonical-materialize tail no longer lives inline in each command and helper-level regression coverage now locks reuse and copy behavior directly
    - latest landing: `commands::artifacts` now routes `cmd_pack` zip execution through a dedicated plan plus execution helper, so ai-only vs full-bundle zip dispatch, default sort normalization, and final zip handoff no longer stay in one command-level branch block and helper-level regression coverage now locks ai-only mode plus bundle default-sort planning directly
    - latest landing: `commands::artifacts` now routes `meta_report_lines` through dedicated summary/window presentation helpers, so the report surface no longer mixes headline rendering with per-window truncation/formatting in one function and helper-level regression coverage now locks semantics summary, missing considered-frame fallback, and truncation behavior directly
    - latest landing: `commands::artifacts` now routes triage payload selection/finalize through dedicated mode and finalize helpers, so lite-vs-full selection, full-sort defaulting, and tooling-warning handoff no longer stay interleaved in one builder and helper-level regression coverage now locks lite metric selection, full default sort, and finalize no-op behavior directly
    - latest landing: `commands::artifacts` now routes path/json/meta-report emission through a shared `ArtifactOutputPresentation` surface plus dedicated builder helpers, so file-write output, emitted-artifact display, and meta-report line printing no longer each own their own terminal rendering path and helper-level regression coverage now locks JSON text/path projection and meta-report line projection directly
    - latest landing: `commands::artifacts` now routes lint report file-write plus output projection through a dedicated `LintReportOutput` seam built on the shared JSON-write presentation surface, so lint payload persistence and exit-required bookkeeping no longer stay interleaved in one helper and helper-level regression coverage now locks path presentation plus exit-flag shaping directly
    - latest landing: `diag_summarize` now routes aggregate command output through a dedicated `SummarizeOutputPresentation` seam plus failed-count and success-line helpers, so JSON-vs-human summarize projection no longer lives inline in `cmd_summarize` and helper-level regression coverage now locks aggregate failed-count accounting and output text shaping directly
    - latest landing: `diag_dashboard` now routes dashboard CLI output through a dedicated `DashboardOutputPresentation` seam plus counter/reason/failing-summary line builders, so JSON-vs-human dashboard projection no longer lives inline in `cmd_dashboard` and helper-level regression coverage now locks counter sections, aggregate summary lines, and failing-summary row shaping directly
    - latest landing: `diag_campaign` now routes campaign-run CLI output through a dedicated `CampaignRunOutputPresentation` seam, so JSON-vs-human output selection no longer lives inline in `print_campaign_run_output` and helper-level regression coverage now locks both JSON text projection and single-run human line shaping directly
    - latest landing: `apps/fret-devtools-mcp` now reuses shared aggregate dashboard projection and human-summary helpers exported from `fret-diag`, so MCP no longer owns a parallel dashboard counter/reason/failing-summary parser or text assembly path and CLI/MCP now share one aggregate presentation vocabulary
    - latest landing: `apps/fret-devtools` now also reuses shared aggregate dashboard projection, human-summary lines, and failing-summary row parsing exported from `fret-diag`, so the GUI regression tab no longer owns another parallel dashboard text assembly path and CLI/MCP/GUI now share one aggregate dashboard vocabulary
    - latest landing: `commands::artifacts` now routes `cmd_triage` through a dedicated `TriageCommandOutput` seam built on the shared JSON-write presentation helper, so triage payload generation and terminal emission no longer stay interleaved in the command entry and helper-level regression coverage now locks both path and JSON text projection directly
    - latest landing: `commands::artifacts` now routes `cmd_triage` through a dedicated `TriageExecutionPlan`, so payload mode selection, stats-top/warmup shaping, and JSON-vs-path output projection no longer stay spread across the command entry and helper signatures and helper-level regression coverage now locks lite metric carry-through plus full-sort defaulting directly
    - latest landing: `commands::artifacts` now routes ensured-artifact commands through a dedicated `EnsuredBundleArtifactOutput` seam, so `ensure + display-mode projection + emit` no longer stay interleaved in one helper and helper-level regression coverage now locks both path and JSON text projection for `cmd_test_ids_index` / `cmd_frames_index`
    - latest landing: `commands::artifacts` now routes ensured-artifact commands through a dedicated `EnsuredBundleArtifactPlan`, so required-input resolution, warmup/display shaping, and ensure-output projection no longer stay mixed in one helper and helper-level regression coverage now locks plan shaping separately from execution
    - latest landing: `commands::artifacts` now routes required-bundle ensured-artifact commands through a dedicated `build_required_bundle_artifact_output` seam, so `resolve input + ensure output + presentation build` no longer stay interleaved inside `cmd_test_ids_index` / `cmd_frames_index` and helper-level regression coverage now locks missing-hint propagation plus resolved-input handoff directly
    - latest landing: `commands::artifacts` now routes repeated `bundle input + optional custom out + default out` setup through shared prepare helpers, so `cmd_lint`, `cmd_test_ids`, `cmd_triage`, and `cmd_meta` no longer each inline their own custom-vs-default out resolution and helper-level regression coverage now locks both default and custom-out handoff directly
    - latest landing: `commands::artifacts` now routes generated-artifact materialization through a dedicated `GeneratedArtifactOutput` seam, so `cmd_test_ids` / `cmd_meta` no longer rely on a monolithic materialize-and-emit helper and helper-level regression coverage now locks reuse-existing vs copy-missing materialization directly
    - latest landing: `commands::artifacts` now routes generated-artifact copy/no-op/reuse decisions through a dedicated `ArtifactMaterializationPlan`, so existing-out reuse, canonical-path no-op, and copy-to-custom-out execution no longer stay hidden inside one helper body and helper-level regression coverage now locks each branch directly
    - latest landing: `commands::artifacts` now routes `cmd_test_ids` through a dedicated `TestIdsExecutionPlan`, so generated-artifact display mode, `warmup_frames`, and `max_test_ids` shaping no longer stay coupled to the command tail and helper-level regression coverage now locks prepared-field carry-through plus output projection directly
    - latest landing: `commands::artifacts` now routes `cmd_meta` through a dedicated `MetaExecutionPlan`, so canonical-path handoff, output target, and display-mode projection no longer stay coupled to the command tail and helper-level regression coverage now locks prepared-field carry-through plus path-presentation materialization directly
    - latest landing: `commands::artifacts` now routes `cmd_pack` through a dedicated `PackCommandOutput` seam, so plan execution and terminal path presentation no longer stay coupled to a one-off `println!` tail and helper-level regression coverage now locks pack output projection directly
    - latest landing: `commands::artifacts` now lets `cmd_lint` consume `build_lint_report_output` directly, so lint report write, output presentation, and exit-policy handoff now follow the same build-then-emit pattern as the rest of the command family instead of going through a dedicated side-effect helper
    - latest landing: `commands::artifacts` now routes meta-sidecar validity checks and bundle-dir sidecar preference through dedicated helpers, so direct-vs-`_root` selection and existing-sidecar reuse no longer stay open-coded across both meta resolution branches and helper-level regression coverage now locks the preference order directly
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
  - [x] first repo-level contract drafted in `M3_ORCHESTRATION_VOCABULARY_AND_CONTRACT_V1.md`,
  - [x] smoke,
  - [x] correctness,
  - [x] matrix,
  - [x] perf,
  - [x] nightly/full,
  - [x] persisted artifacts now prefer `nightly` while keeping `full` as a readable legacy alias,
  - [x] first persisted-field normalization map is now documented in
    `M3_ORCHESTRATION_VOCABULARY_AND_CONTRACT_V1.md`,
  - [x] writer/reader alias lifecycle is now documented in
    `M3_ORCHESTRATION_VOCABULARY_AND_CONTRACT_V1.md`,
  - [x] first repo-level adoption order is now documented in
    `M3_ORCHESTRATION_VOCABULARY_AND_CONTRACT_V1.md`,
  - [x] first bounded implementation audit captured in
    `M3_VOCABULARY_ADOPTION_AUDIT.md`,
  - [ ] remaining persisted residual names still need additive adoption review,
    - [x] campaign metadata now has additive `requires_capabilities` / `flake_policy` adoption in
      the campaign manifest/registry contract and CLI presentation,
    - [x] `RegressionArtifactsV1.index_json` / `perf_summary_json` / `compare_json` now have
      explicit canonical-vs-projection classification in the M3 and artifact-model notes.
- [ ] Define suite metadata needed for scalable execution:
  - [x] first-pass campaign metadata is now present (`tier`, `owner`, `platforms`, `expected_duration_ms`, `tags`),
  - [x] first-pass flake policy vocabulary documented in `M3_ORCHESTRATION_VOCABULARY_AND_CONTRACT_V1.md`,
  - [x] first-pass capability-tag naming rules documented in `M3_ORCHESTRATION_VOCABULARY_AND_CONTRACT_V1.md`,
  - [x] bounded execution-adoption audit captured in
    `CAMPAIGN_METADATA_EXECUTION_ADOPTION_AUDIT.md`,
  - [x] first-pass campaign capability preflight design captured in
    `CAMPAIGN_CAPABILITY_PREFLIGHT_V1.md`,
  - [x] first-pass non-filesystem capability-source direction captured in
    `NON_FILESYSTEM_CAPABILITY_SOURCE_V1.md`,
  - [x] bounded additive implementation plan for capability provenance captured in
    `CAPABILITY_PROVENANCE_MINIMAL_IMPLEMENTATION_V1.md`,
  - [~] keep `requires_capabilities` / `flake_policy` passive until campaign-level preflight or
    retry orchestration has a concrete consumer.
    - [x] campaign-level capability preflight is now partially landed,
    - [x] campaign policy skips now surface as `skipped_policy` + `capability.missing`,
    - [x] batch counters now expose `campaigns_skipped_policy`,
    - [x] capability-source resolution is now shared between campaign preflight and
      `diag doctor`,
    - [x] additive `capability_source` payloads are now emitted by `diag doctor`,
      campaign preflight summary evidence/metadata, and campaign aggregate/result payloads,
    - [x] DevTools `Regression` drill-down now surfaces `Capability Sources` separately from
      campaign-local capability check artifacts,
    - [x] MCP regression dashboard output now surfaces capability provenance and
      `capabilities_check_path` from the sibling `regression.summary.json` when available,
    - [ ] `flake_policy` still remains passive metadata.
- [x] Record the DevTools/MCP raw-JSON defer boundary in
  `DEVTOOLS_MCP_RAW_JSON_DEFER_AUDIT.md`.
- [x] Tighten the M3 orchestration vocabulary before more tooling grows around it:
  - [x] stable reason-code naming and bucket rules,
  - [x] one repo-level artifact/evidence path vocabulary,
  - [x] flake policy wording aligned with campaign/suite outputs,
  - [x] capability tags and lane naming kept consistent at the contract level across CLI, docs, and aggregate artifacts,
  - [x] shared summary serialization now writes canonical lane/evidence vocabulary while accepting
    legacy aliases,
  - [x] summarize/dashboard human wording now uses canonical lane/counter wording,
  - [x] campaign share payload and run-manifest `paths` now start adopting canonical artifact field
    names additively,
  - [ ] residual manifest/consumer naming still needs one explicit pass,
    - [x] run-manifest `files[].id` now writes canonical `script_result` while retaining legacy read
      compatibility for `script_result_json`,
    - [x] Layer B `bundle_artifact` additive adoption is now landed in `diag_repro`,
      `diag_repeat`, and the first two stats payload batches,
    - [x] `stats/ui_gallery_code_editor.rs` now also uses the shared helper for canonical-first
      `bundle_artifact` + legacy `bundle_json` dual-write,
    - [x] `stats/ui_gallery_markdown_editor.rs` now also uses the shared helper for canonical-first
      `bundle_artifact` + legacy `bundle_json` dual-write,
    - [x] the remaining direct `stats/stale.rs` tail now also uses the shared helper, so the
      `crates/fret-diag/src/stats` tree no longer bypasses canonical-first helper-based dual-write,
    - [x] `evidence_index.rs` now reads `selected_bundle_artifact` / `packed_bundle_artifact`
      first while retaining legacy `selected_bundle_json` / `packed_bundle_json` fallback,
    - [x] a follow-up scan now shows no other obvious small reader-side canonical-first patch left
      inside `crates/fret-diag` outside deferred Layer A surfaces or intentional dual-write
      producers,
    - [x] the remaining Layer B follow-up outside the `stats` tree is now reduced to one explicit
      decision before any Layer A manifest chunk-index contract change is considered,
    - [x] the non-`stats/*` Layer B audit is now captured in
      `LAYER_B_PAYLOAD_FAMILIES_AUDIT_V1.md`, which classifies `diag_repro`, `diag_repeat`, and
      `evidence_index` as aligned, initially scoped `lint.rs` as the last small P1 producer, and
      explicitly defers Layer A chunk-index surfaces,
    - [x] `crates/fret-diag/src/lint.rs` now adopts the shared canonical-first helper policy, so
      the non-`stats/*` Layer B follow-up is closed unless a new payload family appears,
    - [x] Layer A run-manifest `bundle_json` is now explicitly documented as a format-specific raw
      bundle chunk index in `RUN_MANIFEST_BUNDLE_JSON_CHUNK_INDEX_CONTRACT_V1.md`, so Layer B
      canonical `bundle_artifact` cleanup no longer implies a manifest rename,
    - [x] DevTools/MCP residual `*json` names are now explicitly classified in the audit as mostly
      raw JSON text holders rather than artifact-path contract drift, so they are deferred by
      default unless those modules are already being changed for another reason.
- [x] Decide whether to introduce a first-class “campaign” orchestration layer.
  - [x] Land a minimal aggregation/index consumer first via `fretboard diag summarize`.
  - [x] Land a first `fretboard diag campaign` surface that composes existing `suite` + `summarize` flows.
  - [x] Decide when campaign definitions should move from built-in Rust registry to external manifests.
    - evidence: `docs/workstreams/diag-fearless-refactor-v2/CAMPAIGN_DEFINITION_EXTERNALIZATION_DECISION_V1.md`
    - implementation: repo-owned JSON manifests under `tools/diag-campaigns/` are now treated as
      the primary authoring surface, while `CampaignRegistry::load_from_workspace_root` keeps
      built-in Rust definitions as fallback/bootstrap entries with same-id manifest override
- [ ] Define expected outputs for orchestrated runs:
  - [x] one machine-readable summary,
    - evidence: `docs/workstreams/diag-fearless-refactor-v2/REGRESSION_SUMMARY_SCHEMA_V1.md`
    - implementation: `diag suite`, `diag repeat`, `diag perf`, and `diag matrix`
      now emit `regression.summary.json`
  - [x] stable reason-code contract documented at the repo level,
    - evidence: `docs/workstreams/diag-fearless-refactor-v2/REASON_CODE_CONSUMER_ALIGNMENT_AUDIT_V1.md`
    - implementation: `RegressionSummaryV1`, `diag_summarize`, `diag_dashboard`, DevTools GUI, and
      MCP all consume normalized `reason_code` / `top_reason_codes` directly; residual
      `error_reason_code` / `reason_code_counts` fields are now explicitly classified as
      producer-local report naming rather than shared consumer drift
  - [x] evidence bundle/artifact paths,
    - evidence: `docs/workstreams/diag-fearless-refactor-v2/ORCHESTRATED_OUTPUT_EVIDENCE_PATH_CONTRACT_V1.md`
    - implementation: orchestrated outputs keep canonical item evidence fields in
      `RegressionEvidenceV1`, keep aggregate `summary_path` / `index_path` /
      `share_manifest_path` / `capabilities_check_path` distinct from item evidence, and retain
      additive alias compatibility for legacy reader spellings
  - [x] optional compact pack for sharing,
    - evidence: `docs/workstreams/diag-fearless-refactor-v2/OPTIONAL_COMPACT_PACK_FOR_SHARING_V1.md`
    - implementation: campaign/batch share flows emit `share/share.manifest.json` plus optional
      `share/combined-failures.zip`, per-item `share_artifact` remains the bounded compact-pack
      pointer, and repro flows keep `repro.ai.zip` / `repro.zip` as the same optional pack family

## M4 — DevTools GUI alignment

- [ ] Keep DevTools GUI follow-up behind artifact/model stabilization:
  - [ ] avoid locking UI polish to unstable artifact/materialization shapes,
  - [ ] prefer shared presentation helpers before new GUI-only projections,
  - [ ] resume GUI alignment after the aggregate artifact model and output surfaces settle.
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
  - [x] selected non-passing summaries can now also surface and copy `capabilities_check_path`
    evidence when the row is `skipped_policy`.
  - [x] selected non-passing summaries now also surface and copy capability provenance through a
    dedicated `Capability Sources` evidence lane.
  - [x] selected summary evidence can now be packed directly from the first failing bundle dir.
- [x] Add a thin GUI summarize trigger over the shared aggregate artifacts:
  - [x] the `Regression` tab now includes a `Summarize` action next to `Refresh`,
  - [x] the action runs the existing `diag summarize` flow against the current artifacts root,
  - [x] successful completion refreshes the aggregate artifacts instead of creating a GUI-only summary model.
- [x] Expose aggregate summary/index artifacts through the MCP consumer lane:
  - [x] `apps/fret-devtools-mcp` now exposes `regression.summary.json`,
  - [x] `apps/fret-devtools-mcp` now exposes `regression.index.json`,
  - [x] resources reuse the existing artifacts-root contract instead of defining a new store,
  - [x] the shared dashboard projection/human-summary path now also keeps `skipped_policy`
    counters and `non-passing summaries` wording aligned with CLI/GUI.
  - [x] the MCP dashboard output now also surfaces capability provenance and
    `capabilities_check_path` when the sibling summary artifact is present.
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

- [x] Add a concise navigation note that tells contributors where to start for diag work.
  - [x] documented in `docs/workstreams/diag-fearless-refactor-v2/START_HERE.md`.
- [x] Cross-link existing v1/v1-architecture docs to this v2 umbrella where appropriate.
- [x] Record migration intent for large existing diag docs rather than duplicating content forever.
  - [x] documented in `docs/workstreams/diag-fearless-refactor-v2/DOCUMENT_MIGRATION_INTENT.md`.
- [x] Document the first aggregate dashboard/index fields for consumers:
  - [x] counters by lane/status/tool/reason,
  - [x] top reason codes,
  - [x] failing summaries ranking.
- [x] Land one thin consumer over the aggregate index:
  - [x] `fretboard diag dashboard` reads `regression.index.json`,
  - [x] default output gives a first-open human summary,
  - [x] `--json` preserves machine-readable access to the full index.
- [x] Add a short maintainer checklist for new diagnostics features:
  - [x] which layer changes,
  - [x] what gate must be added,
  - [x] what evidence should be left behind,
  - [x] what docs must be updated.
  - [x] documented in `docs/workstreams/diag-fearless-refactor-v2/MAINTAINER_CHECKLIST.md`.

## M6 — Debt removal and enforcement

- [ ] Identify duplicated logic that should be removed only after seam adoption is proven.
- [ ] Add at least one regression gate or lint/test expectation for each major seam migration.
  - [x] initial seam-to-gate mapping documented in
    `docs/workstreams/diag-fearless-refactor-v2/SEAM_GATE_MATRIX.md`.
  - [x] first concrete protecting anchors now named for `diag suite`, `diag_campaign`, and
    doctor/lint compatibility seams.
  - [x] legacy manifest compatibility seam now has a named reader-compat test.
  - [x] legacy artifact alias seam now has named `artifact_lint` reader/compat anchors.
- [x] Define “done” criteria for retiring older diag notes or compatibility shims.
  - [x] documented in `docs/workstreams/diag-fearless-refactor-v2/RETIREMENT_CRITERIA.md`.
- [x] Keep a visible debt list so future refactors stay incremental instead of reverting to ad-hoc growth.
  - [x] documented in `docs/workstreams/diag-fearless-refactor-v2/DEBT_RETIREMENT_TRACKER.md`.
