---
title: M3 Orchestration Vocabulary and Contract v1
status: draft
date: 2026-03-09
scope: diagnostics, regression, campaigns, vocabulary, contracts
---

# M3 Orchestration Vocabulary and Contract v1

Status: Draft

Tracking doc: `docs/workstreams/diag-fearless-refactor-v2/README.md`

Related notes:

- `docs/workstreams/diag-fearless-refactor-v2/REGRESSION_CAMPAIGN_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/REGRESSION_SUMMARY_SCHEMA_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/ARTIFACT_AND_EVIDENCE_MODEL_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/CAMPAIGN_EXECUTION_ENTRY_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/IMPLEMENTATION_ROADMAP.md`

## 0) Why this note exists

The workstream already has:

- a first campaign model,
- a first regression summary schema,
- a first artifact/evidence model,
- a first shipped `diag campaign` entry,
- a first aggregate consumer path shared by CLI, GUI, and MCP.

What is still too loose is the **shared vocabulary** between these pieces.

Today the repo is at risk of drift in:

- lane names,
- result wording,
- reason-code usage,
- flake policy wording,
- capability tags,
- artifact and evidence path naming.

That drift would be expensive because more than one consumer already exists:

- CLI,
- DevTools GUI,
- MCP,
- CI,
- local maintainer scripts.

This note defines the first repo-level vocabulary contract that those consumers should share.

## 1) Contract goals

The vocabulary contract should be:

- stable enough for machine consumers,
- small enough to learn quickly,
- additive rather than rewrite-heavy,
- presentation-neutral,
- explicit about what is v1 must-have versus later expansion.

This note does **not** try to list every final reason code or every future capability tag.
It defines the naming system, the minimum shared set, and the compatibility rules.

## 2) Scope

This note covers:

- lane vocabulary,
- stable status vocabulary,
- stable reason-code vocabulary rules,
- flake policy vocabulary,
- capability tag vocabulary,
- repo-level artifact/evidence path vocabulary,
- cross-surface reuse rules for CLI, GUI, MCP, and CI.

This note does not define:

- the full campaign registry schema,
- all future reason codes,
- GUI layout or dashboard wording,
- a new persistence layer.

## 3) Normative terms

For this note:

- **lane** means a repo-level regression execution class such as `smoke` or `perf`,
- **status** means the normalized run outcome class stored in summaries,
- **reason_code** means the stable machine-readable failure or classification code,
- **source_reason_code** means a more local producer-specific reason code preserved for debugging,
- **flake policy** means the retry/classification behavior requested by campaign or suite metadata,
- **capability tag** means a stable requirement or availability label that affects whether an item may run,
- **artifact path** means a canonical persisted output path,
- **evidence path** means a persisted path to supporting debugging material.

## 4) Lane vocabulary

The repo should use the following lane vocabulary as the canonical first set.

| Lane | Meaning | Typical use |
| --- | --- | --- |
| `smoke` | Fast confidence lane | local iteration, PR presubmit |
| `correctness` | Deterministic functional regression lane | curated functional coverage |
| `matrix` | Compare-across-axis lane | cache / shell / runtime toggles |
| `perf` | Performance regression lane | latency / worst-frame / throughput gates |
| `nightly` | Broad scheduled lane | wide coverage outside tight feedback loops |
| `full` | Alias-like selection for broad runs | user-facing convenience, not a new semantic class |

Rules:

- `nightly` is the canonical broad lane name.
- `full` may appear as a user-facing selector, but persisted artifacts should normalize to `nightly`
  unless the repo later decides they are meaningfully different.
- New lanes should be rare and require an explicit workstream or ADR-level justification.

## 5) Stable status vocabulary

The repo should use one normalized status set across `regression.summary.json`,
aggregate indexes, dashboards, and campaign outputs.

Canonical status vocabulary:

- `passed`
- `failed_deterministic`
- `failed_flaky`
- `failed_tooling`
- `failed_timeout`
- `skipped_policy`
- `quarantined`

