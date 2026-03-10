# Diag Fearless Refactor v2 — Milestones

Status: Draft

Tracking doc: `docs/workstreams/diag-fearless-refactor-v2/README.md`

## M0 — Shared scope and boundary map

Outcome:

- The repo has one clear umbrella statement for what diagnostics includes and what it does not.

Deliverables:

- A v2 umbrella workstream note.
- A boundary map for runtime, tooling, transport, artifacts, and GUI (`CRATE_AND_MODULE_MAP.md`).
- A documented decision that DevTools GUI is included as a consumer lane.

Exit criteria:

- A contributor can answer “should this live in runtime, tooling, or GUI?” without guesswork.

## M1 — Core seam plan agreed

Outcome:

- We have a practical refactor sequence instead of a loose list of local improvements.

Deliverables:

- A hotspot inventory for runtime/tooling.
- A short list of next seam extractions with rationale and risk.
- A phased implementation roadmap (`IMPLEMENTATION_ROADMAP.md`).

Progress update:

- `diag_suite` has now landed seventeen consecutive seam slices around post-run and result-summary orchestration:
  - core default check planning,
  - editor/markdown/text default check planning,
  - explicit-or-policy post-run trigger planning,
  - retained-vlist script override planning,
  - suite success/failure summary emit factoring,
  - per-script row payload shaping,
  - failure finalization around stop/emit/return-exit paths,
  - tooling-failure handling around script-result writes plus row/finalize wiring,
  - script-outcome handling for failed/unexpected/lint-failed branches,
  - per-script context assembly for stage/reason accounting plus evidence/lint preparation,
  - transport result decoding around `dump_label`, `run_script_over_transport`, and `tooling.suite.error` fallback,
  - script-execution block assembly for prewarm/prelude/load-script wiring,
  - per-script launch env/default assembly plus connected transport acquisition,
  - transport-backed execution dispatch around context assembly plus block invocation,
  - per-script lint execution plus passed-script post-run preparation,
  - per-script result finalization around stage branching, success-row emission, and stop-demo teardown,
  - the remaining per-script success tail around lint-failure exit, post-run apply, and success finalize orchestration.
- The next decision point in this area is whether the few remaining session-root-only holdouts are
  still worth slicing, or whether the higher-ROI seam has shifted to `diag_campaign`, `diag_run`,
  or artifact resolution/materialization.
- `diag_run` has now landed its first shared result/post-run seam slice:
  - transport result stage normalization,
  - shared bundle doctor/post-run check execution,
  - shared AI packet / pack artifact emission,
  - demo-exit-killed marking before final result exit.
- `diag_run` has now landed its second seam slice around filesystem post-run orchestration:
  - failed-run dump bundle backfill,
  - bundle artifact resolution/wait with optional trigger retry,
  - filesystem post-run finalization for bundle doctor reuse and artifact emission.
- `diag_run` has now landed its third seam slice around command-level branch separation:
  - DevTools WS setup/connect/run wiring,
  - branch-local result finalization and artifact emission,
  - command-level separation between the WS and filesystem paths.
- `diag_run` has now landed its fourth seam slice around filesystem branch separation:
  - launch env/default preparation,
  - launch/connect plus stop-guard ownership,
  - filesystem branch-local execution and finalization dispatch.
- `diag_run` has now landed its fifth seam slice around top-level command setup:
  - argument normalization and script-source resolution,
  - check-derived defaults and pack/bundle intent derivation,
  - transport-mode validation before exclusive branch dispatch.
- `diag_run` is now effectively parked: the remaining body is mostly resolved-path setup plus
  exclusive branch dispatch, so the next higher-ROI seam has shifted back to `diag_campaign`
  summary/finalize execution and report shaping rather than a speculative sixth `cmd_run` slice.
- `diag_campaign` has now landed another artifact/evidence seam around share-manifest item processing:
  - per-item bundle/triage/screenshot/share-zip planning now routes through a dedicated helper,
  - share-manifest counters now use a named mergeable struct instead of ad hoc local integers,
  - combined failure zip staging now uses a named entry shape instead of an anonymous tuple,
  - final manifest payload assembly and combined-zip update now route through dedicated helpers plus a named outcome shape.
