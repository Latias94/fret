# Residual Naming Audit

Status: Draft

Tracking context:

- `docs/workstreams/diag-fearless-refactor-v2/M3_ORCHESTRATION_VOCABULARY_AND_CONTRACT_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/CURRENT_STATUS_AND_PRIORITIES.md`
- `docs/workstreams/diag-fearless-refactor-v2/REGRESSION_SUMMARY_SCHEMA_V1.md`

## Purpose

This note records the remaining naming drift after the first M3 vocabulary adoption pass.

The goal is not to trigger a broad rename. The goal is to identify:

- which residual names are still part of persisted contracts,
- which names are only internal implementation details,
- which cases need an additive compatibility bridge now,
- and which cases should stay deferred until a later cleanup window.

## Audit outcome

### Highest-priority next move

The highest-ROI next step is:

1. keep the current additive compatibility policy,
2. keep canonical `run manifest` `files[].id` naming now that `script_result` has landed,
3. continue the next small-batch Layer B payload cleanup around `bundle_artifact`,
4. leave broader internal field renames for later.

This is the best next slice because the sharpest persisted-contract inconsistency is now resolved,
and the remaining high-value work is additive payload cleanup rather than another break-prone
rename wave.

## Findings by priority

### P0 — Persisted run-manifest `files[].id` canonicalization is now landed

Current state:

- `crates/fret-diag/src/run_artifacts.rs:80`
- `crates/fret-diag/src/run_artifacts.rs:84`
- `crates/fret-diag/src/run_artifacts.rs:111`
- `crates/fret-diag/src/run_artifacts.rs:236`
- `crates/fret-diag/src/run_artifacts.rs:398`
- `crates/fret-diag/src/run_artifacts.rs:402`

Observations:

- `paths.script_result` and `paths.bundle_artifact` are already canonicalized.
- The same manifest now writes canonical `files[].id = "script_result"` for the indexed file entry.
- Reader-side compatibility for legacy `script_result_json` is still retained where old manifests
  or fixtures may appear.
- This removes the previous avoidable split inside one persisted artifact: canonical `paths.*`,
  legacy `files[].id`.

Known in-repo impact:

- `crates/fret-diag/src/artifact_lint.rs:706`
- `crates/fret-diag/src/artifact_lint.rs:874`
- `crates/fret-diag/src/commands/screenshots.rs:437`

Recommendation:

- Keep `files[].id = "script_result"` as the write-side source of truth.
- Keep reader-side compatibility for legacy `script_result_json` where a reader already inspects
  file IDs or test fixtures.
- Treat this slice as done unless another persisted file ID is shown to still drift.

Why this mattered first:

- It improves a persisted contract directly.
- The in-repo consumer/fixture count is small.
- It matches the current additive-migration strategy already used for summary fields and manifest
  `paths`.

### P1 — `RegressionEvidenceV1` still uses legacy Rust field names internally

Current state:

- `crates/fret-diag/src/regression_summary.rs:353`
- `crates/fret-diag/src/regression_summary.rs:360`
- `crates/fret-diag/src/regression_summary.rs:367`
- `crates/fret-diag/src/regression_summary.rs:374`

Observations:

- The serialized contract is already canonical:
  - `triage_artifact`
  - `script_result`
  - `share_artifact`
  - `packed_report`
- The Rust struct field names are still legacy:
  - `triage_json`
  - `script_result_json`
  - `ai_packet_dir`
  - `pack_path`

Known write sites:

- `crates/fret-diag/src/diag_suite.rs:263`
- `crates/fret-diag/src/diag_repeat.rs:85`
- `crates/fret-diag/src/diag_matrix.rs:71`
- `crates/fret-diag/src/diag_perf.rs:137`
- `crates/fret-diag/src/diag_campaign.rs:3280`

Recommendation:

- Defer this rename until after the current Layer B payload cleanup wave.
- When it happens, do it as one compiler-assisted internal rename pass.
- Treat it as an implementation readability cleanup, not a contract migration.

Why this is not first:

- It does not improve the persisted JSON shape further; serde already writes canonical names.
- It touches many constructors and tests for limited immediate user-facing value.

### P1 — Campaign share manifest still intentionally dual-writes legacy fields

Current state:

- `crates/fret-diag/src/diag_campaign.rs:2443`
- `crates/fret-diag/src/diag_campaign.rs:2444`
- `crates/fret-diag/src/diag_campaign.rs:2451`
- `crates/fret-diag/src/diag_campaign.rs:2452`
- `crates/fret-diag/src/diag_campaign.rs:6290`
- `crates/fret-diag/src/diag_campaign.rs:6295`

Observations:

- Share-manifest payload already writes canonical fields:
  - `triage_artifact`
  - `share_artifact`
- It also still writes legacy fields:
  - `triage_json`
  - `share_zip`

Recommendation:

- Keep the current dual-write behavior for now.
- Do not remove legacy keys until consumer coverage is clearer.
- If we later tighten this surface, do it as a dedicated compatibility-window change with explicit
  tests.

Why this should stay additive:

- This is already the intended compatibility bridge.
- Removing legacy fields now would create a larger compatibility risk than the naming benefit.

### P1 — Layer B payload families still need additive `bundle_artifact` adoption

Current state:

- `crates/fret-diag/src/diag_repeat.rs:810`
- `crates/fret-diag/src/diag_repeat.rs:811`
- `crates/fret-diag/src/diag_repro.rs:538`
- `crates/fret-diag/src/diag_repro.rs:695`
- `crates/fret-diag/src/diag_repro.rs:696`
- `crates/fret-diag/src/diag_repro.rs:821`
- `crates/fret-diag/src/diag_repro.rs:822`
- `crates/fret-diag/src/diag_repro.rs:823`
- `crates/fret-diag/src/diag_repro.rs:824`