Rules:

- Summary and campaign artifacts should persist normalized statuses, not ad-hoc prose.
- Presentation surfaces may render friendlier labels, but they must not invent different underlying
  state classes.
- New status classes are a contract change and should be treated as additive versioned work.

## 6) Reason-code vocabulary

## 6.1 Why reason codes exist

Reason codes exist so consumers do not need to parse human text to answer:

- why an item failed,
- whether two failures are the same class,
- which failures dominate a lane,
- which items are tooling failures versus product failures.

## 6.2 Required fields

For any non-passing item, summaries should prefer:

- `reason_code`
- `source_reason_code`

If only one code is available initially, `reason_code` is preferred.

## 6.3 Naming rules

`reason_code` should follow these rules:

- lowercase ASCII,
- dot-separated segments,
- stable semantic category first,
- producer-specific detail later,
- no free-form user text,
- no embedded timestamps, ids, or paths.

Recommended shape:

- `<domain>.<class>[.<detail>]`

Examples:

- `tooling.launch.error`
- `tooling.transport.disconnected`
- `tooling.bundle.missing`
- `script.assertion.failed`
- `script.snapshot.mismatch`
- `policy.capability.missing`
- `policy.quarantined`
- `perf.threshold.exceeded`
- `perf.bundle.missing`

## 6.4 Layering rule

`reason_code` is the cross-surface normalized code.

`source_reason_code` is allowed to stay closer to a specific producer or command path.

That means:

- runtime/tooling producers may emit local details via `source_reason_code`,
- campaign and summary writers should normalize to a stable `reason_code`,
- dashboards and CI should aggregate on `reason_code`,
- debugging views may show both.

## 6.5 Initial reason-code buckets

The first shared bucket families should be:

- `tooling.*`
- `script.*`
- `policy.*`
- `perf.*`

This is enough to stop immediate drift without pretending the full taxonomy is finished.

## 7) Flake policy vocabulary

Campaign and suite metadata should use one shared flake policy vocabulary.

Recommended first set:

- `fail_fast`
- `retry_once`
- `retry_three`
- `classify_only`

Meaning:

- `fail_fast`
  - do not retry beyond the first failure,
- `retry_once`
  - retry once before final classification,
- `retry_three`
  - retry up to three attempts total,
- `classify_only`
  - gather enough evidence to classify likely flake behavior without turning it into silent pass.

Rules:

- policy names describe retry/classification behavior, not final status,
- quarantine remains an explicit result state, not a flake policy synonym,
- free-form policy prose should not be persisted where machine readers expect stable values.

## 8) Capability tag vocabulary

Capability tags exist so campaign/suite selection can remain explicit about environment needs.

Recommended naming rules:

- lowercase ASCII,
- dot-separated,
- capability domain first,
- avoid embedding host-specific incidental details unless they affect run eligibility.

Recommended first families:

- `runtime.*`
- `transport.*`
- `capture.*`
- `platform.*`
- `render.*`

Examples:

- `runtime.devtools_ws`
- `runtime.filesystem`
- `transport.websocket`
- `transport.filesystem`
- `capture.screenshot`
- `capture.bundle_schema2`
- `platform.windows`
- `platform.macos`
- `platform.linux`
- `render.gpu`

Usage rules:

- manifests and summaries should prefer `requires_capabilities` for hard requirements,
- optional descriptive tags should remain separate from hard requirements,
- capability tags should describe execution requirements, not ownership or product area.

## 9) Artifact and evidence path vocabulary

The repo already has an artifact/evidence taxonomy.
This section narrows it into a small path vocabulary for orchestration outputs.

## 9.1 Canonical root names

Recommended canonical roots:

- `campaigns/<campaign_id>/<run_id>/`
- `campaign-batches/<selection_slug>/<run_id>/`
- `suite-results/`
- `share/`

