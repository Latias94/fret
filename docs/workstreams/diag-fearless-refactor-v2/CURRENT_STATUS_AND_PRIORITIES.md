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

- `fretboard-dev diag campaign list`
- `fretboard-dev diag campaign show <campaign_id>`
- `fretboard-dev diag campaign run <campaign_id>`
- filtered batch execution via `--lane`, `--tier`, `--tag`, and `--platform`
- `fretboard-dev diag campaign share <campaign_or_batch_root>`

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

### 4. The M3 vocabulary contract is now partially adopted in implementation

The repo-level vocabulary note is no longer only a doc artifact.

Recent implementation adoption now covers:

- shared summary serialization/deserialization compatibility in
  `crates/fret-diag/src/regression_summary.rs`,
- aggregate summarize wording and canonical lane fallback in
  `crates/fret-diag/src/diag_summarize.rs`,
- shared dashboard human wording in `crates/fret-diag/src/diag_dashboard.rs`,
- campaign share-manifest payload field additions in `crates/fret-diag/src/diag_campaign.rs`,
- run-manifest `paths` canonical naming plus legacy aliases in
  `crates/fret-diag/src/run_artifacts.rs`,
- artifact-lint compatibility with canonical `script_result` path naming in
  `crates/fret-diag/src/artifact_lint.rs`.

The contract note itself is also stronger now:

- it includes a first persisted-field normalization map across summary, aggregate, campaign, share,
  run-manifest, and Layer B evidence payload surfaces,
- it now defines explicit writer-first and reader-first compatibility rules instead of only saying
  "evolve additively",
- it now names a recommended adoption order so follow-up work can tighten machine artifacts before
  reopening more presentation or CI wrappers.

This matters because the contract is now starting to shape persisted artifacts and shared consumer
wording rather than only describing a future target state.

## What is now "good enough" and should not lead the roadmap

The following areas are useful, but they should no longer be the primary driver for the next phase:

- basic campaign command discoverability,
- first-pass share helpers,
- first-pass failure evidence packaging,
- GUI polish that does not change contracts,
- dashboard text or HTML projection.

These are now in the "maintain and extend carefully" bucket, not the "main strategic blocker" bucket.

## What is effectively closed vs still open

### Effectively closed for now

- aggregate dashboard presentation reuse across CLI / MCP / GUI consumers,
- `diag_campaign` CLI/report/finalize presentation seams,
- `commands::artifacts` command-entry output seams for:
  - `cmd_triage`,
  - `cmd_lint`,
  - `cmd_test_ids`,
  - `cmd_test_ids_index`,
  - `cmd_frames_index`,
  - `cmd_meta`,
  - `cmd_pack`,
- `diag_run` and `diag_suite` high-ROI orchestration slicing.

These areas may still have small tail cleanups, but they no longer justify being the default next
refactor target unless a concrete bug, drift, or review blocker appears.

### Still open and worth doing

- repo-level adoption of the artifact/evidence contract in maintainer-facing docs and flows,
- finishing M3 vocabulary adoption in the remaining persisted fields and consumer paths,
- residual naming cleanup where canonical names still coexist with legacy ones (especially manifest
  `files[].id`-style identifiers and older consumer-side assumptions),
- `commands::artifacts` final tail cleanup only where a helper still mixes more than one hard
  concern and produces a reviewable patch,
- `commands::resolve` only if another clearly reviewable small tail appears.

## The highest-value next priorities

### Priority 1. Tighten the shared M3 vocabulary contract

Why it is first:

- the contract note now exists and initial implementation adoption has started,
- multiple consumers now exist and already share campaign, summary, and aggregate artifacts,
- the highest remaining repo-level drift risk is no longer "do we have a campaign model?" but
  "are lane names, reason codes, flake policy, capability tags, and evidence paths stable enough
  that new tooling does not fork the vocabulary again?"

Concretely, the next adoption slice should settle:

- remaining persisted lane/status/reason/path fields that still need canonical names,
- remaining human wording that still implies legacy names,
- additive compatibility rules for legacy field aliases,
- and the requirement that CLI / GUI / MCP / CI reuse the same persisted and human-facing terms.

