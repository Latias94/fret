# Diag Fearless Refactor v2 ?Implementation Roadmap

Status: Draft

Tracking docs:

- `docs/workstreams/diag-fearless-refactor-v2/README.md`
- `docs/workstreams/diag-fearless-refactor-v2/CRATE_AND_MODULE_MAP.md`
- `docs/workstreams/diag-fearless-refactor-v2/REGRESSION_CAMPAIGN_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/REGRESSION_SUMMARY_SCHEMA_V1.md`

## 0) Why this note exists

The v2 docs now answer three design questions:

- what the diagnostics platform should be,
- where changes belong by crate/layer,
- what a future regression campaign model should look like.

This note answers the next question:

- how do we land the work incrementally without reintroducing monoliths or stalling on a big-bang rewrite?

The roadmap is intentionally implementation-facing. It should be usable to slice follow-up PRs.

## 1) Principles for landing work

### 1.1 Prefer seams before renames

Do not start with broad renaming or package reshuffling.

Prefer:

- explicit module seams,
- small wrappers/adapters,
- moving one production path at a time,
- adding one gate per seam migration.

### 1.2 Keep presentation surfaces thin

The first implementation steps should improve shared core behavior for:

- CLI,
- GUI,
- MCP,
- CI,
- offline artifact consumers.

If a step only helps one surface, it should usually wait unless it removes major architectural risk.

### 1.3 Add machine-readable outputs early

Before building more UI around campaigns, land the summary and evidence model first.

### 1.4 Use documentation to drive module boundaries

If a PR changes ownership lines, it should update the relevant v2 docs in the same change.

## 2) Recommended implementation phases

## Phase A ?Stabilize routing and ownership

Goal:

- make future implementation PRs easy to scope.

Main outputs:

- `CRATE_AND_MODULE_MAP.md`
- `IMPLEMENTATION_ROADMAP.md`

Code expectations:

- no major behavior changes required yet,
- ok to add small comments/tests/helpers only if they clarify seams.

Done when:

- contributors can choose the correct layer before editing code,
- upcoming PRs can point to a phase in this roadmap.

## Phase B ?`crates/fret-diag` orchestration seam cleanup

Goal:

- reduce the cost of adding campaign-style orchestration later.

Primary code areas:

- `crates/fret-diag/src/lib.rs`
- `crates/fret-diag/src/diag_run.rs`
- `crates/fret-diag/src/diag_suite.rs`
- `crates/fret-diag/src/diag_repeat.rs`
- `crates/fret-diag/src/diag_matrix.rs`
- `crates/fret-diag/src/diag_perf.rs`
- `crates/fret-diag/src/registry/`
- `crates/fret-diag/src/commands/`

Suggested seam targets:

- run planning context,
- shared artifact output plumbing,
- check planning vs execution,
- suite selection/resolution,
- summary row emission hooks.

Recommended PR slices:

1. extract a reusable run-summary row writer/helper,
2. centralize per-run evidence path collection,
3. centralize retry/attempt accounting hooks,
4. make suite/campaign resolution an explicit seam instead of buried command logic.

Gate expectations:

- existing `fret-diag` tests stay green,
- add targeted tests for any new summary/evidence helpers.

## Phase C ?Runtime artifact and evidence alignment

Goal:

- make runtime exports easy for campaigns to consume without bespoke scraping.

Primary code areas:

- `ecosystem/fret-bootstrap/src/ui_diagnostics/service.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/bundle*.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/script_result.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/extensions.rs`

Suggested seam targets:

- ensure stable evidence anchors are written predictably,
- keep sidecars and `script.result.json` easy to locate,
- make lane-specific evidence optional and bounded.

Recommended PR slices:

1. normalize runtime-written evidence locations used by tooling,
2. ensure a stable minimum evidence contract for failed runs,
3. add any missing bounded sidecars required by the future summary model.

Gate expectations:

- no regression in existing script runs,
- at least one end-to-end scripted run validates the expected evidence set.

## Phase D ?Land `regression.summary.json` generation

Current status:

