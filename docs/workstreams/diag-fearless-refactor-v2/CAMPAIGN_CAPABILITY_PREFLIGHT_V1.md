---
title: Campaign Capability Preflight v1
status: draft
date: 2026-03-09
scope: diagnostics, campaign, capability, preflight, contract
---

# Campaign Capability Preflight v1

Status: Partially implemented

Tracking docs:

- `docs/workstreams/diag-fearless-refactor-v2/CAMPAIGN_METADATA_EXECUTION_ADOPTION_AUDIT.md`
- `docs/workstreams/diag-fearless-refactor-v2/REGRESSION_CAMPAIGN_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/CAMPAIGN_EXECUTION_ENTRY_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/M3_ORCHESTRATION_VOCABULARY_AND_CONTRACT_V1.md`

## 0) Why this note exists

Campaign manifests can now persist:

- `requires_capabilities`
- `flake_policy`

The execution-adoption audit already concluded that these fields should remain passive until a real
consumer appears.

If that consumer appears, the first safe behavior to adopt is not flake automation.

It is campaign-level capability preflight.

This note defines the smallest version of that behavior.

## 1) Goal

The goal is simple:

- if a campaign declares capability requirements,
- evaluate them once before item execution starts,
- and emit one explicit machine-readable decision instead of failing later in scattered per-item
  ways or silently skipping selection.

## 2) Non-goals

This note does **not** propose:

- a new capability probing subsystem,
- a second retry engine,
- broad new CLI filtering syntax,
- campaign-level mutation of lower-level script or suite evidence formats.

## 3) Existing building blocks

The repo already has reusable capability pieces:

### 3.1 Capability normalization and loading

Existing logic already reads filesystem capabilities and normalizes them into a stable list.

Evidence:

- `crates/fret-diag/src/lib.rs`

### 3.2 Capability check payload

Existing logic already produces a machine-readable check payload with:

- `source`
- `required`
- `available`
- `missing`

Evidence:

- `crates/fret-diag/src/lib.rs`

### 3.3 Existing reason-code outcome

Lower-level capability gating already uses:

- `reason_code = "capability.missing"`
- `check.capabilities.json`

Evidence:

- `crates/fret-diag/src/lib.rs`
- `crates/fret-diag/src/diag_repro.rs`

### 3.4 Existing campaign result and report surfaces

Campaign execution already emits:

- `campaign.result.json`
- `campaign.manifest.json`
- run/batch JSON projections

Evidence:

- `crates/fret-diag/src/diag_campaign.rs`

## 4) Proposed insertion point

Preflight should run:

- after campaign selection,
- before the first suite/script item is executed,
- once per selected campaign.

Recommended insertion point:

- immediately before `execute_campaign_inner(...)` begins item execution,
- or as an explicit stage inside `build_campaign_execution_start_plan(...)`.

Important rule:

- preflight must be visible as a first-class campaign step in the execution story,
- not hidden inside the first item.

## 5) Preflight inputs

The preflight should consume:

- `campaign.requires_capabilities`
- the effective available capability set for the current transport/runtime

First version recommendation:

- reuse the same normalized capability vocabulary already used by lower-level checks,
- load available capabilities from the existing filesystem capability source when available,
- otherwise treat the capability set as empty rather than inventing inferred pseudo-capabilities.

## 6) Preflight outputs

The first version should produce exactly two machine-readable artifacts:

### 6.1 A campaign-local capability check file

Recommended path:

- `<campaign_root>/check.capabilities.json`

Recommended shape:

- `schema_version`
- `status`
- `source`
- `required`
- `available`
- `missing`

Recommendation:

- reuse the existing check payload shape as closely as possible.

### 6.2 A campaign result decision

If preflight fails, `campaign.result.json` should persist an explicit campaign-level outcome.

At minimum it should include:

- a normalized `reason_code`
- the path to `check.capabilities.json`
- a clear campaign-level status/error section

## 7) Status and reason-code policy

### 7.1 Reason code

Use:

- `capability.missing`

Reason:

- the repo already uses this reason code for lower-level capability gating,
- reusing it avoids a second campaign-only capability taxonomy.

### 7.2 Status

Recommendation for v1:

- treat campaign capability mismatch as `skipped_policy`

Why:

- the campaign did not fail because an executed item found a deterministic product defect,
- it was blocked by declared execution policy and environment mismatch,
- `skipped_policy` already exists as a normalized result bucket.