The contract note now already includes:

- a first persisted-field normalization map,
- writer/reader alias lifecycle rules,
- and a recommended adoption order.

So the next value is no longer another terminology draft.
It is a bounded implementation audit against that contract.

Recommended landing target:

- `docs/workstreams/diag-fearless-refactor-v2/M3_ORCHESTRATION_VOCABULARY_AND_CONTRACT_V1.md`
- success after that is additive adoption: align campaign, summary, batch, dashboard, GUI, MCP,
  and CI wording around the same persisted vocabulary while keeping legacy aliases readable.

The first bounded implementation audit is now also captured in:

- `docs/workstreams/diag-fearless-refactor-v2/M3_VOCABULARY_ADOPTION_AUDIT.md`

That audit narrows the next concrete implementation targets to:

- explicit classification of `RegressionArtifactsV1.index_json`, `perf_summary_json`, and
  `compare_json`,
- keeping DevTools/MCP raw `*json` text-holder names deferred unless those modules are already
  being changed for another reason.

The first follow-up adoption slice is now landed:

- campaign manifests and built-in registry definitions can now express
  `requires_capabilities` and `flake_policy` additively,
- manifest loading keeps older files compatible while normalizing capability tags and flake-policy
  values,
- `diag campaign list/show` now surfaces those fields without introducing new selector behavior.

The next bounded audit is now also written down:

- `docs/workstreams/diag-fearless-refactor-v2/CAMPAIGN_METADATA_EXECUTION_ADOPTION_AUDIT.md`
- `docs/workstreams/diag-fearless-refactor-v2/DEVTOOLS_MCP_RAW_JSON_DEFER_AUDIT.md`

The next bounded design note is now also written down:

- `docs/workstreams/diag-fearless-refactor-v2/CAMPAIGN_CAPABILITY_PREFLIGHT_V1.md`

That audit makes one important constraint explicit:

- the runner does **not** yet consume `requires_capabilities` or `flake_policy` as execution
  behavior,
- and it should stay that way until there is a concrete need for campaign-level capability
  preflight or retry orchestration.

The new preflight note narrows what the first behavior slice should be if that need appears:

- first implement campaign capability preflight,
- reuse the existing `capability.missing` reason-code family,
- prefer `skipped_policy` over a new status,
- and emit one machine-readable campaign-local check artifact before item execution.

That first implementation slice is now partially landed:

- campaign execution now performs a real capability preflight before item execution,
- capability mismatch now writes campaign-local `check.capabilities.json`,
- campaign summaries now emit a synthetic `CampaignStep` row with `skipped_policy` +
  `capability.missing`,
- campaign run/report JSON now exposes `capabilities_check_path`,
- batch and single-run human output now distinguish policy skips from ordinary failures,
- run counters now also expose `campaigns_skipped_policy`.

The capability-source side of that behavior is now also aligned:

- campaign preflight now reuses the shared filesystem capability loader in
  `crates/fret-diag/src/lib.rs`,
- the shared loader now follows the same fallback order used by diagnostics tooling:
  direct, `_root`, then parent `capabilities.json`,
- `diag doctor` now reports normalized capabilities from that same resolved source path,
- the first-pass non-filesystem direction is now documented in
  `NON_FILESYSTEM_CAPABILITY_SOURCE_V1.md` as an additive provenance contract instead of a
  replacement for filesystem paths,
- the next bounded implementation sketch is now captured in
  `CAPABILITY_PROVENANCE_MINIMAL_IMPLEMENTATION_V1.md`, which keeps legacy path fields and stages
  additive `capability_source` adoption separately from any real transport work,
- `fret-diag` now has a shared internal `CapabilitySource` helper so filesystem and transport-backed
  code paths stop open-coding provenance labels and transport identity separately,
- additive `capability_source` payloads are now emitted by `diag doctor`, campaign preflight
  summary evidence/metadata, and campaign aggregate/result payloads while keeping
  `capabilities_path`, `capabilities_source_path`, and `capabilities_check_path` readable.

The DevTools/MCP defer audit makes another boundary explicit:

- app-local `*json` names in DevTools GUI and MCP are mostly raw JSON text-holder state or
  request/response payload text,
- so they should not be treated as persisted contract drift unless those fields start carrying
  canonical artifact-path semantics.

The first non-CLI policy-skip consumer-adoption slice is now also landed:

- the DevTools `Regression` inspector now treats selected non-passing summaries as evidence sets
  that may include both bundle directories and campaign-local capability-check artifacts,
- policy-skipped summaries no longer show up only as generic "failing" rows in the shared
  dashboard wording,
- DevTools `Regression` drill-down now also surfaces capability provenance as a distinct
  `Capability Sources` evidence lane instead of collapsing it into the check-artifact list,
- MCP dashboard output now also surfaces capability provenance and capability-check paths from the
  sibling `regression.summary.json` when present,
- MCP dashboard tests now lock `skipped_policy` counters, capability provenance fallback, and the
  shared `non-passing summaries` human-summary wording.

That means the next concrete implementation targets are now:

- keeping DevTools/MCP raw `*json` text-holder names deferred unless those modules are already
  being changed for another reason,
- classifying any future optional evidence fields before a new shared consumer depends on them,
- finishing the remaining maintainer-facing docs and any future non-DevTools/non-MCP consumer
  adoption for the new campaign policy-skip breakdown and capability-source fields,
- and only then reopening campaign metadata behavior such as `flake_policy` if CI or batch
  orchestration needs it.

### Priority 2. Keep adopting the canonical artifact/evidence contract

Why it is second:

- the first artifact/evidence contract now exists,
- multiple commands already emit related evidence (`summary`, `index`, `triage`, `ai.packet`, share
  zips, screenshot manifests),
- the remaining value is now mostly adoption across maintainer-facing docs, flows, and persisted
  field names rather than another large drafting pass.

Concretely, the next adoption slice should settle:

- where maintainer docs tell people to open first,
- which top-level files GUI / MCP / CI should prefer,
- how campaign/batch/share roots map back to the canonical artifact taxonomy.

Recommended landing target:

- keep `ARTIFACT_AND_EVIDENCE_MODEL_V1.md` as the contract note,
- add small cross-links and wording alignment in maintainer-facing docs instead of creating another
  parallel artifact note family,
- prefer additive migration in persisted artifacts (`new canonical field` + `legacy alias read` or
  `temporary dual-write`) instead of a break-first rename.

Recent adoption in this priority bucket:

- run-manifest `files[].id` now writes canonical `script_result` while still reading legacy
  `script_result_json`,
- Layer B `bundle_artifact` additive adoption is now landed in `diag_repro` and `diag_repeat`,
- the shared stats payload helper now also covers:
  - notify/gc/stale evidence payloads,
  - `debug_stats_gates`,
  - `frames_index_gates`,
  - `gc_gates_streaming`,
  - `retained_vlist_gates`,
  - `view_cache_gates`,
  - `vlist`,
  - `windowed_rows`.

What this means for the roadmap:

- the next highest-value naming work is no longer another generic seam split,
- it is the remaining small-batch Layer B residual naming cleanup where `bundle_artifact` should be
  primary and `bundle_json` should remain only as a compatibility alias.
- the helper-based stats cleanup plus `evidence_index` reader adoption now cover the main small
  additive naming wins inside `crates/fret-diag`,
- so the next naming work is no longer "keep scanning for another tiny reader-side fix", but either:
  - stop at the current compatibility boundary,
  - or schedule one deliberate internal rename wave later for `RegressionEvidenceV1`-style legacy
    Rust field names.

### Priority 3. Extract the next 2-3 high-ROI seams in `crates/fret-diag`

Why it is second:

- the command surface is usable enough now,
- the real medium-term risk is continued growth in a few orchestration-heavy files.

Current hotspots worth treating as explicit seam candidates:

- `crates/fret-diag/src/diag_suite.rs` (summary/failure emit factoring, per-row payload shaping, failure-finalization helpers, tooling-failure handling helpers, script-outcome handlers, per-script context assembly, transport result decoding, script-execution block assembly, per-script launch/transport acquisition, execution dispatch, success-path bundle/lint/post-run preparation, result-finalization stage/success helpers, and success-tail orchestration have landed; the remaining holdouts in this file are now mostly one-time setup and session-root-adjacent helpers, so this file is no longer the default next seam target)
- `crates/fret-diag/src/diag_run.rs` (the first five `cmd_run` seams are now landed: transport result stage normalization, bundle doctor/post-run checks, bundle artifact emission, demo-exit-killed marking, failure dump bundle backfill, bundle-path resolution/wait, filesystem post-run finalization, both transport branch adapters, and the remaining top-level option/policy setup now all reuse dedicated helpers; `cmd_run` is now mostly resolved-path setup plus transport dispatch, so this file is effectively parked unless a clearly reviewable sixth seam appears)
- `crates/fret-diag/src/diag_campaign.rs` (the run/result/report/finalize seams are already in much better shape, `write_campaign_share_manifest` now routes both per-item evidence planning and the final payload/combined-zip update through dedicated helpers plus named counters/combined-entry/outcome shapes, `write_campaign_combined_failure_zip_inner` now also routes root/item export staging through dedicated zip-entry planners with focused helper tests, `write_campaign_share_manifest` now also consumes a dedicated `CampaignShareManifestItems` aggregate so the per-item collection loop is no longer open-coded inline, `build_campaign_share_manifest_item` now consumes a dedicated `CampaignShareManifestItemArtifacts` snapshot so artifact IO and run-entry shaping stop sharing one block, `write_campaign_share_manifest` now also consumes a dedicated `CampaignShareManifestWritePlan` while combined-zip update routes through `finalize_campaign_share_manifest_write`, `collect_campaign_share_manifest_item_artifacts` now delegates bundle-dir resolution, supporting-artifact collection, and share-zip planning to dedicated helpers, `build_campaign_share_manifest_payload` now consumes dedicated source/selection/counters/share section helpers, and `apply_campaign_share_manifest_combined_zip` now routes through dedicated field-building and share-section apply helpers, `execute_campaign_run_selection` now routes counters/failure aggregation through a dedicated `build_campaign_run_outcome` helper, the shared summary-finalize path now routes summarize/share execution through `execute_campaign_summary_finalize_outcome` plus timing/materialization through `build_campaign_summary_artifacts`, batch artifact handoff now routes manifest/finalize setup through `CampaignBatchArtifactWritePlan` plus `build_campaign_batch_manifest_write_plan`, the single-campaign startup path now routes execution-plan plus manifest setup through `CampaignExecutionStartPlan` plus `build_campaign_manifest_write_plan`, the single-campaign finalize path now routes failure counting plus summary-finalize setup through `CampaignExecutionFinalizePlan`, the single-campaign report handoff now routes result normalization plus report construction through `build_campaign_execution_report_from_outcome_result`, and campaign-run CLI output now also routes through a dedicated `CampaignRunOutputPresentation` seam; the next highest-ROI slice here is no longer another CLI output helper but pushing the same aggregate presentation vocabulary into non-CLI consumers such as `apps/fret-devtools-mcp/src/native.rs`)
- `crates/fret-diag/src/commands/resolve.rs` (`diag resolve latest` now routes option parsing and JSON/text rendering through dedicated pure helpers with direct regression coverage, the deeper session-resolution path now routes target session-id selection plus existing-session directory validation through dedicated helpers, `resolve_script_result_json_path_or_latest` now routes latest-vs-src search-start selection through dedicated helpers, the shared bundle resolution path now routes source-path selection, bundle-ref derivation, and artifacts-root policy through dedicated helpers, latest run/bundle output projection now routes through dedicated helpers reused by both `resolve_latest_for_out_dir` and `resolve_latest_bundle_dir_from_base_or_session_out_dir`, `resolve_latest_bundle_dir_for_out_dir` now routes script-result hint normalization plus latest-marker/scan fallback through dedicated helpers, and `resolve_session_out_dir_for_base_dir` now routes session-root vs base-dir selection through a dedicated mode helper plus direct-resolution builder, so the remaining work here is down to only small tail holdouts rather than top-level command-blob cleanup or a default next seam target)
- `crates/fret-diag/src/commands/artifacts.rs` (`cmd_pack` now routes bundle/source resolution and default out-path selection through a dedicated setup helper plus pure out-path logic while ai.packet path resolution, best-effort ensure, and `--ai-only` directory validation now also route through dedicated helpers, the repeated single-bundle emitter commands now also reuse shared bundle-input plus path/json output helpers, `cmd_meta` now routes its human-readable projection through pure report-line helpers and now also routes direct-sidecar vs bundle-dir vs bundle-path canonical-meta resolution through dedicated helpers with regression coverage while parse/resolve/out preparation now also routes through a dedicated helper, canonical artifact output materialization now also routes through a shared helper reused by `cmd_meta` and `cmd_test_ids`, repeated ensured-artifact emitters now also route through a shared helper reused by `cmd_test_ids_index` and `cmd_frames_index`, `cmd_lint` now routes bundle/out-path preparation through a dedicated helper while lint report write plus exit-required decision now also route through a dedicated helper, `cmd_triage` now routes bundle resolution plus default/custom out-path selection through a dedicated prepare helper, `cmd_test_ids` now routes bundle/out preparation plus cached-output short-circuit through dedicated helpers, and the shared output surface now routes text reads, JSON reads, and pretty JSON text shaping through dedicated helpers reused by `emit_path_or_json_output`, `emit_artifact_output`, and `write_json_artifact_output`; the larger remaining holdouts here are now the deeper artifact update/materialization tails that still do more than one concern at a time rather than top-level emitter parsing)