- `commands/resolve` has now landed another artifact-resolution seam around `diag resolve latest`, session selection, script-result search-start normalization, and shared bundle resolution:
  - option parsing now routes through a dedicated pure helper,
  - JSON and text projections now route through dedicated render helpers,
  - target session-id selection plus existing-session directory validation now route through dedicated helpers,
  - `resolve_script_result_json_path_or_latest` now routes latest-vs-src search-start selection through dedicated helpers,
  - `resolve_bundle_input_or_latest` / `resolve_bundle_ref` now route source-path selection, bundle-ref derivation, and artifacts-root policy through dedicated helpers,
  - helper-level regression coverage now locks the command output shape plus session-selection/search-start/bundle-root policy behavior without relying on stdout capture.
- `commands::artifacts` plus `commands/artifact.rs` have now landed another artifact-resolution seam cluster around `cmd_pack`, `cmd_meta`, `cmd_lint`, `cmd_triage`, and `cmd_artifact_lint`:
  - bundle/source resolution and user-facing error hinting now route through dedicated setup helpers,
  - default output path selection now routes through pure helpers,
  - repeated single-bundle emitters now also reuse shared bundle-input plus path/json output helpers,
  - `cmd_meta` now routes its human-readable projection through pure report-line helpers,
  - `cmd_lint` now routes bundle/out-path preparation through `prepare_cmd_lint` and exit-policy dispatch through a dedicated helper while reusing the shared JSON writer,
  - `cmd_triage` now routes payload assembly through dedicated lite/full builders plus a tooling-warning attachment helper,
  - `cmd_artifact_lint` now routes artifact-dir/out-path preparation plus exit-policy dispatch through dedicated helpers while reusing a dedicated write helper,
  - helper-level regression coverage now locks the AI-only output-path policy, shared input validation paths, meta-report line shape, lint exit predicate, triage lite out-path policy, tooling-warning attachment behavior, artifact-lint out-path policy, and artifact-dir normalization for `_root` manifests.
- `diag_campaign` has now landed another orchestration seam around run-outcome assembly:
  - `execute_campaign_run_selection` now routes counters plus command-failure aggregation through a dedicated `build_campaign_run_outcome` helper,
  - helper-level regression coverage now locks the combined counters/failures outcome shape without relying on command execution.
- `diag_campaign` has now landed another orchestration seam around summary-finalize materialization:
  - single-run and batch finalize paths now share `execute_campaign_summary_finalize_outcome` for summarize/share execution plus error capture,
  - timing/materialization now route through `build_campaign_summary_artifacts`,
  - helper-level regression coverage now locks saturating duration handling plus outcome preservation without running summarize/share side effects.
- `diag_campaign` has now landed another artifact handoff seam around batch writes:
  - `write_campaign_batch_artifacts` now builds a dedicated `CampaignBatchArtifactWritePlan`,
  - batch manifest writing now routes through `build_campaign_batch_manifest_write_plan`,
  - helper-level regression coverage now locks manifest output-path/payload shaping plus summary-finalize setup reuse without running campaign execution.
- `diag_campaign` has now landed another startup seam around single-campaign execution:
  - `execute_campaign` now builds a dedicated `CampaignExecutionStartPlan`,
  - single-campaign manifest writing now routes through `build_campaign_manifest_write_plan`,
  - helper-level regression coverage now locks execution-plan/manifest setup reuse without running suite/script execution.
- `diag_campaign` has now landed another finalize seam around single-campaign execution:
  - `finalize_campaign_execution` now builds a dedicated `CampaignExecutionFinalizePlan`,
  - failure counting plus summary-finalize setup now route through that plan before finalize IO,
  - helper-level regression coverage now locks failure-count and summary-finalize setup reuse without running summarize/write side effects.
- `diag_campaign` has now landed another report handoff seam around single-campaign execution:
  - `execute_campaign` now routes normalization plus report construction through `build_campaign_execution_report_from_outcome_result`,
  - helper-level regression coverage now locks the error-to-failed-report normalization path without running campaign execution.
- `diag_campaign` has now landed another combined-failure export seam around zip entry planning:
  - `write_campaign_combined_failure_zip_inner` now consumes dedicated root/item zip-entry planners,
  - helper-level regression coverage now locks root index inclusion and per-item artifact ordering without writing a zip file.