These roots should remain preferred across CLI, CI, GUI, and MCP references.

## 9.2 Canonical top-level files

Recommended canonical file names:

- `campaign.manifest.json`
- `campaign.result.json`
- `batch.manifest.json`
- `batch.result.json`
- `regression.summary.json`
- `regression.index.json`
- `share/share.manifest.json`
- `share/combined-failures.zip`

Rules:

- producers should prefer these names before inventing synonyms,
- presentation surfaces should link to these paths rather than persist parallel copies,
- any additional convenience projection should remain additive.

## 9.3 Evidence path vocabulary inside summaries

When summaries point to supporting outputs, the repo should prefer stable field names aligned with
the artifact model.

Recommended evidence path field vocabulary:

- `bundle_artifact`
- `script_result`
- `triage_artifact`
- `screenshots_manifest`
- `share_artifact`
- `packed_report`

Rules:

- fields should point to persisted artifacts, not to presentation-only concepts,
- absent evidence should be encoded as missing or `null`, not by overloading status,
- consumers should tolerate partial evidence as long as canonical summary status remains readable.

## 9.4 Persisted-field normalization map

The repo now has enough persisted surfaces that the contract should name the minimum normalization
map explicitly.

This is not a full schema listing.
It is the smallest field map that stops new consumers from inventing parallel names.

| Surface | Canonical persisted vocabulary | Legacy aliases still tolerated | Notes |
| --- | --- | --- | --- |
| `regression.summary.json` item rows | `lane`, `status`, `reason_code`, `source_reason_code`, `bundle_artifact`, `script_result`, `triage_artifact`, `screenshots_manifest`, `share_artifact`, `packed_report` | previously emitted older path aliases where already documented in reader compatibility code | summary rows should be the main cross-surface machine handoff |
| `regression.index.json` and aggregate counters | `lane`, normalized `status`, `reason_code` | older human wording is presentation-only and should not become schema | aggregate consumers should count on normalized fields, not parse labels |
| `regression.summary.json` root `artifacts` section | `summary_dir`, `index_json`, `packed_report` | none preferred | `index_json` is canonical for the derived summary/index layer; `html_report` remains presentation-only |
| campaign and batch roots | `campaign.manifest.json`, `campaign.result.json`, `batch.manifest.json`, `batch.result.json`, `regression.summary.json`, `regression.index.json` | none preferred; additive extra files are allowed | root file names are contract-level vocabulary |
| share roots | `share/share.manifest.json`, `share/combined-failures.zip` | none preferred | future convenience outputs should stay additive under `share/` |
| run-manifest `files[].id` | `script_result` | `script_result_json` | canonical write, legacy read compatibility is already the preferred policy |
| Layer B evidence payloads | `bundle_artifact` | `bundle_json` | canonical-first dual-write is allowed where compatibility is still needed |
| capability requirements in manifests/summaries | `requires_capabilities` | ad hoc free-form wording should be treated as non-canonical metadata | capability requirements should remain machine-readable |
| item-row projection-only evidence fields | `perf_summary_json`, `compare_json` | none preferred | these are additive projection/check pointers, not canonical cross-surface artifact vocabulary |

Rules:

- new persisted fields should prefer canonical names from this table before introducing another local
  spelling,
- if a surface needs a new machine-readable field, add it here or to a tighter schema note before
  more than one consumer ships,
- human wording such as dashboard labels or CLI prose should not be treated as schema evidence.

## 10) Cross-surface reuse rules

The vocabulary in this note should be treated as shared repo-level contract.

### CLI

- may render human text freely,
- must persist normalized lane, status, and path vocabulary in machine artifacts.

### DevTools GUI

- may group or relabel for readability,
- must consume the same normalized vocabulary from persisted artifacts or shared helpers.

### MCP

- may expose richer summaries,
- must not invent a parallel status or path schema for the same underlying artifacts.

### CI

- should aggregate on normalized lane, status, and `reason_code`,
- should treat human console wording as non-canonical.