Recommended next seam choices:

1. artifact resolution and materialization holdouts,
2. M3 orchestration contract tightening around reason-code / artifact-path vocabulary,
3. `commands::resolve` small tail holdouts only if another clearly reviewable seam appears.

Near-term execution plan (1-2 weeks):

1. land and adopt the M3 orchestration vocabulary contract around stable reason codes,
   artifact/evidence path vocabulary, flake policy, capability tags, and lane naming,
2. keep adopting the artifact/evidence contract in maintainer-facing docs and flows,
3. finish only the remaining reviewable artifact resolution/materialization tail in
   `commands::artifacts`,
4. only reopen aggregate presentation reuse if another consumer still owns a parallel projection
   path,
5. defer further DevTools GUI alignment until the artifact model and presentation seams are stable
   enough that UI polish does not lock in the wrong backend shape.

Recent progress since this note was drafted:

- the first repo-level artifact/evidence contract now exists in
  `ARTIFACT_AND_EVIDENCE_MODEL_V1.md`, which names source-of-truth artifacts, derived/index
  artifacts, optional evidence, presentation-facing projections, the bounded first-open set, and
  consumer checklists for CLI / GUI / CI / share flows in one place,
- `diag_campaign` share-manifest orchestration now also has dedicated aggregation and artifact-snapshot
  seams, so include-passed filtering, missing-bundle handling, and run-entry shaping are directly
  testable without running the full share export path,
- `diag_campaign` share-manifest orchestration now also has dedicated write-plan and finalize-handoff
  seams, so initial payload/output-path planning and combined-zip path recording are directly testable
  without going through full campaign execution,
- `diag_campaign` share-manifest item artifact planning now also has dedicated bundle-dir,
  supporting-artifact, and share-zip helpers, so triage/screenshots/zip behavior is directly testable
  without reopening the full item-artifact collector,
- `diag_campaign` share-manifest payload shaping now also has dedicated section helpers, so source,
  selection, counters, and share field evolution is directly testable without reopening the full
  payload builder,
- `diag_campaign` share-manifest combined-zip mutation now also has dedicated field-building and
  section-apply helpers, so final share-field updates are directly testable without reopening the
  full finalize helper,
- `commands::artifacts` meta canonical-path resolution now also has dedicated source-kind helpers, so
  sidecar reuse, fallback regeneration, and `_root` preference are directly testable without reopening
  the full `resolve_meta_artifact_paths` branch ladder,