- `diag_campaign` has now landed another share-manifest seam around item aggregation and artifact shaping:
  - `write_campaign_share_manifest` now consumes a dedicated `CampaignShareManifestItems` aggregate instead of owning the per-item loop inline,
  - `build_campaign_share_manifest_item` now consumes a dedicated `CampaignShareManifestItemArtifacts` snapshot so artifact IO and run-entry shaping stop living in one block,
  - helper-level regression coverage now locks include-passed filtering, missing-bundle artifact handling, and pure run-entry shaping without executing a full campaign share flow.
- `diag_campaign` has now landed another share-manifest seam around payload planning and finalize handoff:
  - `write_campaign_share_manifest` now consumes a dedicated `CampaignShareManifestWritePlan` for initial payload/output-path planning,
  - combined-failure zip update now routes through `finalize_campaign_share_manifest_write`,
  - helper-level regression coverage now locks write-plan payload/output shaping and finalize-time combined-zip path recording without running a full campaign execution.
- `diag_campaign` has now landed another share-manifest seam around per-item artifact planning:
  - bundle-dir resolution now routes through a dedicated helper,
  - triage/screenshot collection now routes through a dedicated supporting-artifacts helper,
  - AI-packet/share-zip planning now routes through a dedicated share-zip helper,
  - helper-level regression coverage now locks evidence-path resolution, reuse of existing triage/screenshots artifacts, and missing-bundle share-zip error handling directly.
- `diag_campaign` has now landed another share-manifest seam around payload section shaping:
  - `build_campaign_share_manifest_payload` now consumes dedicated source/selection/counters/share sections,
  - helper-level regression coverage now locks section-level field shaping without going through the full write-plan path.
- `diag_campaign` has now landed another share-manifest seam around combined-zip field mutation:
  - `apply_campaign_share_manifest_combined_zip` now routes through dedicated field-building and share-section apply helpers,
  - helper-level regression coverage now locks combined-zip field projection plus section-local mutation without going through the full finalize path.
- `commands::artifacts` has now landed another artifact-resolution/materialization seam around `cmd_meta`:
  - `resolve_meta_artifact_paths` now routes direct sidecar, bundle-dir, and bundle-path resolution through dedicated helpers,
  - helper-level regression coverage now locks valid-sidecar reuse, invalid-sidecar fallback, and `_root` sidecar preference without invoking the full command.
- `commands::artifacts` has now landed another materialization seam across repeated artifact emitters:
  - canonical artifact output materialization now routes through a shared helper reused by `cmd_meta` and `cmd_test_ids`,
  - helper-level regression coverage now locks same-path no-op, existing-out reuse, and nested custom-out copy behavior without invoking the full command.
- `commands::artifacts` has now landed another emitter seam across generated index commands:
  - ensured artifact output emission now routes through a shared helper reused by `cmd_test_ids_index` and `cmd_frames_index`,
  - helper-level regression coverage now locks success, ensure-error, and emit-error behavior without invoking the full command.
- `commands::artifacts` has now landed another preparation seam around `cmd_triage`:
  - triage bundle resolution and default/custom out-path selection now route through a dedicated prepare helper,
  - helper-level regression coverage now locks lite default-out and custom-out behavior without invoking the full command.
- `commands::artifacts` has now landed another output-surface seam across artifact emitters:
  - artifact text reads, JSON reads, and pretty JSON text shaping now route through dedicated helpers reused by `emit_path_or_json_output`, `emit_artifact_output`, and `write_json_artifact_output`,
  - helper-level regression coverage now locks text-read, JSON-read, and pretty-format behavior without invoking the full command.
- `commands::artifacts` has now landed another preparation seam around `cmd_test_ids`:
  - test-ids bundle/out preparation now routes through a dedicated helper while cached-output short-circuit routes through a dedicated existing-file helper,
  - helper-level regression coverage now locks default/custom out-path selection and existing-file shortcut behavior without invoking the full command.
- `commands::artifacts` has now landed another preparation seam around `cmd_meta`:
  - meta parse/resolve/out preparation now routes through a dedicated helper,
  - helper-level regression coverage now locks default/custom out-path selection plus display-mode preparation without invoking the full command.