## 11) Compatibility policy

This vocabulary should evolve additively where possible.

### 11.1 Writer policy

Writers should prefer:

1. canonical field/file/root names,
2. additive new canonical fields before removing old ones,
3. temporary dual-write only when a known reader still expects the legacy name.

Writers should avoid:

- writing only a legacy alias in a newly touched path,
- introducing a third spelling for an already normalized concept,
- embedding presentation labels where machine consumers expect a stable code.

### 11.2 Reader policy

Readers should prefer:

- canonical names first,
- explicit fallback to documented legacy aliases,
- one normalization boundary close to deserialization or artifact loading.

Readers should avoid:

- scattering legacy-name fallback across many presentation surfaces,
- re-exporting legacy aliases as if they were still canonical,
- deriving machine state from human labels when canonical fields exist.

### 11.3 Alias lifecycle

The expected lifecycle for vocabulary migration is:

1. document the canonical name,
2. make writers emit the canonical name,
3. keep readers tolerant of documented legacy aliases,
4. audit remaining producers/consumers,
5. only then consider removing dual-write or fallback behavior.

This means "legacy alias tolerated" is not the same as "legacy alias still preferred".
The repo should bias toward canonical-first writes and canonical-first reads.

Allowed additive changes:

- adding new reason codes under an existing family,
- adding new capability tags,
- adding new optional evidence fields,
- adding new convenience projections such as indexes or ranked lists.

Higher-bar changes:

- renaming a lane,
- changing normalized status names,
- changing canonical top-level artifact file names,
- changing the meaning of an existing reason-code family.

If a higher-bar change becomes necessary, it should be treated as explicit contract work and not
as incidental cleanup inside one command or UI surface.

## 12) V1 must-have versus later

## V1 must-have

- one canonical lane set,
- one canonical status set,
- one reason-code naming rule plus initial bucket families,
- one flake policy vocabulary,
- one capability-tag naming rule plus initial families,
- one canonical campaign/batch/share path vocabulary,
- one requirement that CLI, GUI, MCP, and CI reuse these terms.

## Later / additive

- a fuller reason-code registry,
- richer capability catalogs,
- stricter manifest validation,
- richer dashboard-only rankings,
- remote storage or external reporting vocabulary.

## 13) Immediate adoption targets

The next repo-level tightening work should apply this vocabulary to:

1. `REGRESSION_CAMPAIGN_V1.md`
2. `REGRESSION_SUMMARY_SCHEMA_V1.md`
3. `CAMPAIGN_EXECUTION_ENTRY_V1.md`
4. `diag summarize` / `diag dashboard` presentation wording
5. DevTools and MCP aggregate views
6. future CI campaign wrappers

The success condition is not more prose.
The success condition is fewer parallel names for the same concepts.

## 14) Recommended adoption order

The next implementation passes should apply the contract in this order:

1. persisted machine fields first,
   - lane/status/reason fields,
   - evidence path field names,
   - capability requirement field names,
2. shared aggregate consumers second,
   - `diag summarize`,
   - `diag dashboard`,
   - DevTools GUI,
   - MCP,
3. CI and maintainer wrappers third,
   - campaign wrappers,
   - share/report scripts,
   - repo-owned automation notes.

Why this order:

- persisted machine fields are the highest-cost drift point,
- aggregate consumers should normalize once instead of learning parallel names,
- CI and maintainer wrappers should be the last adopters because they often depend on the first two
  layers being stable.

## 15) Definition of done for the first adoption pass

The first adoption pass is good enough when:

- a contributor can inspect one machine artifact and know which lane/status/reason/path names are
  canonical,
- shared consumers no longer need to invent vocabulary that is absent from the persisted artifacts,
- documented legacy aliases are few, explicit, and reader-side only unless dual-write is still
  required for a known compatibility boundary,
- follow-up work can focus on residual audited gaps instead of another terminology draft.