- `commands::artifacts` canonical artifact output materialization now also has a shared helper reused by
  `cmd_meta` and `cmd_test_ids`, so same-path no-op, existing-out reuse, and nested custom-out copy are
  directly testable without reopening each command's inline copy branch,
- `commands::artifacts` generated-artifact materialization now also routes through a dedicated
  `ArtifactMaterializationPlan`, so existing-out reuse, canonical-path no-op, and copy-to-custom-out
  execution are directly testable without reopening the full generated-artifact output helper,
- `commands::artifacts` pack command output now also routes through a dedicated `PackCommandOutput`
  seam, so zip execution and terminal path presentation no longer stay coupled to a one-off
  `println!` tail and the output handoff now matches the rest of the command family,
- `commands::artifacts` lint command output now also reuses `build_lint_report_output` directly at
  the command boundary, so report write, presentation, and exit-policy handoff now follow the same
  build-then-emit pattern as the rest of the command family instead of relying on a dedicated
  side-effect helper,
- `commands::artifacts` meta resolution now also routes sidecar validity checks and bundle-dir
  sidecar preference through dedicated helpers, so direct-vs-`_root` selection and existing-sidecar
  reuse no longer stay open-coded across both meta resolution branches,
- `commands::artifacts` repeated ensured-artifact emitters now also have a shared helper reused by
  `cmd_test_ids_index` and `cmd_frames_index`, so success, ensure-error, and emit-error behavior are
  directly testable without reopening each command's inline ensure-and-emit branch,
- `commands::artifacts` ensured-artifact emitters now also route through a dedicated
  `EnsuredBundleArtifactPlan`, so required-input resolution, warmup/display shaping, and ensure-output
  projection are directly testable without reopening the full output helper,
- `commands::artifacts` repeated bundle-input plus custom/default out setup now also routes through
  shared prepare helpers, so `cmd_lint`, `cmd_test_ids`, `cmd_triage`, and `cmd_meta` no longer each
  inline their own out-path selection and that handoff is directly testable at the helper layer,
- `commands::artifacts` test-ids command tail now also routes through a dedicated
  `TestIdsExecutionPlan`, so generated-artifact display mode, `warmup_frames`, and `max_test_ids`
  shaping are directly testable without reopening `cmd_test_ids`,
- `commands::artifacts` triage setup now also has a dedicated prepare helper, so bundle resolution and
  lite default-vs-custom out-path policy are directly testable without reopening `cmd_triage`,
- `commands::artifacts` triage execution now also routes through a dedicated `TriageExecutionPlan`, so
  payload mode selection, stats-top/warmup shaping, and JSON-vs-path output projection are directly
  testable without reopening `cmd_triage` parameter plumbing,
- `commands::artifacts` meta command tail now also routes through a dedicated `MetaExecutionPlan`, so
  canonical-path handoff, output target, and display-mode projection are directly testable without
  reopening `cmd_meta`,
- `commands::artifacts` output surface now also has dedicated text-read, JSON-read, and pretty-text
  helpers, so file-read / JSON-parse / pretty-format behavior is directly testable without reopening
  `emit_path_or_json_output`, `emit_artifact_output`, or `write_json_artifact_output`,
- `commands::artifacts` test-ids setup now also has a dedicated prepare helper plus existing-file
  shortcut helper, so default/custom out-path selection and cached-output reuse are directly testable
  without reopening `cmd_test_ids`,
- `commands::artifacts` meta setup now also has a dedicated prepare helper, so parse/resolve/out
  selection and display-mode preparation are directly testable without reopening `cmd_meta`,
- `commands::artifacts` lint output now also has a dedicated report-write/exit helper, so JSON output
  emission and exit-required behavior are directly testable without reopening `cmd_lint`,
- `commands::artifacts` pack preflight now also has dedicated ai.packet helpers, so ai.packet path
  shaping, best-effort ensure, and `--ai-only` validation are directly testable without reopening
  `cmd_pack`,
- `commands::resolve` latest output assembly now also has dedicated run/bundle projection helpers, so
  latest-run-dir reuse, latest-bundle artifact filtering, and base/session bundle projection are directly
  testable without reopening the full `resolve_latest_for_out_dir` flow,