- `commands::artifacts` has now landed another lint-output seam around `cmd_lint`:
  - lint report write and exit-required decision now route through a dedicated helper,
  - helper-level regression coverage now locks report-output and exit-required behavior without invoking the full command.
- `commands::artifacts` has now landed another pack preflight seam around `cmd_pack`:
  - ai.packet path resolution, best-effort ensure, and `--ai-only` directory validation now route through dedicated helpers,
  - helper-level regression coverage now locks ai.packet path shaping and `--ai-only` success/error behavior without invoking the full command.
- `commands::artifacts` has now landed another materialization/output seam across generated artifact commands:
  - `cmd_test_ids` and `cmd_meta` now share a dedicated materialize-plus-emit helper,
  - path-vs-JSON output mode selection now also routes through a dedicated helper reused by the simple emit path,
  - helper-level regression coverage now locks existing-output reuse and missing-out copy behavior without invoking the full command.
- `commands::artifacts` has now landed another pack execution seam around `cmd_pack`:
  - ai-only vs full-bundle zip dispatch now routes through a dedicated execution plan plus runner,
  - full-bundle default sort normalization now no longer lives inline in the command entry,
  - helper-level regression coverage now locks ai-only mode and bundle default-sort planning without invoking the full command.
- `commands::artifacts` has now landed another presentation seam around `meta_report_lines`:
  - summary lines now route through a dedicated presentation helper,
  - per-window formatting, missing considered-frame fallback, and truncation now route through dedicated helpers,
  - helper-level regression coverage now locks semantics summary plus window-row fallback/truncation behavior without invoking the full command.
- `commands::artifacts` has now landed another triage payload seam:
  - lite-vs-full payload mode selection now routes through a dedicated helper,
  - full-sort defaulting and warning-finalize handoff now no longer live inside one payload builder,
  - helper-level regression coverage now locks lite metric selection, full default sort, and finalize no-op behavior without invoking the full command.
- `commands::artifacts` has now landed another presentation-surface reuse seam:
  - path, JSON text, and meta-report lines now route through a shared `ArtifactOutputPresentation` surface,
  - file-write output projection and emitted-artifact display no longer each own their own terminal rendering path,
  - helper-level regression coverage now locks JSON text/path projection and meta-report line projection without invoking the full command.
- `commands::artifacts` has now landed another report-output seam around `cmd_lint`:
  - lint report file-write plus output projection now route through a dedicated `LintReportOutput` seam,
  - the lint helper now reuses the shared JSON-write presentation surface instead of owning a parallel output path,
  - helper-level regression coverage now locks path presentation plus exit-flag shaping without invoking the full command.
- `diag_summarize` has now landed another aggregate presentation seam:
  - JSON-vs-human summarize output now routes through a dedicated `SummarizeOutputPresentation` seam,
  - failed-count accounting and success-line shaping now no longer live inline in `cmd_summarize`,
  - helper-level regression coverage now locks aggregate failed-count accounting and output text shaping without invoking the full command.
- `diag_dashboard` has now landed another aggregate presentation seam:
  - JSON-vs-human dashboard output now routes through a dedicated `DashboardOutputPresentation` seam,
  - counter sections, reason-code rows, and failing-summary rows now build through dedicated line helpers instead of inline `println!` blocks,
  - helper-level regression coverage now locks aggregate summary lines, counter sections, and failing-summary row shaping without invoking the full command.
- `diag_campaign` has now landed another aggregate presentation seam:
  - JSON-vs-human campaign-run output now routes through a dedicated `CampaignRunOutputPresentation` seam,
  - `print_campaign_run_output` now only orchestrates build-plus-emit instead of owning output selection inline,
  - helper-level regression coverage now locks JSON text projection and single-run human line shaping without invoking the full command.
- `apps/fret-devtools-mcp` has now landed the first non-CLI reuse of the aggregate dashboard presentation vocabulary:
  - dashboard counter, reason-code, and failing-summary projection now reuse exported helpers from `fret-diag`,
  - MCP `human_summary` now reuses the same dashboard line builder as the CLI instead of maintaining a parallel text-assembly path,
  - targeted regression coverage now locks the shared projection on both the `fret-diag` side and the MCP consumer side.