- Partially landed.
- `crates/fret-diag/src/regression_summary.rs` now defines the shared summary model.
- `diag suite`, `diag repeat`, `diag perf`, and `diag matrix` now emit
  `regression.summary.json` additively without replacing their existing outputs.
- `diag matrix` also writes `matrix.summary.json` as a stable compare-oriented sidecar.
- `diag summarize` now provides a first consumer-side aggregation/index surface over many
  `regression.summary.json` artifacts.
- the aggregate index already exposes dashboard-oriented helpers such as counters, top reason
  codes, and failing summary rankings.
- Remaining work is now mostly about contract hardening, richer campaign selection, and
  campaign-level orchestration beyond simple aggregation.

Goal:

- produce one machine-readable summary artifact from existing run primitives.

Primary code areas:

- `crates/fret-diag/src/diag_suite.rs`
- `crates/fret-diag/src/diag_repeat.rs`
- `crates/fret-diag/src/diag_matrix.rs`
- `crates/fret-diag/src/diag_perf.rs`
- likely a new summary-focused module under `crates/fret-diag/src/`

Recommended implementation direction:

- start additive,
- generate the summary as an extra artifact for one or two flows first,
- keep existing console output and JSON outputs intact during migration.

Recommended PR slices:

1. [done] add summary model types in `crates/fret-diag`,
2. [done] emit summary rows for `diag suite`,
3. [done] extend summary rows for `diag repeat` and flake classification,
4. [done] extend summary rows for `diag matrix` and `diag perf`,
5. [done] write `regression.summary.json` under a stable location,
6. [done] define a first campaign-level aggregation/index output over many summary artifacts,
7. [next] tighten stable reason-code and evidence-path conventions across all lanes,
8. [next] decide whether aggregation should stay as `diag summarize` or become part of a
   future `diag campaign` surface.

Gate expectations:

- unit tests for summary serialization,
- regression tests for one success case and one failure case,
- no breakage in existing JSON consumers.

## Phase E ?Introduce campaign entry surface

Goal:

- expose the lane model as a first-class repo workflow.

Current status:

- A first minimal `fretboard diag campaign` surface is now landed.
- The new entry currently provides `list`, `show`, and `run` over a small built-in registry.
- Campaign definitions are now routed through `crates/fret-diag/src/registry/campaigns.rs` so the command layer no longer owns built-in campaign data directly.
- The first external resolver path now reads `tools/diag-campaigns/*.json` and lets repo-owned manifests override same-id built-in fallbacks.
- `run` composes existing `diag suite` and `diag summarize` flows instead of introducing a second execution engine.
- Campaign runs now leave a predictable root under `campaigns/<campaign_id>/<run_id>/` with
  `campaign.manifest.json`, `campaign.result.json`, `regression.index.json`, and
  `regression.summary.json`.

Primary code areas:

- `crates/fret-diag/src/commands/`
- `crates/fret-diag/src/registry/`
- `apps/fretboard/src/diag.rs`

Suggested surface:

- a future `diag campaign` or equivalent command,
- lane selection,
- suite filtering,
- output directory and summary emission,
- explicit flake/retry policy knobs.

Recommended PR slices:

1. finish the canonical artifact/evidence contract now that campaign, share, and failure packaging are landed,
2. extract the next high-ROI orchestration seams: `artifact resolution/materialization`, `run planning/context`, and `suite/campaign resolution`,
   - first landing expanded: shared bundle input resolution now lives in `crates/fret-diag/src/commands/resolve.rs` and is now reused by:
     - `diag pack`
     - `diag trace`
     - `diag index`
     - `diag windows`
     - `diag dock-graph`
     - `diag dock-routing`
     - `diag compare` (bundle compare path; resource-footprint path still uses session-root resolution)
     - `diag stats` (single-bundle + diff)
     - `diag slice` (step-index early resolution path)
     - selected `diag query` subcommands that require a bundle artifact
   - second landing: shared `script.result.json` resolution (preferring evidence-bearing parents) now lives in `crates/fret-diag/src/commands/resolve.rs` and is reused by:
     - `diag query overlay-placement-trace`
     - `diag screenshots`
     - `diag doctor` (script result discovery for report generation)
   - third landing: shared run-path context now reuses `ResolvedScriptPaths` across the main orchestration flows instead of re-threading parallel path fields:
     - `diag run`
     - `diag suite`
     - `diag repeat`
     - `diag repro`
     - `diag campaign` (suite/script item dispatch)
     - `diag matrix`
   - fourth landing: `diag run` and `diag repro` now share a higher-level `ResolvedRunContext`, so transport wiring no longer travels as a separate parallel argument beside resolved script paths
   - fifth landing: repeated launch-time filesystem transport config assembly is now centralized for the main script-driven flows, with `ResolvedScriptPaths` exposing a convenience method and the remaining callers reusing the same helper
   - sixth landing: `diag_suite` now reuses a dedicated result-only filesystem transport helper, closing the last duplicated `script.result` override path in the main orchestration commands
   - seventh landing: `diag_suite` now resolves suite script inputs, reuse-process env defaults, and prewarm/prelude path normalization through `ResolvedSuiteRunInputs`, reducing the size of the main orchestration body and preparing the next `diag_campaign` extraction
   - eighth landing: `diag_campaign` now builds per-item `diag_suite::SuiteCmdContext` through a shared invocation builder, removing the duplicated suite/script handoff block and making item expansion/context extraction easier to continue
   - ninth landing: `diag_campaign` now computes per-run and batch output roots through explicit execution plans, shrinking the orchestration body around manifest/summarize/share wiring and giving the next item-expansion slice a stable home
   - tenth landing: `diag_campaign` now builds suite/script item execution through a single `CampaignItemInvocation` builder, so the item dispatcher no longer carries parallel suite/script branches around the same `diag_suite` handoff payload
   - eleventh landing: `diag_campaign` now routes selection execution, aggregate counters, JSON/CLI rendering, and command-failure collection through an explicit `CampaignRunOutcome`, shrinking `cmd_campaign_run` into parse -> execute -> render -> fail orchestration
   - twelfth landing: `diag_campaign` now separates per-campaign item execution, finalize/summarize/share/result writing, and failed-item error formatting, so `execute_campaign_inner` mainly coordinates named stages instead of carrying one long inline finalize block
   - thirteenth landing: `diag_campaign` now shares summarize/share timing through `CampaignSummaryArtifacts`, while batch execution uses a matching finalize stage and batch result counters reuse the same aggregate counters helper as `cmd_campaign_run`
   - fourteenth landing: `diag_campaign` now builds repeated `run`, `selection`, `aggregate`, and item-result JSON payload fragments through shared helpers, so manifest/result writers no longer duplicate the same record-shape assembly inline
   - fifteenth landing: `diag_campaign` now shares `resolved` and `counters` payload assembly through dedicated helpers, which lets manifest/result writers reuse the same aggregate math instead of open-coding per-emitter totals
   - sixteenth landing: `diag_campaign` now routes campaign-report JSON shaping and share/failure text formatting through shared helpers, so run/result emitters and failure aggregation no longer carry duplicate report-field or message-shape assembly
   - seventeenth landing: `diag_campaign` now formats single-run and batch-run CLI output through pure helper functions, shrinking `print_campaign_run_output` and giving the human-readable output shape lightweight regression coverage
   - eighteenth landing: `diag_campaign` now routes run-selection JSON, explicit-vs-filtered selection, and run-flag parsing through dedicated helpers, so command parsing/selection logic no longer mixes with the higher-level orchestration body
   - nineteenth landing: `diag_campaign` now resolves campaign subcommands through a small enum helper and converts `CampaignCmdContext` into `CampaignRunContext` through a dedicated boundary, shrinking the top-level command adapter and isolating command-context ownership flow
   - twentieth landing: `diag_campaign` now normalizes execution errors into a stable `CampaignExecutionOutcome` and builds `CampaignExecutionReport` through a dedicated helper, so `execute_campaign` no longer hand-splices tuple fields into the final report payload
   - twenty-first landing: `diag_campaign` now builds final per-campaign execution outcomes through dedicated outcome/error helpers, so summarize-failure priority and failed-item wording no longer stay inline inside `execute_campaign_inner`
   - twenty-second landing: `diag_campaign` now builds single-run and batch result artifacts through dedicated payload helpers driven by `plan + summary_artifacts`, removing another pair of wide writer signatures and giving result JSON shaping direct regression coverage
   - twenty-third landing: `diag_campaign` now builds final execution and batch outward artifacts through dedicated builder helpers, so finalization no longer re-splices summarize/share outputs into command-facing structs inline
   - twenty-fourth landing: `diag_campaign` now reuses a named aggregate-artifact contract for summary/index/share paths and summarize/share errors, reducing field drift across finalization, batch artifacts, and result payload assembly
   - twenty-fifth landing: `diag_campaign` now also stores per-campaign report artifact paths and share-export state through the same aggregate contract, removing another parallel path/error shape from `CampaignExecutionReport` and keeping report JSON/output helpers aligned with batch/finalization
   - twenty-sixth landing: `diag_campaign` now reuses a shared aggregate path-projection helper for report and batch JSON emitters, so run-outcome and artifact-style JSON keep the same summary/index/share visibility rules without duplicating path logic
   - twenty-seventh landing: `diag_campaign` now routes top-level run-outcome JSON through dedicated counters/batch/runs helpers, so `campaign_run_outcome_to_json` no longer hand-splices the same payload sections inline and its CLI JSON shape gains direct regression coverage
   - twenty-eighth landing: `diag_campaign` now splits per-report JSON assembly into dedicated status/paths/counters helpers, so `campaign_report_json` no longer grows as one long field-insertion block and each projection slice can be tested independently
   - twenty-ninth landing: `diag_campaign` now splits batch JSON assembly into dedicated root/paths/status helpers, so `campaign_batch_to_json` follows the same decomposition style as per-report JSON and avoids another inline emitter blob
   - thirtieth landing: `diag_campaign` now reuses dedicated `run` and `aggregate` helpers across single-run and batch result payload assembly, so `campaign_result_payload` and `campaign_batch_result_payload` no longer duplicate the same result-artifact section pair inline
   - thirty-first landing: `diag_campaign` now reuses dedicated manifest payload helpers across single-run and batch manifest writing, so `write_campaign_manifest` and `write_campaign_batch_manifest` mainly own output-path resolution plus file IO while manifest JSON shaping gets direct regression coverage
   - thirty-second landing: `diag_campaign` now separates per-item execution planning from suite-context assembly, so item kind/path/script-input selection no longer grows in the same helper that wires runtime flags and checks into `diag_suite::SuiteCmdContext`
   - thirty-third landing: `diag_campaign` now routes per-item suite success/error mapping through a dedicated item-result helper, so `run_campaign_item` no longer open-codes the same `CampaignItemRunResult` projection inline after each `diag_suite` execution
   - thirty-fourth landing: `diag_campaign` now separates batch item planning from plan consumption, so `execute_campaign_items` no longer mixes campaign-item enumeration with "run planned items" in the same loop body and the multi-item planning stage gains direct regression coverage
   - thirty-fifth landing: `diag_campaign` now builds a shared summary-finalize plan for single-run and batch finalize paths, so summarize inputs, output roots, timestamps, and failure-share conditions stop being re-derived inline in two separate finalize branches
   - thirty-sixth landing: `diag_campaign` now builds dedicated result-write plans for single-run and batch result artifacts, so output-path resolution and payload shaping are settled before file IO and the write layer no longer duplicates the same "path + payload + write" pattern inline
   - thirty-seventh landing: `diag_campaign` now builds dedicated result-payload section bundles for single-run and batch result artifacts, so payload roots no longer open-code the same run/counters/aggregate/list section planning inline before composing the final JSON object
   - thirty-eighth landing: `diag_suite` now builds core default post-run checks through a dedicated helper, so viewport/vlist/view-cache/retained/gc default planning no longer expands inline beside bundle wait and doctor orchestration
   - thirty-ninth landing: `diag_suite` now builds editor/markdown/text default post-run checks plus merge wiring through dedicated helpers, so policy-heavy boolean gate planning no longer lives in the same orchestration block as bundle resolution and summary accounting
   - fortieth landing: `diag_suite` now routes explicit-or-policy post-run trigger decisions through a dedicated helper, so trigger growth no longer stays as one expanding OR-chain inline in `cmd_suite`
   - forty-first landing: `diag_suite` now routes retained-vlist script-specific overrides through `SuiteScriptOverrideChecks`, so trigger planning and post-run application reuse the same per-script override seam instead of filtering the same checks twice
   - forty-second landing: `diag_suite` now routes suite success/failure summary payload assembly and emission through `SuiteSummaryEmitInput` plus dedicated helpers, so setup failures, run failures, lint failures, and pass-result writing no longer duplicate payload/write/regression-summary plumbing inline
   - forty-third landing: `diag_suite` now builds tooling-error rows and script-result rows through dedicated helpers, so setup/tooling/script/lint outcome payloads stop open-coding the same JSON fragments inline beside execution control flow
   - forty-fourth landing: `diag_suite` now routes stop-demo, summary emit, and return/exit decisions through dedicated failure-finalization helpers plus a shared summary context, so setup/run/lint failure branches stop repeating the same cleanup + summary plumbing inline
   - forty-fifth landing: `diag_suite` now routes tooling failure script-result writes, row shaping, and main finalize wiring through dedicated helpers, so DevTools/connect/launch failure branches stop repeating the same tooling-error bookkeeping inline
   - forty-sixth landing: `diag_suite` now routes failed/unexpected/lint-failed script outcomes through a dedicated exit helper, so row shaping and failure finalization no longer interleave inline with outcome logging in those branches
   - forty-seventh landing: `diag_suite` now prepares per-script stage/reason accounting plus evidence/lint context through a dedicated helper, so each script iteration no longer reassembles that bookkeeping inline after transport execution completes
   - forty-eighth landing: `diag_suite` now routes dump-label planning, `run_script_over_transport` lowering, and `tooling.suite.error` fallback through dedicated helpers, so the transport result decoding path no longer stays inline in one execution block
   - forty-ninth landing: `diag_suite` now routes prewarm/prelude execution and load-script wiring through a dedicated execution-block context, so the main script loop no longer interleaves that setup with transport result decoding in one large closure
   - fiftieth landing: `diag_suite` now routes per-script launch env/default assembly and connected transport acquisition through `SuiteScriptLaunchRequest` plus `SuiteScriptTransportRequest` / `SuiteScriptTransportSelection`, so `maybe_launch_demo` and filesystem-vs-DevTools selection no longer stay inline in the main script loop
   - next recommended landing: collapse the remaining per-script execution closure so transport-backed `SuiteScriptExecutionBlockContext` assembly and `execute_suite_script_iteration_block` invocation live behind one helper seam
   - remaining known holdouts: a few session-root-only helpers that intentionally do not require bundle materialization,
 3. stabilize metadata and evidence vocabulary beyond the current first pass (`owner`, `platforms`, `tier`, `expected_duration_ms`, `tags`, capability tags, flake policy),
 4. add richer lane composition (`matrix`, `perf`, `nightly/full`) only after the first seam slices settle,
 5. decide whether campaign runs should persist a dashboard text or HTML projection,
 6. evaluate whether campaign manifests should become JSON-only, TOML, or generated registry inputs long-term,