Observations:

- `diag_repro` and `diag_repeat` now already treat `bundle_artifact` as primary while retaining
  `bundle_json` as a compatibility alias.
- The shared stats helper already covers:
  - `notify_gates`,
  - `notify_gates_streaming`,
  - `gc_gates`,
  - `gc_gates_streaming`,
  - `stale`,
  - `stale_streaming`,
  - `debug_stats_gates`,
  - `frames_index_gates`,
  - `retained_vlist_gates`,
  - `view_cache_gates`,
  - `vlist`,
  - `windowed_rows`.
- Some remaining stats payload families still write both `bundle_json` and `bundle_artifact`
  without using the shared helper.
- This is the same residual vocabulary pattern as `full` vs `nightly`: one canonical term plus one
  older alias.

Recommendation:

- Keep taking this in small batches.
- Prefer remaining stats payload families with repetitive evidence builders over Layer C internal
  variable renames.
- Keep `bundle_artifact` primary and `bundle_json` as the retained compatibility alias until
  external-consumer coverage is clearer.
- After the stats-tree helper migration, continue with canonical-first reader adoption in
  downstream consumers that still probe only `selected_bundle_json` / `packed_bundle_json`.

Why this is now the next priority:

- The sharpest persisted-contract inconsistency inside `run manifest` is already fixed.
- The remaining work is additive and reviewable if kept to small payload-family batches.
- The next low-risk value after payload cleanup is reader-side canonical-first adoption, because it
  reduces accidental reintroduction of legacy-first assumptions without changing persisted payloads.

Reader-side scan update:

- `evidence_index.rs` was the main remaining small reader-side artifact-path consumer still
  preferring legacy `selected_bundle_json` / `packed_bundle_json`.
- After that landing, the remaining `fret-diag` hits mostly fall into three buckets:
  - Layer A manifest/chunk-index handling (`artifact_lint`, `doctor`, `run_artifacts`), which is
    intentionally deferred,
  - intentional write-side dual-write in payload producers such as `diag_repro` and campaign/share
    payloads,
  - internal Rust field names such as `RegressionEvidenceV1::{triage_json, script_result_json}`,
    which are readability cleanup rather than contract migration.

Implication:

- there is no longer another obvious tiny reader-side canonical-first patch in `crates/fret-diag`
  with better ROI than either:
  - stopping here for the naming pass, or
  - doing one deliberate compiler-assisted internal field rename wave later.

### P2 — Internal UI/MCP variable names are not a contract problem

Examples:

- `apps/fret-devtools/src/native.rs:127`
- `apps/fret-devtools/src/native.rs:137`
- `apps/fret-devtools-mcp/src/native.rs:590`
- `apps/fret-devtools-mcp/src/native.rs:1686`

Observations:

- Names such as `last_script_result_json`, `last_bundle_json`, or `pack_path` still exist in app
  state and presentation plumbing.
- These are internal UI/MCP names, not persisted cross-tool contracts.
- The current DevTools/MCP residuals split into two different buckets:
  - **raw JSON text buffers**, such as `last_script_result_json`, `last_bundle_json`, and the MCP
    `fret_diag_bundle_json_latest` / `bundle_json: Option<String>` result shape,
  - **artifact-path vocabulary**, such as summary/evidence readers that choose between
    `bundle_artifact` and `bundle_json`.
- The first bucket is usually not naming drift at all: those fields really do store raw JSON text,
  not bundle-artifact paths.
- The second bucket is where canonical-first adoption still matters, because that affects how
  downstream tools interpret persisted artifact relationships.

Recommendation:

- Defer.
- Only rename when touching those modules for another behavior reason.
- Do not spend churn on renaming raw JSON text holders just to remove the `json` suffix.
- Prefer future effort on the remaining reader-side artifact-path aliases outside the `stats` tree,
  for example summary/evidence consumers that still probe only legacy `selected_bundle_json` /
  `packed_bundle_json` fields.

Why this is deferred:

- Renaming these DevTools/MCP fields would be readability-only churn in user-facing app shells.
- Several names are semantically accurate because they hold serialized `bundle.json` text, not
  artifact-path contracts.
- The higher-value remaining work is still in canonical-first artifact-path consumption, not in
  local UI state naming.

## Recommended execution order

1. Keep `run manifest` `files[].id = "script_result"` as the settled write-side default.
2. Treat small reader-side canonical-first adoption inside `crates/fret-diag` as mostly complete
   for this workstream slice.
3. Audit whether any remaining `bundle_json` payload aliases are consumed by external scripts.
4. Only then consider the internal `RegressionEvidenceV1` field rename pass.
5. Keep campaign share-manifest dual-write behavior unchanged until a dedicated compatibility review.
6. Leave DevTools/MCP raw JSON text holder names alone unless those modules are already changing
   for another feature.

## Non-goals for the next slice

- removing `triage_json` / `share_zip` from campaign share manifests,
- broad renaming across DevTools or MCP app state,
- renaming every internal variable that still contains `json` in the name,
- reopening large `commands::artifacts` seam work unless the residual naming path gets blocked.

## Definition of done for the next slice

- Remaining Layer B payload builders prefer `bundle_artifact` as the write-side primary field.
- Retained `bundle_json` fields are explicitly treated as compatibility aliases.
- Existing legacy payload readers remain compatible where they already exist.
- Raw JSON text holders in DevTools/MCP are clearly classified as non-contract names and therefore
  not part of the current rename budget.
- The remaining `fret-diag` naming work is explicitly narrowed to either deferred Layer A contract
  decisions or a future internal field-rename wave, not more opportunistic small reader-side
  patches.
- Workstream docs explain clearly that Layer A remains intentionally deferred while Layer B is under
  additive adoption.
