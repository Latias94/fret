---
title: M3 Vocabulary Adoption Audit
status: draft
date: 2026-03-09
scope: diagnostics, regression, vocabulary, adoption, audit
---

# M3 Vocabulary Adoption Audit

Status: Draft

Tracking docs:

- `docs/workstreams/diag-fearless-refactor-v2/M3_ORCHESTRATION_VOCABULARY_AND_CONTRACT_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/CURRENT_STATUS_AND_PRIORITIES.md`
- `docs/workstreams/diag-fearless-refactor-v2/TODO.md`
- `docs/workstreams/diag-fearless-refactor-v2/CAMPAIGN_METADATA_EXECUTION_ADOPTION_AUDIT.md`
- `docs/workstreams/diag-fearless-refactor-v2/DEVTOOLS_MCP_RAW_JSON_DEFER_AUDIT.md`
- `docs/workstreams/diag-fearless-refactor-v2/CAMPAIGN_CAPABILITY_PREFLIGHT_V1.md`

## 0) Why this audit exists

The repo now has a stronger M3 vocabulary contract.

What it still needs is one bounded implementation audit answering:

- which persisted machine fields already follow the contract,
- which surfaces are only partially aligned,
- which residual names are intentionally deferred,
- and what the next additive adoption pass should actually touch.

This note is intentionally narrow.
It does not restate the full vocabulary contract.
It audits adoption against that contract.

## 1) Audit scope

Included:

- persisted machine-readable artifacts under `crates/fret-diag`,
- campaign/summary/index/share vocabulary that crosses more than one consumer,
- shared aggregate consumers used by CLI, GUI, and MCP.

Explicitly out of scope for this pass:

- raw JSON text-holder names inside DevTools or MCP state models,
- internal streaming helper names such as `bundle_json_deserializer`,
- Layer A run-manifest chunk subtree naming such as `bundle_json` chunk indexes,
- purely presentation-local wording that does not persist a machine field.

## 2) Audit summary

Current conclusion:

- the repo is **mostly aligned** on normalized lane, status, and reason-code vocabulary,
- the repo is **partially aligned** on evidence-path vocabulary because canonical names now exist
  but a few legacy aliases or deferred layers remain,
- the main remaining contract gap is **campaign metadata adoption** rather than another summary or
  dashboard wording pass.

In practical terms:

- the next M3 pass should be a small persisted-field adoption pass,
- not another terminology draft,
- and not another broad naming sweep across raw JSON holder variables.

## 3) Adoption matrix

| Surface | Status | Evidence | Audit note |
| --- | --- | --- | --- |
| `RegressionLaneV1` + `RegressionStatusV1` in summary artifacts | Aligned | `crates/fret-diag/src/regression_summary.rs` | summary serialization already normalizes canonical lane/status vocabulary |
| summary reason-code vocabulary | Aligned | `crates/fret-diag/src/regression_summary.rs`, `crates/fret-diag/src/diag_summarize.rs`, `crates/fret-diag/src/diag_dashboard.rs` | `reason_code` and `source_reason_code` are first-class machine fields and aggregate consumers count `top_reason_codes` directly |
| summary evidence-path vocabulary | Aligned (with known additive projections) | `crates/fret-diag/src/regression_summary.rs` | canonical names exist and legacy aliases are reader-compatible; `perf_summary_json` / `compare_json` are now explicitly classified as projection-only additive evidence fields |
| run-manifest path vocabulary | Partially aligned | `crates/fret-diag/src/run_artifacts.rs` | canonical `script_result` and `bundle_artifact` paths exist additively; Layer A `bundle_json` chunk subtree remains intentionally deferred |
| campaign persisted lane vocabulary | Aligned | `crates/fret-diag/src/registry/campaigns.rs`, `crates/fret-diag/src/regression_summary.rs` | machine JSON emitted through `RegressionLaneV1` is canonical-first; `full` remains acceptable as an input/presentation alias |
| campaign metadata vocabulary | Aligned (with known gaps) | `crates/fret-diag/src/registry/campaigns.rs` | campaign manifests now expose `requires_capabilities` and `flake_policy` additively, but execution behavior does not consume them yet |
| aggregate CLI / GUI / MCP reason aggregation | Aligned | `crates/fret-diag/src/diag_dashboard.rs`, `apps/fret-devtools/src/native.rs`, `apps/fret-devtools-mcp/src/native.rs` | aggregate consumers already reuse normalized `reason_code`-centric projections instead of parsing ad hoc labels |
| DevTools / MCP raw bundle or script-result text holders | Deferred by design | `apps/fret-devtools/src/native.rs`, `apps/fret-devtools-mcp/src/native.rs` | current `*json` names in these modules mostly hold raw JSON text, not canonical artifact-path contract fields |

## 4) Detailed findings

### 4.1 Summary lane/status/reason vocabulary is in good shape

Evidence:

- `RegressionStatusV1` already persists the canonical status set,
- `RegressionLaneV1` serializes `Nightly` and `Full` as canonical `nightly`,
- summary highlights and aggregate projections already count `reason_code` directly.