7. remove legacy top-level `suites` / `scripts` compatibility only after manifest authoring and evidence contracts stabilize.

Important rule:

- campaign should initially compose existing run primitives, not replace them.

Gate expectations:

- one golden test or fixture proving lane expansion,
- one end-to-end doc example per first implemented lane.

## Phase F ?DevTools GUI and MCP alignment

Goal:

- let presentation surfaces consume campaigns and summaries without defining their own semantics.

Current status:

- A first thin consumer now exists via `fretboard diag dashboard`, which reads
  `regression.index.json` for human-oriented inspection.
- The next step in this phase is no longer “whether a consumer is useful? but how GUI/MCP
  should reuse the same index/summary contracts without forking semantics.
- pps/fret-devtools-mcp now exposes 
egression.summary.json and
  
egression.index.json as MCP resources when those artifacts exist in the current
  artifacts root.

Primary code areas:

- `apps/fret-devtools-mcp/src/native.rs`
- DevTools GUI implementation surfaces referenced by `docs/workstreams/diag-devtools-gui-v1.md`
- offline viewer, if summary browsing becomes useful there

Recommended PR slices:

1. [done] expose summary artifact paths/resources,
2. [in progress] add campaign-aware run triggers in MCP or GUI,
   - `apps/fret-devtools-mcp` now provides a thin `fret_diag_regression_summarize` bridge over
     `fretboard diag summarize`, including session-scoped MCP resource update notifications,
   - `apps/fret-devtools-mcp` now also provides `fret_diag_regression_dashboard` as a thin
     consumer over `regression.index.json`,