- `apps/fret-devtools` has now landed the first GUI reuse of the aggregate dashboard presentation vocabulary:
  - the `Regression` tab now reuses exported dashboard human-summary helpers from `fret-diag`,
  - failing-summary table rows now also reuse exported failing-summary projection helpers instead of reparsing the index payload locally,
  - targeted regression coverage now locks the shared projection on the GUI consumer side as well.
- `commands::artifacts` has now landed another output seam around `cmd_triage`:
  - triage payload build plus output-file write now route through a dedicated `TriageCommandOutput` seam,
  - `cmd_triage` now only orchestrates prepare plus build-plus-emit instead of owning payload build and terminal output inline,
  - targeted regression coverage now locks both path and JSON text presentation for the triage command seam.
- `commands::artifacts` has now landed another ensured-artifact output seam:
  - ensured-artifact commands now route `ensure + display-mode projection` through a dedicated `EnsuredBundleArtifactOutput` seam,
  - the shared ensure helper now only orchestrates build-plus-emit instead of owning both ensure and terminal output shaping inline,
  - targeted regression coverage now locks both path and JSON text projection for the ensured-artifact seam.
- `commands::artifacts` has now landed another required-bundle command seam:
  - `cmd_test_ids_index` and `cmd_frames_index` now route `resolve input + ensure output + presentation build` through `build_required_bundle_artifact_output`,
  - command entrypoints now only orchestrate guard plus build-plus-emit instead of re-splicing required-input resolution inline,
  - targeted regression coverage now locks missing-hint propagation and resolved bundle-path handoff for this seam.
- `commands::artifacts` has now landed another generated-artifact materialization seam:
  - `cmd_test_ids` and `cmd_meta` now route materialization through a dedicated `GeneratedArtifactOutput` seam,
  - command entrypoints now only orchestrate prepare plus build-plus-emit instead of delegating to a monolithic materialize-and-emit helper,
  - targeted regression coverage now locks reuse-existing and copy-missing materialization behavior at the builder seam.
- `commands::resolve` has now landed another normalization seam around `diag resolve latest` output assembly:
  - latest run projection now routes through a dedicated helper,
  - latest bundle projection now routes through a dedicated helper reused by `resolve_latest_for_out_dir` and `resolve_latest_bundle_dir_from_base_or_session_out_dir`,
  - `resolve_latest_bundle_dir_for_out_dir` now routes script-result hint parsing and latest-marker/scan fallback through dedicated helpers,
  - `resolve_session_out_dir_for_base_dir` now routes session-root vs base-dir selection through a dedicated mode helper plus direct-resolution builder,
  - helper-level regression coverage now locks existing-run-dir projection, missing-artifact filtering, base/session latest-bundle reuse, relative/absolute hint fallback behavior, and session-root marker preference without invoking the full command.
- The next decision point in this area is no longer whether `cmd_triage`, the ensured-artifact
  helper, or required-bundle ensured commands still need basic orchestration seams; those slices are
  now landed. The same is now true for the shared generated-artifact materialization tail. The higher-ROI follow-up remains the deeper
  artifact resolution/materialization holdouts in `commands::artifacts`, especially where command
  entrypoints still mix input resolution with ensured-artifact production or write-time update policy.
- Practically, this means the default next move should no longer be “keep slicing `commands::artifacts`
  indefinitely”; it should usually be either a clearly reviewable remaining tail in that file or a
  shift back to repo-level contract/vocabulary tightening.

Near-term execution order:

- First, finish cross-linking the canonical artifact/evidence and M3 vocabulary contract updates across the remaining diagnostics notes.
- Second, inspect residual naming in persisted artifacts and consumers before reopening another seam-slicing pass.
- Third, keep `commands::artifacts` and `commands::resolve` parked unless a clearly reviewable holdout becomes higher ROI than residual-vocabulary adoption.

Exit criteria:

- Follow-up implementation PRs can be scoped as small seam migrations, not broad rewrites.

## M2 — Artifact model consolidated

Progress update:

- the first repo-level artifact/evidence contract is now drafted in
  `docs/workstreams/diag-fearless-refactor-v2/ARTIFACT_AND_EVIDENCE_MODEL_V1.md`,