- `commands::resolve` latest-bundle-dir selection now also has dedicated hint/fallback helpers, so
  `script.result.json:last_bundle_dir` normalization and `latest.txt_or_scan` fallback are directly
  testable without reopening `resolve_latest_bundle_dir_for_out_dir`,
- `commands::resolve` session-root/base-dir selection now also has a dedicated mode helper and
  direct-resolution builder, so nested session-marker preference and no-sessions direct resolution are
  directly testable without reopening `resolve_session_out_dir_for_base_dir`,
- artifact resolution/materialization now has shared seams for bundle input resolution,
  `script.result.json` discovery, `diag resolve latest` option/render shaping, session-target
  selection / existing-session directory validation, latest-vs-src script-result search-start
  normalization, and shared bundle source/ref/artifacts-root derivation under
  `crates/fret-diag/src/commands/resolve.rs`, `commands::artifacts` emitter setup/display shaping
  for `cmd_pack`, the repeated single-bundle emitters, `cmd_meta`, `cmd_lint`, and `cmd_triage`,
  plus `commands/artifact.rs` setup/write/exit shaping for `cmd_artifact_lint`,
- `diag_campaign` now also shares run-outcome assembly and summary-finalize execution/timing across
  single-run and batch finalize flows, so counters/failure aggregation plus summarize/share
  materialization no longer sprawl inline after report collection or inside
  `finalize_campaign_summary_artifacts`,
- `diag_campaign` now also settles batch manifest output-path/payload shaping plus summary-finalize
  setup before IO via `CampaignBatchArtifactWritePlan` and `build_campaign_batch_manifest_write_plan`,
  so `write_campaign_batch_artifacts` mainly owns orchestration instead of re-deriving manifest and
  finalize setup inline,
- `diag_campaign` now also settles single-campaign execution-plan plus manifest setup before IO via
  `CampaignExecutionStartPlan` and `build_campaign_manifest_write_plan`, so `execute_campaign` and
  `execute_campaign_inner` no longer thread raw startup setup inline,
- `diag_campaign` now also settles single-campaign failure counting plus summary-finalize setup via
  `CampaignExecutionFinalizePlan`, so `finalize_campaign_execution` mainly owns finalize execution
  rather than re-deriving those inputs inline,
- `diag_campaign` now also settles single-campaign result normalization plus report construction via
  `build_campaign_execution_report_from_outcome_result`, so `execute_campaign` no longer threads that
  handoff inline after startup execution returns,
- `diag_campaign` now also routes campaign-run CLI output through `CampaignRunOutputPresentation`,
  so JSON-vs-human output selection no longer lives inline in `print_campaign_run_output` and the
  aggregate presentation vocabulary now extends beyond `diag_summarize` / `diag_dashboard`,
- `apps/fret-devtools-mcp/src/native.rs` now also reuses `fret-diag`'s shared dashboard projection
  and human-summary helpers, so counter/reason/failing-summary parsing plus text assembly no longer
  drift in a parallel MCP-only path,
- `apps/fret-devtools/src/native.rs` now also reuses `fret-diag`'s shared dashboard human-summary
  and failing-summary projection helpers, so the GUI `Regression` tab no longer maintains another
  parallel dashboard text/row parsing path,
- `crates/fret-diag/src/commands/artifacts.rs` now also routes `cmd_triage` through a dedicated
  `TriageCommandOutput` seam built on the shared JSON-write presentation helper, so triage payload
  generation and terminal emission no longer stay interleaved in one command-level block,
- `crates/fret-diag/src/commands/artifacts.rs` now also routes ensured-artifact commands through a
  dedicated `EnsuredBundleArtifactOutput` seam, so ensure-time output-path resolution and
  JSON-vs-path presentation shaping no longer stay interleaved in one helper,
- `crates/fret-diag/src/commands/artifacts.rs` now also routes required-bundle ensured commands
  through `build_required_bundle_artifact_output`, so `cmd_test_ids_index` / `cmd_frames_index`
  no longer mix required-input resolution with ensured-output projection inline,