3. [in progress] add summary browsing panels,
   - `apps/fret-devtools` now includes a first read-only `Regression` details tab over the shared aggregate artifacts,
   - the tab now supports failing-summary drill-down and bundle-dir path copying without introducing a new campaign model,
   - selected summaries now expose a direct `Copy first bundle dir` action for faster evidence handoff into external triage/viewer flows,
   - selected summaries can now start the existing pack flow directly from the first failing bundle dir,
   - the tab now also provides a thin `Summarize` trigger that runs `fretboard diag summarize` semantics from the current artifacts root and refreshes the shared aggregate artifacts on completion,
4. [in progress] add flake/evidence drill-down UX.

Important rule:

- GUI/MCP should read shared summary and artifact contracts rather than inventing their own run model.

## Phase G ?Metadata and selection scaling

Goal:

- make campaign selection sustainable as suite count grows.

Primary code areas:

- `tools/diag-scripts/suites/`
- `crates/fret-diag/src/registry/suites.rs`
- future metadata parsers/helpers in `crates/fret-diag`

Recommended PR slices:

1. define suite/script metadata shape,
2. teach registry to read metadata,
3. support filtering by lane/tier/tag/platform,
4. document authoring rules for new suites.

Gate expectations:

- metadata parsing tests,
- one or two real suites migrated as proof.