Explicitly not recommended for v1:

- `failed_tooling`
  - too broad and blurs environmental policy with internal tool failure,
- `quarantined`
  - wrong semantics,
- a new status
  - unnecessary contract growth.

## 8) Result shaping recommendation

The cleanest first version is:

- create one synthetic `RegressionItemSummaryV1` row of kind `CampaignStep`,
- name it something like `capability_preflight`,
- set:
  - `status = skipped_policy`
  - `reason_code = capability.missing`
  - `lane = campaign lane`
- attach evidence:
  - `extra.capabilities_check_path`
  - or a dedicated path pointer if the summary contract later grows one.

Why this shape is useful:

- aggregate summaries already understand item rows,
- dashboards already aggregate normalized status and reason-code data,
- no new summary-level status mechanism is needed.

## 9) CLI and JSON behavior

If preflight blocks execution:

- human output should say the campaign was skipped by capability policy,
- JSON output should make the decision visible in the run report,
- explicit campaign ids should still appear in the selected set rather than disappearing.

Important rule:

- a capability mismatch must never look like "campaign not selected".

## 10) Interaction with lower-level capability gates

Campaign preflight should be a coarse gate, not a replacement.

That means:

- if campaign preflight already fails, item execution should not start,
- if campaign preflight passes, lower-level script/suite gates may still fail on more specific
  item requirements,
- lower-level `check.capabilities.json` files remain valid and useful.

This preserves layering:

- campaign preflight = run-level eligibility,
- script/suite gates = item-level evidence and enforcement.

## 11) Recommended landing order

### Step 1

Document the contract and keep behavior deferred.

Status:

- this note.

### Step 2

Add a campaign-local preflight helper that:

- computes `required`,
- reads `available`,
- writes `<campaign_root>/check.capabilities.json`,
- returns a typed preflight outcome.

Status:

- landed in first form.
- the implementation now reuses the shared filesystem capability loader in
  `crates/fret-diag/src/lib.rs` instead of keeping a campaign-local capability reader.
- capability discovery now follows the same fallback order already used by diagnostics tooling:
  direct `capabilities.json`, `_root/capabilities.json`, then parent `capabilities.json`.

### Step 3

If preflight fails, short-circuit item execution and emit:

- one synthetic campaign-step summary row,
- one explicit campaign result entry,
- one stable human output line.

Status:

- landed in first form.
- single-run human output now distinguishes `skipped_policy` from generic failure.
- batch human output and run-report JSON now also distinguish `skipped_policy` from ordinary
  execution failure.

### Step 4

Only after that, consider whether batch output needs a dedicated preflight counter.

Status:

- landed in additive form.
- batch counters now expose `campaigns_skipped_policy`.

## 12) Deliberate deferrals

Still defer:

- campaign-level `flake_policy` execution behavior,
- CLI selectors such as `--require-capability`,
- transport-specific capability probing APIs,
- new canonical summary evidence fields just for preflight.

## 13) Definition of done for the first implementation

The first implementation is done when:

- a campaign can declare `requires_capabilities`,
- execution performs a visible preflight before item execution,
- capability mismatch produces machine-readable evidence under the campaign root,
- the campaign run/report surfaces show a normalized status and `reason_code`,
- and no lower-level evidence contracts need to be rewritten to support it.

## 14) Current implementation notes

The current implementation now does the following:

- runs campaign-local capability preflight before item execution starts,
- writes `<campaign_root>/check.capabilities.json`,
- writes a synthetic `CampaignStep` summary row with:
  - `status = skipped_policy`
  - `reason_code = capability.missing`
- writes `campaign.result.json` with aggregate visibility of
  `capabilities_check_path`,
- emits single-run and batch human output that distinguishes policy skips from ordinary failures,
- records the capability source path used for the decision.

The capability source is now aligned with the shared diagnostics loader:

- `crates/fret-diag/src/lib.rs` resolves capability input using:
  - `<base>/capabilities.json`
  - `<base>/_root/capabilities.json`
  - `<base>/../capabilities.json`
- `diag campaign` preflight reuses that helper,
- `diag doctor` now reports both raw and normalized capabilities from the same source path.

This means the current remaining gap is no longer basic preflight behavior.

The next likely follow-up is one of:

- document/consumer adoption of the new policy-skip counter and source-path fields,
- or a later explicit decision on whether any non-filesystem transport should expose the same
  capability-source contract.