What this means:

- the next M3 pass should not reopen lane/status naming in summary artifacts,
- any remaining work here is additive reader compatibility only.

### 4.2 Summary evidence vocabulary is mostly good, but still has bounded residuals

Evidence:

- `RegressionEvidenceV1` already prefers canonical names such as `triage_artifact`,
  `script_result`, `share_artifact`, and `packed_report`,
- the same struct still accepts documented legacy aliases such as `triage_json`,
  `script_result_json`, `ai_packet_dir`, and `pack_path`.

Residuals:

- `perf_summary_json` and `compare_json` still exist as bounded projection-specific fields,
- `RegressionArtifactsV1.index_json` remains a canonical pointer inside the derived summary/index
  layer rather than a source-of-truth field.

Recommended interpretation:

- do not rename these immediately,
- keep `index_json` as canonical within the derived summary/index layer,
- keep `perf_summary_json` and `compare_json` as explicit projection-only additive fields rather
  than promoting them into the generic cross-surface artifact vocabulary.

### 4.3 Run-manifest adoption is intentionally split across two layers

Evidence:

- `RunManifestPathsV1` now writes canonical `script_result` and `bundle_artifact` paths while
  retaining documented reader aliases,
- `files[].id` now prefers canonical `script_result`,
- the chunked run-manifest subtree still uses `bundle_json`.

Audit judgment:

- this is not an accidental inconsistency,
- it is the already-documented Layer A versus Layer B split.

Recommended action:

- keep Layer A deferred,
- keep Layer B canonical-first,
- do not fold the Layer A chunk subtree into a broad rename wave.

### 4.4 Campaign metadata additive adoption is now landed, with behavior follow-up still deferred

Evidence:

- campaign manifests and built-in registry definitions already persist:
  - `lane`,
  - `owner`,
  - `platforms`,
  - `tier`,
  - `expected_duration_ms`,
  - `tags`,
- and the current campaign manifest/registry contract now also exposes:
  - `requires_capabilities`,
  - `flake_policy`.

Why this matters:

- the contract note now names both concepts explicitly,
- campaign selection and CI-facing orchestration are the natural place for those fields,
- and landing them as optional manifest fields closes the most obvious producer-side schema drift.

What is now aligned:

- older manifests remain loadable without these fields,
- capability tags are normalized into a lowercase machine-readable list,
- flake policy is surfaced in campaign JSON plus maintainer-facing `list/show` output.

Remaining follow-up:

- this pass does **not** yet make campaign selection/filtering depend on these fields,
- and it does **not** yet define a stricter enum-level flake-policy contract.

### 4.5 Aggregate consumers are in better shape than the remaining producers

Evidence:

- CLI dashboard projections aggregate on `top_reason_codes`,
- GUI now reuses shared dashboard projection helpers,
- MCP exposes normalized reason-code rows and not a parallel aggregate schema.

Audit judgment:

- aggregate consumers are no longer the main M3 risk,
- the producer-side metadata/evidence contract is now the higher-leverage target.

### 4.6 DevTools/MCP raw `*json` state names remain deferred

Evidence:

- DevTools and MCP still contain names such as `bundle_json` and `script_result_json`,
- those names are frequently attached to raw text blobs or dump payloads rather than canonical
  artifact-path fields.

Audit judgment:

- these are not the right next rename target,
- touching them only makes sense if those modules are already being changed for another reason.

## 5) Residual adoption list

The next bounded M3 adoption pass should focus on exactly these items:

1. **Residual Layer B review only where it changes machine fields**
   - do not reopen Layer A run-manifest chunk naming,
   - do not reopen DevTools/MCP raw text-holder naming by default.
2. **Campaign behavior follow-up only if execution needs it**
   - keep `requires_capabilities` and `flake_policy` as additive metadata unless campaign
     scheduling, retries, or CI policy actually starts consuming them.
3. **Optional evidence vocabulary review only when a new field ships**
   - future perf/matrix drill-down fields should be classified up front as canonical or
     projection-only instead of relying on implied usage.

## 6) Recommended priority order

Priority order for the next implementation pass:

1. residual Layer B review only where a machine field still drifts,
2. only then campaign behavior consumption if orchestration starts depending on it,
3. and classify any future optional evidence fields before another shared consumer ships.

Reasoning:

- the first campaign metadata hole is now closed additively,
- summary/artifact field classification is now explicit,
- residual evidence-field migration should stay tightly bounded and avoid becoming another generic
  naming sweep,
- campaign behavior should not start consuming the new metadata until there is a concrete
  orchestration need.

## 7) Exit criteria for this audit follow-up

This audit is considered successfully acted on when:

- campaign manifests can express `requires_capabilities` and `flake_policy` additively,
- the status of `index_json` / `perf_summary_json` / `compare_json` is explicit instead of implied,
- and no new shared consumer needs to guess whether a name is canonical, legacy, or projection-only.