## 3) Suggested PR order

Recommended near-term order:

1. Phase B small seam extraction
2. Phase D summary model and summary emission across core lanes
3. Phase E first `smoke` lane or campaign index surface
4. Phase C runtime evidence normalization where needed
5. Phase F presentation surface alignment
6. Phase G suite metadata scaling

Reasoning:

- summary generation already unlocks CLI, CI, GUI, and MCP together,
- campaign entrypoints are easier once a summary/evidence model exists,
- metadata scaling should wait until the first lane model is proven useful.

## 4) What not to do first

Avoid these as initial moves:

- splitting new crates before internal seams are stable,
- rewriting DevTools GUI first,
- introducing a new scripting language,
- over-designing distributed execution,
- deleting older docs before the replacement paths are proven.

## 5) Evidence and gate expectations by phase

Every phase should leave behind at least one of:

- a unit/integration test,
- a diag script or suite,
- a summary artifact example,
- a documented end-to-end command.

Suggested minimums:

- Phase B: tests around new orchestration helpers
- Phase C: one end-to-end evidence contract example
- Phase D: summary serialization + fixture tests
- Phase E: one implemented lane with a stable example command
- Phase F: one thin consumer example is landed; next is GUI/MCP adoption
- Phase G: metadata parsing + filtering tests

## 6) Definition of done for this roadmap

This roadmap is doing its job when:

- the next implementation PR can cite a phase and a small scope,
- v2 no longer reads like architecture-only prose,
- contributors can move from docs to code entry points without guesswork,
- diagnostics refactoring proceeds as a sequence of additive, gated steps.
