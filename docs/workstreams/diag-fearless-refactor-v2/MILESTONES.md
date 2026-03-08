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
- The next decision point in this area is no longer broad report/outcome shaping; the higher-ROI
  follow-up is the remaining final materialization/update holdouts around share payload mutation and
  artifact handoff before shifting to artifact materialization or
  presentation-surface follow-up work.

Exit criteria:

- Follow-up implementation PRs can be scoped as small seam migrations, not broad rewrites.

## M2 — Artifact model consolidated

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
- The remaining gap in this milestone is campaign aggregation and tighter reason/evidence
  vocabulary standardization.

Outcome:

- Regression execution becomes a product surface instead of a loose collection of commands.

Deliverables:

- A documented lane model (`smoke`, `correctness`, `matrix`, `perf`, `nightly/full`).
- A metadata plan for suites/scripts.
- A summary/evidence contract for orchestrated runs.
- Initial design note: `REGRESSION_CAMPAIGN_V1.md`.
- Initial summary schema note: `REGRESSION_SUMMARY_SCHEMA_V1.md`.

Exit criteria:

- It is obvious how to scale from one script to one suite to one repo-level regression run.

## M4 — DevTools GUI aligned to the same contracts

Outcome:

- GUI participates in the same diagnostics architecture instead of drifting into a parallel model.

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

- Cross-links from older workstreams where needed.
- A maintainer checklist for landing diagnostics changes safely.

Exit criteria:

- A maintainer can add a diagnostics feature and know which docs and gates to touch.

## M6 — Enforcement and debt retirement

Outcome:

- Refactor gains are preserved instead of slowly regressing back into duplication.

Deliverables:

- A visible debt retirement list.
- Seam migrations paired with gates or tests.
- Exit criteria for removing stale compatibility/documentation paths.

Exit criteria:

- The diagnostics stack can continue evolving through additive seam-based changes rather than future monolithic rewrites.