- `crates/fret-diag/src/commands/artifacts.rs` now also routes generated-artifact materialization
  through `GeneratedArtifactOutput`, so `cmd_test_ids` / `cmd_meta` no longer depend on one
  monolithic materialize-and-emit helper for the final output handoff,
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
- `diag_suite` now also routes stop-demo, summary emit, and return/exit decisions through
  dedicated failure-finalization helpers plus a shared summary context, so setup/run/lint failure
  branches no longer repeat the same cleanup + summary plumbing inline.
- `diag_suite` now also routes tooling failure script-result writes, row shaping, and main
  finalize wiring through dedicated helpers, so DevTools/connect/launch failure branches stop
  repeating the same tooling-error bookkeeping inline.
- `diag_suite` now also routes failed/unexpected/lint-failed script outcomes through a dedicated
  exit helper, so row shaping and failure finalization no longer interleave inline with outcome
  logging in those branches.
- `diag_suite` now also prepares per-script stage/reason accounting plus evidence/lint context through
  a dedicated helper, so each script iteration no longer reassembles that bookkeeping inline after
  transport execution completes.
- `diag_suite` now also routes dump-label planning, `run_script_over_transport` lowering, and
  `tooling.suite.error` fallback through dedicated helpers, so the transport result decoding path no
  longer stays as one monolithic inline block in the script execution loop.
- `diag_suite` now also routes prewarm/prelude execution and load-script wiring through a dedicated
  execution-block context, so the main script loop no longer interleaves that setup with transport
  result decoding in one large closure.
- `diag_suite` now also routes per-script launch env/default assembly plus connected transport
  acquisition through `SuiteScriptLaunchRequest`, `SuiteScriptTransportRequest`, and
  `SuiteScriptTransportSelection`, so `maybe_launch_demo` and filesystem-vs-DevTools selection no
  longer expand inline in the main script loop.
- `diag_suite` now also routes transport-backed execution dispatch through
  `SuiteScriptExecutionRequest`, so `SuiteScriptExecutionBlockContext` assembly plus
  `execute_suite_script_iteration_block` invocation no longer stay inline in the main script loop.
- `diag_suite` now also routes per-script lint execution plus passed-script post-run preparation
  through `SuiteScriptLintRequest` and `SuiteScriptPostRunPreparationRequest`, so bundle waits,
  bundle doctor application, lint/report wiring, and post-run default-check planning no longer
  expand inline in the main script loop.
- `diag_suite` now also routes per-script result finalization through
  `SuiteScriptStageFinalizeRequest` and `SuiteScriptSuccessFinalizeRequest`, so stage branching,
  success-row emission, and stop-demo teardown no longer share one inline tail block.
- `diag_suite` now also routes the remaining per-script success tail through
  `SuiteScriptSuccessTailRequest`, so lint-failure exit, post-run apply, and success finalize
  orchestration no longer re-expand inline in the main script loop.
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
- stable reason-code expectations for orchestrated runs,
- a decision on whether residual manifest names such as `files[].id` should gain canonical aliases
  now or stay deferred until the next compatibility window.

## What should be deferred for now

The following items are still valid, but they should not jump ahead of the priorities above:

- dashboard text or HTML projection,
- packing screenshot PNG bodies into combined failure zips,
- TOML or generated campaign manifests,
- removing legacy top-level `suites` / `scripts` compatibility,
- GUI-only workflow polish.

These are meaningful follow-ups once the contract and seam story are more settled.

## Recommended next implementation sequence

1. finish cross-linking the canonical artifact/evidence and M3 vocabulary contract updates across the remaining diagnostics notes,
2. inspect residual naming in persisted artifacts and consumers, especially manifest `files[].id`
   style identifiers and any older consumer-side exact-name checks,
3. land only additive compatibility bridges for those residual names,
4. revisit deeper orchestration seam work only if a clearly reviewable holdout remains higher ROI
   than residual-vocabulary adoption.

## Bottom line

The workstream is now past the point of proving that campaign automation is viable.

The next stage should optimize for:

- clearer contracts,
- additive vocabulary adoption,
- smaller orchestration seams,
- and more stable scaling vocabulary,

not for more output formats or more GUI polish first.