- the next value in this area is adoption across maintainer-facing docs and flows rather than a new
  parallel artifact note family.

## M3 — Orchestration vocabulary stabilized

Outcome:

- The repo uses one shared regression orchestration vocabulary across docs, artifacts, CLI, GUI,
  MCP, and CI-facing flows.

Deliverables:

- A repo-level vocabulary contract for lanes, statuses, reason codes, flake policy, capability
  tags, and artifact/evidence path names
  (`M3_ORCHESTRATION_VOCABULARY_AND_CONTRACT_V1.md`).
- Campaign, summary, and execution notes cross-linked to that vocabulary.
- A clear split between v1 must-have terminology and later additive expansion.

Progress update:

- the first repo-level vocabulary contract is now drafted in
  `docs/workstreams/diag-fearless-refactor-v2/M3_ORCHESTRATION_VOCABULARY_AND_CONTRACT_V1.md`,
- the lane vocabulary now explicitly normalizes `full` as a user-facing broad selector while
  persisted artifacts should prefer `nightly`,
- stable status vocabulary is now explicitly named in one note instead of being implied across
  multiple documents,
- reason-code naming rules, flake-policy wording, capability-tag naming rules, and canonical
  campaign/batch/share path names now live in one place,
- the contract now also includes a first persisted-field normalization map plus explicit
  writer/reader alias-lifecycle rules,
- the contract now also names a recommended adoption order, so the next work can audit persisted
  fields and shared consumers against one explicit checklist,
- the first bounded adoption audit is now written down in
  `docs/workstreams/diag-fearless-refactor-v2/M3_VOCABULARY_ADOPTION_AUDIT.md`,
- implementation adoption is now partially landed in shared summary serde, summarize/dashboard
  wording, campaign share payload fields, run-manifest `paths`, artifact-lint compatibility, and
  the first additive campaign-metadata pass (`requires_capabilities` / `flake_policy`), plus
  explicit canonical-vs-projection classification for `index_json`, `perf_summary_json`, and
  `compare_json`,
- a bounded campaign metadata execution-adoption audit now also exists, explicitly deferring
  campaign-level capability gating and flake-policy behavior until a concrete orchestration
  consumer appears,
- a first-pass campaign capability preflight design note now also exists, narrowing the first
  behavior slice to campaign-local capability checks plus normalized `skipped_policy` /
  `capability.missing` reporting,
- that first capability-preflight behavior slice is now partially landed:
  - campaign execution now performs capability preflight before item execution,
  - campaign-local `check.capabilities.json` is now written on mismatch,
  - campaign summaries now emit a synthetic `CampaignStep` row for the decision,
  - batch and single-run output now distinguish `skipped_policy` from ordinary failure,
  - campaign counters now expose `campaigns_skipped_policy`,
  - capability-source resolution is now shared with the diagnostics filesystem capability loader
    and surfaced consistently in `diag doctor`,
  - DevTools `Regression` drill-down now also surfaces campaign-local capability-check artifacts
    for selected `skipped_policy` summaries,
  - MCP dashboard coverage now locks shared `skipped_policy` counters and `non-passing summaries`
    wording,
- campaign authoring now also has a first dedicated validation entrypoint:
  - `diag campaign validate` reuses the manifest loader contract already used by registry loading,
  - the command can validate either the repo-owned `tools/diag-campaigns/*.json` set or explicit
    ad hoc manifest paths,
  - subcommand dispatch now avoids preloading the full workspace registry for validation-only runs,
  - maintainer docs now include validate examples instead of leaving manifest checks implicit,
- campaign authoring preflight now also has a first doctor surface:
  - `diag doctor campaigns` checks the repo-owned manifest set as a read-only maintainer preflight,
  - the first report covers invalid manifests, duplicate ids, and residual legacy top-level
    `suites` / `scripts` authoring shape,
  - `--strict` can already treat legacy shape usage as non-OK without changing the loader contract,
- a bounded DevTools/MCP raw-JSON defer audit now also exists, explicitly separating app-local text
  holders from shared persisted artifact vocabulary,
- the next value in this area is finishing residual naming adoption rather than another terminology
  draft.

Exit criteria:

- contributors can tell which lane/status/reason/path term is canonical without comparing multiple
  docs,
- new CLI / GUI / MCP / CI-facing work reuses the shared vocabulary instead of adding synonyms,
- future reason-code or capability-tag growth can happen additively under a stable naming system.

Outcome:

- Diagnostics outputs are described as one coherent artifact system.

Progress update:

- The first consolidated artifact and evidence contract now exists in
  `docs/workstreams/diag-fearless-refactor-v2/ARTIFACT_AND_EVIDENCE_MODEL_V1.md`.
- The workstream now names source-of-truth artifacts, derived/index artifacts, optional evidence,
  and presentation-facing projections explicitly.
- The remaining gap in this milestone is adoption and enforcement across older notes and any future
  artifact additions.

Deliverables:

- A canonical artifact taxonomy.
- Compatibility and bounded-evidence policy.
- A defined “first-open” artifact set for everyday triage.
- Primary note: `docs/workstreams/diag-fearless-refactor-v2/ARTIFACT_AND_EVIDENCE_MODEL_V1.md`.

Exit criteria:

- Humans and tools can perform common triage without assuming the raw largest artifact is always required.

## M3 — Regression orchestration model chosen

Progress update:

- The initial summary/evidence contract is no longer doc-only.
- `diag suite`, `diag repeat`, `diag perf`, and `diag matrix` now emit
  `regression.summary.json` as an additive artifact.
- `diag matrix` also leaves behind `matrix.summary.json` for compare-oriented consumers.
- campaign aggregation is now landed through `diag summarize` and `diag campaign`,
- the remaining gap in this milestone is tighter reason/evidence vocabulary adoption across the
  remaining persisted fields and consumers.

Near-term execution order:

- finish adoption of the orchestration vocabulary around stable reason codes, artifact/evidence path
  naming, flake policy, capability tags, and lane naming before more tools and GUI flows depend on
  drifted terms,
- then inspect residual manifest names such as `files[].id` before deciding whether another
  additive compatibility bridge is required.

Outcome:

- Regression execution becomes a product surface instead of a loose collection of commands.

Deliverables:

- A documented lane model (`smoke`, `correctness`, `matrix`, `perf`, canonical `nightly`, and the
  legacy user-facing alias `full`).
- A metadata plan for suites/scripts.
- A summary/evidence contract for orchestrated runs.
- Initial design note: `REGRESSION_CAMPAIGN_V1.md`.
- Initial summary schema note: `REGRESSION_SUMMARY_SCHEMA_V1.md`.

Exit criteria:

- It is obvious how to scale from one script to one suite to one repo-level regression run.

## M4 — DevTools GUI aligned to the same contracts

Outcome:

- GUI participates in the same diagnostics architecture instead of drifting into a parallel model.

Near-term execution order:

- Defer another round of DevTools GUI polish until the artifact model and presentation seams are
  stable enough that the UI is consuming the right backend shape.

Deliverables:

- A documented GUI-in-scope boundary.
- At least one end-to-end dogfood workflow that crosses pick/run/artifacts.
- A clear defer list for GUI-only polish.

Exit criteria:

- GUI can be discussed as a presentation surface over shared diagnostics contracts.

## M5 — Documentation migration and maintainer workflow

Outcome:

- Diagnostics docs become easier to navigate and less likely to diverge.

Deliverables:

- A concise navigation note for maintainers/contributors.
- Cross-links from older workstreams where needed.
- A maintainer checklist for landing diagnostics changes safely.
- One thin migration-intent note that says which docs stay active, which stay as linked background,
  and which should stop acting as parallel planning surfaces.

Exit criteria:

- A maintainer can add a diagnostics feature and know which docs and gates to touch.

## M6 — Enforcement and debt retirement

Outcome:

- Refactor gains are preserved instead of slowly regressing back into duplication.

Deliverables:

- A visible debt retirement list.
- Seam migrations paired with gates or tests.
- Exit criteria for removing stale compatibility/documentation paths.
- A tracker that records debt item, current protection, retirement trigger, and retirement action.
- An initial seam-to-gate matrix that names current coverage and explicit gaps.
- One thin retirement-criteria note covering compatibility shims, old notes, and duplicated seam paths.

Exit criteria:

- The diagnostics stack can continue evolving through additive seam-based changes rather than future monolithic rewrites.
